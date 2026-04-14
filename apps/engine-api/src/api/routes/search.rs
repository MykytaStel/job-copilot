use std::collections::HashMap;

use axum::extract::{Query, State};
use serde::Deserialize;
use serde_json::json;

use crate::api::dto::search::{RunSearchRequest, RunSearchResponse, SearchResponse};
use crate::api::error::{ApiError, ApiJson};
use crate::api::routes::events::log_user_event_softly;
use crate::api::routes::jobs::load_feedback_state;
use crate::domain::feedback::model::{CompanyFeedbackStatus, JobFeedbackState};
use crate::domain::user_event::model::{CreateUserEvent, UserEventType};
use crate::services::matching::RankedJob;
use crate::state::AppState;

/// Bonus applied to score when the job's company is whitelisted for the profile.
const WHITELIST_SCORE_BONUS: u8 = 10;

/// Penalty subtracted from score when the exact job is marked as bad fit.
const BAD_FIT_SCORE_PENALTY: u8 = 30;

#[derive(Deserialize)]
pub struct SearchQuery {
    pub q: String,
    pub limit: Option<i64>,
    pub page: Option<i64>,
}

pub async fn search(
    State(state): State<AppState>,
    Query(query): Query<SearchQuery>,
) -> Result<axum::Json<SearchResponse>, ApiError> {
    let q = query.q.trim();

    if q.is_empty() {
        return Err(ApiError::bad_request(
            "invalid_search_query",
            "Query parameter 'q' must not be empty",
        ));
    }

    let per_page = query.limit.unwrap_or(20).clamp(1, 50);
    let page = query.page.unwrap_or(1).max(1);
    let offset = (page - 1) * per_page;

    // Fetch one extra row so we can cheaply detect whether a next page exists.
    let mut jobs = state
        .jobs_service
        .search(q, per_page + 1, offset)
        .await
        .map_err(|error| ApiError::from_repository(error, "search_query_failed"))?;

    let has_more = jobs.len() as i64 > per_page;
    jobs.truncate(per_page as usize);

    Ok(axum::Json(SearchResponse::from_jobs(
        jobs, page, per_page, has_more,
    )))
}

pub async fn run_search(
    State(state): State<AppState>,
    ApiJson(payload): ApiJson<RunSearchRequest>,
) -> Result<axum::Json<RunSearchResponse>, ApiError> {
    let input = payload.validate()?;
    let fetch_limit = (input.limit * 5).clamp(50, 200);

    let candidate_jobs = state
        .jobs_service
        .list_filtered_views(fetch_limit, Some("active"), None)
        .await
        .map_err(|error| ApiError::from_repository(error, "search_run_failed"))?;
    let total_candidates = candidate_jobs.len();
    let feedback_states =
        load_feedback_state(&state, input.profile_id.as_deref(), &candidate_jobs).await?;
    let mut filtered_out_hidden = 0usize;
    let mut filtered_out_company_blacklist = 0usize;
    let jobs_with_feedback = candidate_jobs
        .into_iter()
        .zip(feedback_states.into_iter())
        .filter_map(|(job, feedback)| {
            if feedback.hidden {
                filtered_out_hidden += 1;
                return None;
            }

            if feedback.company_status == Some(CompanyFeedbackStatus::Blacklist) {
                filtered_out_company_blacklist += 1;
                return None;
            }

            Some((job, feedback))
        })
        .collect::<Vec<_>>();
    let mut feedback_by_job_id = std::collections::HashMap::new();
    let ranked_candidates = jobs_with_feedback
        .iter()
        .map(|(job, feedback)| {
            feedback_by_job_id.insert(job.job.id.clone(), feedback.clone());
            job.clone()
        })
        .collect::<Vec<_>>();

    let result = state.search_matching_service.run(
        &input.search_profile,
        ranked_candidates,
    );
    // Apply feedback-aware score adjustments, re-sort, then truncate.
    // Truncation happens here (after feedback) so that a whitelisted job ranked
    // just outside the limit by pure score can still be promoted into the result
    // set, and a bad-fit job with a high initial score can be demoted out of it.
    let mut adjusted_jobs = apply_feedback_scoring(result.ranked_jobs, &feedback_by_job_id);
    adjusted_jobs.truncate(input.limit as usize);
    let ranked_jobs: Vec<crate::api::dto::search::RankedJobResponse> = adjusted_jobs
        .into_iter()
        .map(|ranked| {
            let feedback = feedback_by_job_id
                .get(&ranked.job.job.id)
                .cloned()
                .unwrap_or_default();

            crate::api::dto::search::RankedJobResponse {
                job: crate::api::dto::jobs::JobResponse::from_view_with_feedback(
                    ranked.job, feedback,
                ),
                fit: crate::api::dto::search::JobFitResponse::from(ranked.fit),
            }
        })
        .collect();

    let meta = crate::api::dto::search::SearchRunMetaResponse {
        total_candidates,
        filtered_out_by_source: result.filtered_out_by_source,
        filtered_out_hidden,
        filtered_out_company_blacklist,
        scored_jobs: total_candidates
            .saturating_sub(result.filtered_out_by_source)
            .saturating_sub(filtered_out_hidden)
            .saturating_sub(filtered_out_company_blacklist),
        returned_jobs: ranked_jobs.len(),
    };

    if let Some(profile_id) = input.profile_id.clone() {
        let allowed_sources = input
            .search_profile
            .allowed_sources
            .iter()
            .map(|source| source.canonical_key().to_string())
            .collect::<Vec<_>>();
        let primary_source = match allowed_sources.as_slice() {
            [source] => Some(source.clone()),
            _ => None,
        };

        log_user_event_softly(
            &state,
            CreateUserEvent {
                profile_id,
                event_type: UserEventType::SearchRun,
                job_id: None,
                company_name: None,
                source: primary_source,
                role_family: input
                    .search_profile
                    .primary_role
                    .family()
                    .map(str::to_string),
                payload_json: Some(json!({
                    "limit": input.limit,
                    "primary_role": input.search_profile.primary_role.canonical_key(),
                    "primary_role_confidence": input.search_profile.primary_role_confidence,
                    "target_roles": input
                        .search_profile
                        .target_roles
                        .iter()
                        .map(|role| role.canonical_key())
                        .collect::<Vec<_>>(),
                    "allowed_sources": allowed_sources,
                    "target_regions": input.search_profile.target_regions,
                    "work_modes": input.search_profile.work_modes,
                    "search_terms": input.search_profile.search_terms,
                    "exclude_terms": input.search_profile.exclude_terms,
                    "meta": {
                        "total_candidates": meta.total_candidates,
                        "filtered_out_by_source": meta.filtered_out_by_source,
                        "filtered_out_hidden": meta.filtered_out_hidden,
                        "filtered_out_company_blacklist": meta.filtered_out_company_blacklist,
                        "scored_jobs": meta.scored_jobs,
                        "returned_jobs": meta.returned_jobs,
                    }
                })),
            },
        )
        .await;
    }

    Ok(axum::Json(RunSearchResponse {
        meta,
        results: ranked_jobs,
    }))
}

/// Adjust fit scores based on explicit job feedback, then re-sort by adjusted score.
///
/// - Company whitelisted for this profile → +WHITELIST_SCORE_BONUS (capped at 100)
/// - Job marked as bad fit by this profile → −BAD_FIT_SCORE_PENALTY (floored at 0)
///
/// Re-sorts so the caller always receives a properly ordered slice.
fn apply_feedback_scoring(
    mut ranked_jobs: Vec<RankedJob>,
    feedback_by_job_id: &HashMap<String, JobFeedbackState>,
) -> Vec<RankedJob> {
    for ranked in &mut ranked_jobs {
        let Some(feedback) = feedback_by_job_id.get(&ranked.job.job.id) else {
            continue;
        };

        if feedback.company_status == Some(CompanyFeedbackStatus::Whitelist) {
            ranked.fit.score = ranked
                .fit
                .score
                .saturating_add(WHITELIST_SCORE_BONUS)
                .min(100);
            ranked
                .fit
                .reasons
                .push("Company is whitelisted for this profile".to_string());
        }

        if feedback.bad_fit {
            ranked.fit.score = ranked.fit.score.saturating_sub(BAD_FIT_SCORE_PENALTY);
            ranked
                .fit
                .reasons
                .push("Job was previously marked as bad fit".to_string());
        }
    }

    // Re-sort by adjusted score (same tiebreakers as the matching service).
    ranked_jobs.sort_by(|left, right| {
        right
            .fit
            .score
            .cmp(&left.fit.score)
            .then_with(|| right.job.job.last_seen_at.cmp(&left.job.job.last_seen_at))
            .then_with(|| left.job.job.id.cmp(&right.job.job.id))
    });

    ranked_jobs
}

#[cfg(test)]
mod tests {
    use axum::extract::{Query, State};
    use axum::response::IntoResponse;
    use axum::{Json, body};
    use serde_json::{Value, json};

    use crate::api::dto::search::{RunSearchRequest, SearchProfileRequest};
    use crate::api::error::ApiJson;
    use crate::domain::feedback::model::{
        CompanyFeedbackRecord, CompanyFeedbackStatus, JobFeedbackRecord,
    };
    use crate::domain::job::model::{Job, JobLifecycleStage, JobSourceVariant, JobView};
    use crate::domain::profile::model::Profile;
    use crate::domain::search::profile::{TargetRegion, WorkMode};
    use crate::domain::user_event::model::UserEventType;
    use crate::services::applications::{ApplicationsService, ApplicationsServiceStub};
    use crate::services::feedback::{FeedbackService, FeedbackServiceStub};
    use crate::services::jobs::{JobsService, JobsServiceStub};
    use crate::services::profiles::{ProfilesService, ProfilesServiceStub};
    use crate::services::resumes::{ResumesService, ResumesServiceStub};
    use crate::services::user_events::{UserEventsService, UserEventsServiceStub};
    use crate::state::AppState;

    use super::{SearchQuery, run_search, search};

    fn sample_job(id: &str, title: &str) -> crate::domain::job::model::Job {
        crate::domain::job::model::Job {
            id: id.to_string(),
            title: title.to_string(),
            company_name: "NovaLedger".to_string(),
            remote_type: Some("remote".to_string()),
            seniority: Some("senior".to_string()),
            description_text: "Rust and Postgres".to_string(),
            salary_min: None,
            salary_max: None,
            salary_currency: None,
            posted_at: None,
            last_seen_at: "2026-04-11T00:00:00Z".to_string(),
            is_active: true,
        }
    }

    fn sample_profile() -> Profile {
        Profile {
            id: "profile-1".to_string(),
            name: "Jane Doe".to_string(),
            email: "jane@example.com".to_string(),
            location: Some("Kyiv".to_string()),
            raw_text: "Senior backend engineer".to_string(),
            analysis: None,
            salary_min_usd: None,
            salary_max_usd: None,
            preferred_work_mode: None,
            created_at: "2026-04-14T00:00:00Z".to_string(),
            updated_at: "2026-04-14T00:00:00Z".to_string(),
            skills_updated_at: None,
        }
    }

    fn sample_job_view(
        id: &str,
        title: &str,
        description_text: &str,
        remote_type: Option<&str>,
        source: &str,
    ) -> JobView {
        let source_url = match source {
            "djinni" => format!("https://djinni.co/jobs/{id}-sample-role/"),
            "work_ua" => format!("https://www.work.ua/jobs/{id}/"),
            "robota_ua" => format!("https://robota.ua/vacancy/{id}"),
            _ => format!("https://example.com/{id}"),
        };

        JobView {
            job: Job {
                id: id.to_string(),
                title: title.to_string(),
                company_name: "NovaLedger".to_string(),
                remote_type: remote_type.map(str::to_string),
                seniority: Some("senior".to_string()),
                description_text: description_text.to_string(),
                salary_min: None,
                salary_max: None,
                salary_currency: None,
                posted_at: Some("2026-04-12T09:00:00Z".to_string()),
                last_seen_at: "2026-04-14T09:00:00Z".to_string(),
                is_active: true,
            },
            first_seen_at: "2026-04-12T09:00:00Z".to_string(),
            inactivated_at: None,
            reactivated_at: None,
            lifecycle_stage: JobLifecycleStage::Active,
            primary_variant: Some(JobSourceVariant {
                source: source.to_string(),
                source_job_id: format!("{id}-source"),
                source_url,
                raw_payload: None,
                fetched_at: "2026-04-14T09:00:00Z".to_string(),
                last_seen_at: "2026-04-14T09:00:00Z".to_string(),
                is_active: true,
                inactivated_at: None,
            }),
        }
    }

    #[tokio::test]
    async fn rejects_empty_query() {
        let response = search(
            State(AppState::for_services(
                ProfilesService::for_tests(ProfilesServiceStub::default()),
                JobsService::for_tests(JobsServiceStub::default()),
                ApplicationsService::for_tests(ApplicationsServiceStub::default()),
                ResumesService::for_tests(ResumesServiceStub::default()),
            )),
            Query(SearchQuery {
                q: "   ".to_string(),
                limit: None,
                page: None,
            }),
        )
        .await
        .expect_err("empty query should be rejected")
        .into_response();

        assert_eq!(response.status(), axum::http::StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn returns_paginated_results_with_has_more() {
        let state = AppState::for_services(
            ProfilesService::for_tests(ProfilesServiceStub::default()),
            JobsService::for_tests(
                JobsServiceStub::default()
                    .with_job(sample_job("job-1", "Backend Rust Engineer"))
                    .with_job(sample_job("job-2", "Senior Rust Platform Engineer"))
                    .with_job(sample_job("job-3", "Rust Data Engineer")),
            ),
            ApplicationsService::for_tests(ApplicationsServiceStub::default()),
            ResumesService::for_tests(ResumesServiceStub::default()),
        );

        let Json(response) = search(
            State(state),
            Query(SearchQuery {
                q: "rust".to_string(),
                limit: Some(2),
                page: Some(1),
            }),
        )
        .await
        .expect("search should succeed");

        assert_eq!(response.jobs.len(), 2);
        assert_eq!(response.page, 1);
        assert_eq!(response.per_page, 2);
        assert!(response.has_more);
    }

    #[tokio::test]
    async fn returns_second_page_results() {
        let state = AppState::for_services(
            ProfilesService::for_tests(ProfilesServiceStub::default()),
            JobsService::for_tests(
                JobsServiceStub::default()
                    .with_job(sample_job("job-1", "Backend Rust Engineer"))
                    .with_job(sample_job("job-2", "Senior Rust Platform Engineer"))
                    .with_job(sample_job("job-3", "Rust Data Engineer")),
            ),
            ApplicationsService::for_tests(ApplicationsServiceStub::default()),
            ResumesService::for_tests(ResumesServiceStub::default()),
        );

        let response = search(
            State(state),
            Query(SearchQuery {
                q: "rust".to_string(),
                limit: Some(2),
                page: Some(2),
            }),
        )
        .await
        .expect("search should succeed")
        .into_response();

        let body = body::to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("response body should be readable");
        let payload: Value =
            serde_json::from_slice(&body).expect("response body should be valid JSON");

        assert_eq!(payload["jobs"].as_array().map(Vec::len), Some(1));
        assert_eq!(payload["page"].as_i64(), Some(2));
        assert_eq!(payload["has_more"].as_bool(), Some(false));
    }

    #[tokio::test]
    async fn run_search_creates_search_run_event() {
        let state = AppState::for_services(
            ProfilesService::for_tests(
                ProfilesServiceStub::default().with_profile(sample_profile()),
            ),
            JobsService::for_tests(
                JobsServiceStub::default().with_job_view(sample_job_view(
                    "job-1",
                    "Senior Backend Developer",
                    "Rust and Postgres",
                    Some("remote"),
                    "djinni",
                )),
            ),
            ApplicationsService::for_tests(ApplicationsServiceStub::default()),
            ResumesService::for_tests(ResumesServiceStub::default()),
        );

        let Json(_) = run_search(
            State(state.clone()),
            ApiJson(RunSearchRequest {
                profile_id: Some("profile-1".to_string()),
                search_profile: SearchProfileRequest {
                    primary_role: "backend_developer".to_string(),
                    primary_role_confidence: Some(91),
                    target_roles: vec!["backend_developer".to_string()],
                    role_candidates: vec![],
                    seniority: "senior".to_string(),
                    target_regions: vec![TargetRegion::Ua],
                    work_modes: vec![WorkMode::Remote],
                    allowed_sources: vec!["djinni".to_string()],
                    profile_skills: vec!["rust".to_string()],
                    profile_keywords: vec!["postgres".to_string()],
                    search_terms: vec!["rust".to_string(), "backend".to_string()],
                    exclude_terms: vec!["php".to_string()],
                },
                limit: Some(10),
            }),
        )
        .await
        .expect("run_search should succeed");

        let events = state
            .user_events_service
            .list_by_profile("profile-1")
            .await
            .expect("events should be queryable");

        assert!(
            events.iter().any(|event| {
                event.event_type == UserEventType::SearchRun
                    && event.role_family.as_deref() == Some("engineering")
            }),
            "run_search should emit a structured search_run event"
        );
    }

    #[tokio::test]
    async fn run_search_succeeds_when_event_logging_fails_softly() {
        let state = AppState::for_services(
            ProfilesService::for_tests(
                ProfilesServiceStub::default().with_profile(sample_profile()),
            ),
            JobsService::for_tests(
                JobsServiceStub::default().with_job_view(sample_job_view(
                    "job-1",
                    "Senior Backend Developer",
                    "Rust and Postgres",
                    Some("remote"),
                    "djinni",
                )),
            ),
            ApplicationsService::for_tests(ApplicationsServiceStub::default()),
            ResumesService::for_tests(ResumesServiceStub::default()),
        )
        .with_user_events_service(UserEventsService::for_tests(
            UserEventsServiceStub::default().with_database_disabled(),
        ));

        let result = run_search(
            State(state),
            ApiJson(RunSearchRequest {
                profile_id: Some("profile-1".to_string()),
                search_profile: SearchProfileRequest {
                    primary_role: "backend_developer".to_string(),
                    primary_role_confidence: Some(91),
                    target_roles: vec!["backend_developer".to_string()],
                    role_candidates: vec![],
                    seniority: "senior".to_string(),
                    target_regions: vec![TargetRegion::Ua],
                    work_modes: vec![WorkMode::Remote],
                    allowed_sources: vec!["djinni".to_string()],
                    profile_skills: vec!["rust".to_string()],
                    profile_keywords: vec!["postgres".to_string()],
                    search_terms: vec!["rust".to_string(), "backend".to_string()],
                    exclude_terms: vec![],
                },
                limit: Some(10),
            }),
        )
        .await;

        assert!(result.is_ok(), "search should not fail when event logging is unavailable");
    }

    #[tokio::test]
    async fn run_search_returns_ranked_jobs_with_fit_reasons() {
        let state = AppState::for_services(
            ProfilesService::for_tests(
                ProfilesServiceStub::default().with_profile(sample_profile()),
            ),
            JobsService::for_tests(
                JobsServiceStub::default()
                    .with_job_view(sample_job_view(
                        "job-1",
                        "Senior Backend Developer",
                        "Remote EU role working with Rust and Postgres",
                        Some("remote"),
                        "djinni",
                    ))
                    .with_job_view(sample_job_view(
                        "job-2",
                        "Project Manager",
                        "Hybrid delivery coordination role in Warsaw",
                        Some("hybrid"),
                        "work_ua",
                    )),
            ),
            ApplicationsService::for_tests(ApplicationsServiceStub::default()),
            ResumesService::for_tests(ResumesServiceStub::default()),
        );

        let response = run_search(
            State(state),
            ApiJson(RunSearchRequest {
                profile_id: None,
                search_profile: SearchProfileRequest {
                    primary_role: "backend_developer".to_string(),
                    primary_role_confidence: Some(95),
                    target_roles: vec!["devops_engineer".to_string()],
                    role_candidates: vec![
                        crate::api::dto::search::SearchRoleCandidateRequest {
                            role: "backend_developer".to_string(),
                            confidence: 95,
                        },
                        crate::api::dto::search::SearchRoleCandidateRequest {
                            role: "devops_engineer".to_string(),
                            confidence: 66,
                        },
                    ],
                    seniority: "senior".to_string(),
                    target_regions: vec![TargetRegion::EuRemote],
                    work_modes: vec![WorkMode::Remote],
                    allowed_sources: vec!["djinni".to_string()],
                    profile_skills: vec!["rust".to_string(), "postgres".to_string()],
                    profile_keywords: vec!["backend".to_string()],
                    search_terms: vec!["rust".to_string(), "postgres".to_string()],
                    exclude_terms: vec![],
                },
                limit: Some(10),
            }),
        )
        .await
        .expect("run search should succeed")
        .into_response();

        let body = body::to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("response body should be readable");
        let payload: Value =
            serde_json::from_slice(&body).expect("response body should be valid JSON");

        assert_eq!(payload["meta"]["filtered_out_by_source"], json!(1));
        assert_eq!(payload["results"].as_array().map(Vec::len), Some(1));
        assert_eq!(payload["results"][0]["job"]["id"], json!("job-1"));
        assert_eq!(
            payload["results"][0]["job"]["presentation"]["source_label"],
            json!("Djinni")
        );
        assert_eq!(
            payload["results"][0]["job"]["presentation"]["outbound_url"],
            json!("https://djinni.co/jobs/job-1-sample-role")
        );
        assert!(
            payload["results"][0]["fit"]["reasons"]
                .as_array()
                .expect("reasons should be an array")
                .iter()
                .any(|reason| reason
                    .as_str()
                    .is_some_and(|reason| reason.contains("Matched target roles")))
        );
    }

    #[tokio::test]
    async fn hidden_jobs_are_excluded_from_ranked_results() {
        let state = AppState::for_services(
            ProfilesService::for_tests(
                ProfilesServiceStub::default().with_profile(sample_profile()),
            ),
            JobsService::for_tests(
                JobsServiceStub::default()
                    .with_job_view(sample_job_view(
                        "job-1",
                        "Senior Backend Developer",
                        "Remote EU role working with Rust and Postgres",
                        Some("remote"),
                        "djinni",
                    ))
                    .with_job_view(sample_job_view(
                        "job-2",
                        "Senior Backend Engineer",
                        "Remote role working with Rust and distributed systems",
                        Some("remote"),
                        "djinni",
                    )),
            ),
            ApplicationsService::for_tests(ApplicationsServiceStub::default()),
            ResumesService::for_tests(ResumesServiceStub::default()),
        )
        .with_feedback_service(FeedbackService::for_tests(
            FeedbackServiceStub::default().with_job_feedback(JobFeedbackRecord {
                profile_id: "profile-1".to_string(),
                job_id: "job-1".to_string(),
                saved: false,
                hidden: true,
                bad_fit: false,
                created_at: "2026-04-14T00:00:00Z".to_string(),
                updated_at: "2026-04-14T00:00:00Z".to_string(),
            }),
        ));

        let response = run_search(
            State(state),
            ApiJson(RunSearchRequest {
                profile_id: Some("profile-1".to_string()),
                search_profile: SearchProfileRequest {
                    primary_role: "backend_developer".to_string(),
                    primary_role_confidence: Some(95),
                    target_roles: vec![],
                    role_candidates: vec![],
                    seniority: "senior".to_string(),
                    target_regions: vec![TargetRegion::EuRemote],
                    work_modes: vec![WorkMode::Remote],
                    allowed_sources: vec!["djinni".to_string()],
                    profile_skills: vec!["rust".to_string()],
                    profile_keywords: vec!["backend".to_string()],
                    search_terms: vec!["rust".to_string()],
                    exclude_terms: vec![],
                },
                limit: Some(10),
            }),
        )
        .await
        .expect("run search should succeed")
        .into_response();

        let body = body::to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("response body should be readable");
        let payload: Value =
            serde_json::from_slice(&body).expect("response body should be valid JSON");

        assert_eq!(payload["meta"]["filtered_out_hidden"], json!(1));
        assert_eq!(payload["results"].as_array().map(Vec::len), Some(1));
        assert_eq!(payload["results"][0]["job"]["id"], json!("job-2"));
    }

    #[tokio::test]
    async fn blacklisted_companies_are_excluded_from_ranked_results() {
        let state = AppState::for_services(
            ProfilesService::for_tests(
                ProfilesServiceStub::default().with_profile(sample_profile()),
            ),
            JobsService::for_tests(
                JobsServiceStub::default()
                    .with_job_view(sample_job_view(
                        "job-1",
                        "Senior Backend Developer",
                        "Remote EU role working with Rust and Postgres",
                        Some("remote"),
                        "djinni",
                    ))
                    .with_job_view(sample_job_view(
                        "job-2",
                        "Senior Backend Engineer",
                        "Remote role working with Rust and distributed systems",
                        Some("remote"),
                        "djinni",
                    )),
            ),
            ApplicationsService::for_tests(ApplicationsServiceStub::default()),
            ResumesService::for_tests(ResumesServiceStub::default()),
        )
        .with_feedback_service(FeedbackService::for_tests(
            FeedbackServiceStub::default().with_company_feedback(CompanyFeedbackRecord {
                profile_id: "profile-1".to_string(),
                company_name: "NovaLedger".to_string(),
                normalized_company_name: "novaledger".to_string(),
                status: CompanyFeedbackStatus::Blacklist,
                created_at: "2026-04-14T00:00:00Z".to_string(),
                updated_at: "2026-04-14T00:00:00Z".to_string(),
            }),
        ));

        let response = run_search(
            State(state),
            ApiJson(RunSearchRequest {
                profile_id: Some("profile-1".to_string()),
                search_profile: SearchProfileRequest {
                    primary_role: "backend_developer".to_string(),
                    primary_role_confidence: Some(95),
                    target_roles: vec![],
                    role_candidates: vec![],
                    seniority: "senior".to_string(),
                    target_regions: vec![TargetRegion::EuRemote],
                    work_modes: vec![WorkMode::Remote],
                    allowed_sources: vec!["djinni".to_string()],
                    profile_skills: vec!["rust".to_string()],
                    profile_keywords: vec!["backend".to_string()],
                    search_terms: vec!["rust".to_string()],
                    exclude_terms: vec![],
                },
                limit: Some(10),
            }),
        )
        .await
        .expect("run search should succeed")
        .into_response();

        let body = body::to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("response body should be readable");
        let payload: Value =
            serde_json::from_slice(&body).expect("response body should be valid JSON");

        assert_eq!(payload["meta"]["filtered_out_company_blacklist"], json!(2));
        assert_eq!(payload["results"].as_array().map(Vec::len), Some(0));
    }

    #[tokio::test]
    async fn whitelisted_company_raises_score_and_adds_reason() {
        // job-1 is from NovaLedger (whitelisted), job-2 is from AcmeCorp (no feedback).
        // Both have similar content, so the whitelist bonus should push job-1 above job-2.
        let state = AppState::for_services(
            ProfilesService::for_tests(
                ProfilesServiceStub::default().with_profile(sample_profile()),
            ),
            JobsService::for_tests(
                JobsServiceStub::default()
                    .with_job_view(sample_job_view(
                        "job-1",
                        "Senior Backend Developer",
                        "Remote EU role working with Rust and Postgres",
                        Some("remote"),
                        "djinni",
                    ))
                    .with_job_view({
                        let mut jv = sample_job_view(
                            "job-2",
                            "Senior Backend Engineer",
                            "Remote role working with Rust and distributed systems",
                            Some("remote"),
                            "djinni",
                        );
                        jv.job.company_name = "AcmeCorp".to_string();
                        jv
                    }),
            ),
            ApplicationsService::for_tests(ApplicationsServiceStub::default()),
            ResumesService::for_tests(ResumesServiceStub::default()),
        )
        .with_feedback_service(FeedbackService::for_tests(
            FeedbackServiceStub::default().with_company_feedback(CompanyFeedbackRecord {
                profile_id: "profile-1".to_string(),
                company_name: "NovaLedger".to_string(),
                normalized_company_name: "novaledger".to_string(),
                status: CompanyFeedbackStatus::Whitelist,
                created_at: "2026-04-14T00:00:00Z".to_string(),
                updated_at: "2026-04-14T00:00:00Z".to_string(),
            }),
        ));

        let response = run_search(
            State(state),
            ApiJson(RunSearchRequest {
                profile_id: Some("profile-1".to_string()),
                search_profile: SearchProfileRequest {
                    primary_role: "backend_developer".to_string(),
                    primary_role_confidence: Some(95),
                    target_roles: vec![],
                    role_candidates: vec![],
                    seniority: "senior".to_string(),
                    target_regions: vec![TargetRegion::EuRemote],
                    work_modes: vec![WorkMode::Remote],
                    allowed_sources: vec!["djinni".to_string()],
                    profile_skills: vec!["rust".to_string()],
                    profile_keywords: vec!["backend".to_string()],
                    search_terms: vec!["rust".to_string()],
                    exclude_terms: vec![],
                },
                limit: Some(10),
            }),
        )
        .await
        .expect("run search should succeed")
        .into_response();

        let body = body::to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("response body should be readable");
        let payload: Value =
            serde_json::from_slice(&body).expect("response body should be valid JSON");

        // job-1 (whitelisted company) must appear first.
        assert_eq!(payload["results"].as_array().map(Vec::len), Some(2));
        assert_eq!(payload["results"][0]["job"]["id"], json!("job-1"));
        assert!(
            payload["results"][0]["fit"]["reasons"]
                .as_array()
                .expect("reasons should be an array")
                .iter()
                .any(|r| r
                    .as_str()
                    .is_some_and(|s| s.contains("whitelisted"))),
            "whitelist reason should appear in fit reasons"
        );

        // job-1 score must be higher than job-2 score due to whitelist bonus.
        let score_1 = payload["results"][0]["fit"]["score"]
            .as_u64()
            .expect("score should be a number");
        let score_2 = payload["results"][1]["fit"]["score"]
            .as_u64()
            .expect("score should be a number");
        assert!(score_1 > score_2, "whitelist bonus must raise job-1 score above job-2");
    }

    #[tokio::test]
    async fn bad_fit_job_gets_score_penalty_and_reason() {
        // Use identical content for both jobs so they start with the same score.
        // The bad_fit penalty on job-1 should push job-2 above it.
        let shared_desc = "Remote EU role working with Rust and Postgres backend systems";
        let state = AppState::for_services(
            ProfilesService::for_tests(
                ProfilesServiceStub::default().with_profile(sample_profile()),
            ),
            JobsService::for_tests(
                JobsServiceStub::default()
                    .with_job_view(sample_job_view(
                        "job-1",
                        "Senior Backend Developer",
                        shared_desc,
                        Some("remote"),
                        "djinni",
                    ))
                    .with_job_view(sample_job_view(
                        "job-2",
                        "Senior Backend Developer",
                        shared_desc,
                        Some("remote"),
                        "djinni",
                    )),
            ),
            ApplicationsService::for_tests(ApplicationsServiceStub::default()),
            ResumesService::for_tests(ResumesServiceStub::default()),
        )
        .with_feedback_service(FeedbackService::for_tests(
            FeedbackServiceStub::default().with_job_feedback(JobFeedbackRecord {
                profile_id: "profile-1".to_string(),
                job_id: "job-1".to_string(),
                saved: false,
                hidden: false,
                bad_fit: true,
                created_at: "2026-04-14T00:00:00Z".to_string(),
                updated_at: "2026-04-14T00:00:00Z".to_string(),
            }),
        ));

        let response = run_search(
            State(state),
            ApiJson(RunSearchRequest {
                profile_id: Some("profile-1".to_string()),
                search_profile: SearchProfileRequest {
                    primary_role: "backend_developer".to_string(),
                    primary_role_confidence: Some(95),
                    target_roles: vec![],
                    role_candidates: vec![],
                    seniority: "senior".to_string(),
                    target_regions: vec![TargetRegion::EuRemote],
                    work_modes: vec![WorkMode::Remote],
                    allowed_sources: vec!["djinni".to_string()],
                    profile_skills: vec!["rust".to_string()],
                    profile_keywords: vec!["backend".to_string()],
                    search_terms: vec!["rust".to_string()],
                    exclude_terms: vec![],
                },
                limit: Some(10),
            }),
        )
        .await
        .expect("run search should succeed")
        .into_response();

        let body = body::to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("response body should be readable");
        let payload: Value =
            serde_json::from_slice(&body).expect("response body should be valid JSON");

        assert_eq!(payload["results"].as_array().map(Vec::len), Some(2));

        // job-2 (no bad fit) must appear first.
        assert_eq!(payload["results"][0]["job"]["id"], json!("job-2"));

        // job-1 reasons must mention bad fit penalty.
        let job1_pos = payload["results"]
            .as_array()
            .unwrap()
            .iter()
            .position(|r| r["job"]["id"] == json!("job-1"))
            .expect("job-1 must appear in results");
        assert!(
            payload["results"][job1_pos]["fit"]["reasons"]
                .as_array()
                .expect("reasons should be an array")
                .iter()
                .any(|r| r.as_str().is_some_and(|s| s.contains("bad fit"))),
            "bad fit reason should appear in fit reasons"
        );
    }

    /// Whitelist bonus must be applied before truncation so a job ranked just
    /// outside the limit by pure score can be promoted into the result set.
    ///
    /// Setup: limit=1, job-1 has identical content to job-2 but belongs to a
    /// whitelisted company.  Pure scoring gives job-2 an earlier id tiebreak
    /// edge (job-1 < job-2 but that sorts job-1 first by id; both have the
    /// same score so the id tiebreak puts job-1 first without feedback).
    /// We need a scenario where the feedback bonus actually matters for ordering.
    ///
    /// Simpler: two identical jobs, limit=1.  job-2 is whitelisted.
    /// Without the fix, job-1 would be the sole result (id tiebreak).
    /// After the fix, job-2 should win because its whitelist bonus is applied
    /// before truncation.
    #[tokio::test]
    async fn whitelist_bonus_promotes_job_before_truncation() {
        let shared_desc = "Remote EU role working with Rust and Postgres backend systems";
        let state = AppState::for_services(
            ProfilesService::for_tests(
                ProfilesServiceStub::default().with_profile(sample_profile()),
            ),
            JobsService::for_tests(
                JobsServiceStub::default()
                    .with_job_view(sample_job_view(
                        "job-1",
                        "Senior Backend Developer",
                        shared_desc,
                        Some("remote"),
                        "djinni",
                    ))
                    .with_job_view({
                        let mut jv = sample_job_view(
                            "job-2",
                            "Senior Backend Developer",
                            shared_desc,
                            Some("remote"),
                            "djinni",
                        );
                        jv.job.company_name = "FavoriteCorp".to_string();
                        jv
                    }),
            ),
            ApplicationsService::for_tests(ApplicationsServiceStub::default()),
            ResumesService::for_tests(ResumesServiceStub::default()),
        )
        .with_feedback_service(FeedbackService::for_tests(
            FeedbackServiceStub::default().with_company_feedback(CompanyFeedbackRecord {
                profile_id: "profile-1".to_string(),
                company_name: "FavoriteCorp".to_string(),
                normalized_company_name: "favoritecorp".to_string(),
                status: CompanyFeedbackStatus::Whitelist,
                created_at: "2026-04-14T00:00:00Z".to_string(),
                updated_at: "2026-04-14T00:00:00Z".to_string(),
            }),
        ));

        let response = run_search(
            State(state),
            ApiJson(RunSearchRequest {
                profile_id: Some("profile-1".to_string()),
                search_profile: SearchProfileRequest {
                    primary_role: "backend_developer".to_string(),
                    primary_role_confidence: Some(95),
                    target_roles: vec![],
                    role_candidates: vec![],
                    seniority: "senior".to_string(),
                    target_regions: vec![TargetRegion::EuRemote],
                    work_modes: vec![WorkMode::Remote],
                    allowed_sources: vec!["djinni".to_string()],
                    profile_skills: vec!["rust".to_string()],
                    profile_keywords: vec!["backend".to_string()],
                    search_terms: vec!["rust".to_string()],
                    exclude_terms: vec![],
                },
                limit: Some(1),
            }),
        )
        .await
        .expect("run search should succeed")
        .into_response();

        let body = body::to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("response body should be readable");
        let payload: Value =
            serde_json::from_slice(&body).expect("response body should be valid JSON");

        // limit=1: only one job returned; it must be the whitelisted one.
        assert_eq!(payload["results"].as_array().map(Vec::len), Some(1));
        assert_eq!(
            payload["results"][0]["job"]["id"],
            json!("job-2"),
            "whitelisted job-2 must be promoted before truncation"
        );
    }

    /// Bad-fit penalty must push a job out of the result set when limit is tight.
    ///
    /// Setup: limit=1, two identical jobs, job-1 is marked bad fit.
    /// Before the fix, job-1 could still win the id tiebreak and appear in results.
    /// After the fix, the -30 penalty is applied before truncation, so job-2 wins.
    #[tokio::test]
    async fn bad_fit_penalty_demotes_job_before_truncation() {
        let shared_desc = "Remote EU role working with Rust and Postgres backend systems";
        let state = AppState::for_services(
            ProfilesService::for_tests(
                ProfilesServiceStub::default().with_profile(sample_profile()),
            ),
            JobsService::for_tests(
                JobsServiceStub::default()
                    .with_job_view(sample_job_view(
                        "job-1",
                        "Senior Backend Developer",
                        shared_desc,
                        Some("remote"),
                        "djinni",
                    ))
                    .with_job_view(sample_job_view(
                        "job-2",
                        "Senior Backend Developer",
                        shared_desc,
                        Some("remote"),
                        "djinni",
                    )),
            ),
            ApplicationsService::for_tests(ApplicationsServiceStub::default()),
            ResumesService::for_tests(ResumesServiceStub::default()),
        )
        .with_feedback_service(FeedbackService::for_tests(
            FeedbackServiceStub::default().with_job_feedback(JobFeedbackRecord {
                profile_id: "profile-1".to_string(),
                job_id: "job-1".to_string(),
                saved: false,
                hidden: false,
                bad_fit: true,
                created_at: "2026-04-14T00:00:00Z".to_string(),
                updated_at: "2026-04-14T00:00:00Z".to_string(),
            }),
        ));

        let response = run_search(
            State(state),
            ApiJson(RunSearchRequest {
                profile_id: Some("profile-1".to_string()),
                search_profile: SearchProfileRequest {
                    primary_role: "backend_developer".to_string(),
                    primary_role_confidence: Some(95),
                    target_roles: vec![],
                    role_candidates: vec![],
                    seniority: "senior".to_string(),
                    target_regions: vec![TargetRegion::EuRemote],
                    work_modes: vec![WorkMode::Remote],
                    allowed_sources: vec!["djinni".to_string()],
                    profile_skills: vec!["rust".to_string()],
                    profile_keywords: vec!["backend".to_string()],
                    search_terms: vec!["rust".to_string()],
                    exclude_terms: vec![],
                },
                limit: Some(1),
            }),
        )
        .await
        .expect("run search should succeed")
        .into_response();

        let body = body::to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("response body should be readable");
        let payload: Value =
            serde_json::from_slice(&body).expect("response body should be valid JSON");

        // limit=1: only one job returned; bad-fit job-1 must be excluded.
        assert_eq!(payload["results"].as_array().map(Vec::len), Some(1));
        assert_eq!(
            payload["results"][0]["job"]["id"],
            json!("job-2"),
            "bad-fit job-1 must be demoted out of results before truncation"
        );
    }
}
