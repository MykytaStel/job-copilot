use std::collections::HashMap;

use axum::extract::{Path, Query, State};
use serde::Deserialize;
use tracing::warn;

use crate::api::dto::jobs::{JobResponse, MlJobLifecycleResponse, RecentJobsResponse};
use crate::api::dto::ranking::FitScoreResponse;
use crate::api::dto::search::JobFitResponse;
use crate::api::error::{ApiError, ApiJson};
use crate::api::routes::feedback::ensure_profile_exists;
use crate::api::routes::search::{
    apply_behavior_scoring, apply_feedback_scoring, apply_learned_reranking,
    apply_trained_reranking, load_learning_aggregates, score_by_job_id,
};
use crate::domain::feedback::model::{CompanyFeedbackRecord, JobFeedbackRecord, JobFeedbackState};
use crate::domain::search::profile::SearchPreferences;
use crate::domain::source::SourceId;
use crate::services::feedback::FeedbackService;
use crate::services::matching::{RankedJob, summarize_match_quality};
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct RecentJobsQuery {
    pub limit: Option<i64>,
    /// Filter by lifecycle stage: "active" | "inactive" | "reactivated"
    pub lifecycle: Option<String>,
    /// Filter by source name: "djinni" | "work_ua" | "robota_ua"
    pub source: Option<SourceId>,
    pub profile_id: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
pub struct JobContextQuery {
    pub profile_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct BulkProfileJobMatchRequest {
    pub job_ids: Vec<String>,
}

#[derive(Debug)]
struct BulkProfileJobMatchInput {
    job_ids: Vec<String>,
}

#[derive(Debug, serde::Serialize)]
pub struct BulkProfileJobMatchResponse {
    pub profile_id: String,
    pub results: Vec<JobFitResponse>,
    pub meta: BulkProfileJobMatchMeta,
}

#[derive(Debug, serde::Serialize)]
pub struct BulkProfileJobMatchMeta {
    pub returned_jobs: usize,
    pub low_evidence_jobs: usize,
    pub weak_description_jobs: usize,
    pub role_mismatch_jobs: usize,
    pub seniority_mismatch_jobs: usize,
    pub source_mismatch_jobs: usize,
    pub top_missing_signals: Vec<String>,
}

impl BulkProfileJobMatchRequest {
    fn validate(self) -> Result<BulkProfileJobMatchInput, ApiError> {
        let mut job_ids = Vec::new();

        for job_id in self.job_ids {
            let normalized = job_id.trim();
            if normalized.is_empty() || job_ids.iter().any(|existing| existing == normalized) {
                continue;
            }

            job_ids.push(normalized.to_string());
        }

        if job_ids.is_empty() {
            return Err(ApiError::bad_request(
                "invalid_job_ids",
                "job_ids must contain at least one non-empty id",
            ));
        }

        if job_ids.len() > 200 {
            return Err(ApiError::bad_request(
                "invalid_job_ids",
                "job_ids must contain at most 200 ids",
            ));
        }

        Ok(BulkProfileJobMatchInput { job_ids })
    }
}

pub async fn get_job_by_id(
    State(state): State<AppState>,
    Path(job_id): Path<String>,
    Query(query): Query<JobContextQuery>,
) -> Result<axum::Json<JobResponse>, ApiError> {
    let Some(job) = state
        .jobs_service
        .get_view_by_id(&job_id)
        .await
        .map_err(|error| ApiError::from_repository(error, "jobs_query_failed"))?
    else {
        return Err(ApiError::not_found(
            "job_not_found",
            format!("Job '{job_id}' was not found"),
        ));
    };

    let feedback = load_feedback_state(
        &state,
        query.profile_id.as_deref(),
        std::slice::from_ref(&job),
    )
    .await?
    .into_iter()
    .next()
    .unwrap_or_default();

    Ok(axum::Json(JobResponse::from_view_with_feedback(
        job, feedback,
    )))
}

pub async fn get_ml_job_lifecycle(
    State(state): State<AppState>,
    Path(job_id): Path<String>,
) -> Result<axum::Json<MlJobLifecycleResponse>, ApiError> {
    let Some(job) = state
        .jobs_service
        .get_view_by_id(&job_id)
        .await
        .map_err(|error| ApiError::from_repository(error, "jobs_query_failed"))?
    else {
        return Err(ApiError::not_found(
            "job_not_found",
            format!("Job '{job_id}' was not found"),
        ));
    };

    Ok(axum::Json(MlJobLifecycleResponse::from(job)))
}

pub async fn get_recent_jobs(
    State(state): State<AppState>,
    Query(query): Query<RecentJobsQuery>,
) -> Result<axum::Json<RecentJobsResponse>, ApiError> {
    let limit = query.limit.unwrap_or(50);

    if !(1..=200).contains(&limit) {
        return Err(ApiError::invalid_limit(limit));
    }

    let lifecycle = query.lifecycle.as_deref();
    let source = query.source.map(SourceId::canonical_key);
    let profile_id = query.profile_id.as_deref();

    let jobs = state
        .jobs_service
        .list_filtered_views(limit, lifecycle, source)
        .await
        .map_err(|error| ApiError::from_repository(error, "jobs_query_failed"))?;
    let feedback_states = load_feedback_state(&state, profile_id, &jobs).await?;
    let jobs = jobs
        .into_iter()
        .zip(feedback_states.into_iter())
        .filter(|(_, feedback)| !feedback.hidden)
        .map(|(job, feedback)| JobResponse::from_view_with_feedback(job, feedback))
        .collect();
    let summary = state
        .jobs_service
        .feed_summary()
        .await
        .map_err(|error| ApiError::from_repository(error, "jobs_query_failed"))?;

    Ok(axum::Json(RecentJobsResponse {
        jobs,
        summary: summary.into(),
    }))
}

pub async fn get_profile_job_match(
    State(state): State<AppState>,
    Path((profile_id, job_id)): Path<(String, String)>,
) -> Result<axum::Json<JobFitResponse>, ApiError> {
    let ranked_jobs = load_profile_ranked_jobs(&state, &profile_id, &[job_id.clone()]).await?;
    let Some(ranked) = ranked_jobs
        .into_iter()
        .find(|ranked| ranked.job.job.id == job_id)
    else {
        return Err(ApiError::not_found(
            "job_not_found",
            format!("Job '{job_id}' was not found"),
        ));
    };

    Ok(axum::Json(JobFitResponse::from(ranked.fit)))
}

pub async fn bulk_profile_job_match(
    State(state): State<AppState>,
    Path(profile_id): Path<String>,
    ApiJson(payload): ApiJson<BulkProfileJobMatchRequest>,
) -> Result<axum::Json<BulkProfileJobMatchResponse>, ApiError> {
    let input = payload.validate()?;
    let ranked_jobs = load_profile_ranked_jobs(&state, &profile_id, &input.job_ids).await?;
    let quality = summarize_match_quality(&ranked_jobs);
    let results = ranked_jobs
        .into_iter()
        .map(|ranked| JobFitResponse::from(ranked.fit))
        .collect::<Vec<_>>();

    Ok(axum::Json(BulkProfileJobMatchResponse {
        profile_id,
        meta: BulkProfileJobMatchMeta {
            returned_jobs: results.len(),
            low_evidence_jobs: quality.low_evidence_jobs,
            weak_description_jobs: quality.weak_description_jobs,
            role_mismatch_jobs: quality.role_mismatch_jobs,
            seniority_mismatch_jobs: quality.seniority_mismatch_jobs,
            source_mismatch_jobs: quality.source_mismatch_jobs,
            top_missing_signals: quality.top_missing_signals,
        },
        results,
    }))
}

async fn load_profile_ranked_jobs(
    state: &AppState,
    profile_id: &str,
    job_ids: &[String],
) -> Result<Vec<RankedJob>, ApiError> {
    ensure_profile_exists(state, profile_id).await?;

    let profile = state
        .profiles_service
        .get_by_id(profile_id)
        .await
        .map_err(|error| ApiError::from_repository(error, "profiles_query_failed"))?
        .expect("profile existence already verified above");
    let analyzed_profile = state.profile_analysis_service.analyze(&profile.raw_text);
    let search_profile = state
        .search_profile_service
        .build(&analyzed_profile, &SearchPreferences::default());

    let mut jobs = Vec::new();
    for job_id in job_ids {
        if let Some(job) = state
            .jobs_service
            .get_view_by_id(job_id)
            .await
            .map_err(|error| ApiError::from_repository(error, "jobs_query_failed"))?
        {
            jobs.push(job);
        }
    }

    let feedback_states = load_feedback_state(state, Some(profile_id), &jobs).await?;
    let mut feedback_by_job_id = HashMap::new();
    let ranked_candidates = jobs
        .into_iter()
        .zip(feedback_states)
        .map(|(job, feedback)| {
            feedback_by_job_id.insert(job.job.id.clone(), feedback);
            job
        })
        .collect::<Vec<_>>();

    let result = state
        .search_matching_service
        .run(&search_profile, ranked_candidates);
    let deterministic_score_by_job_id = result
        .ranked_jobs
        .iter()
        .map(|ranked| (ranked.job.job.id.clone(), ranked.fit.score))
        .collect::<HashMap<_, _>>();

    let mut adjusted_jobs = apply_feedback_scoring(result.ranked_jobs, &feedback_by_job_id);
    let learning_aggregates = load_learning_aggregates(state, Some(profile_id)).await;

    if let Some(aggregates) = learning_aggregates.as_ref() {
        adjusted_jobs = apply_behavior_scoring(state, adjusted_jobs, &aggregates.behavior);
    }

    let behavior_score_by_job_id = score_by_job_id(&adjusted_jobs);

    if state.learned_reranker_enabled {
        if let Some(aggregates) = learning_aggregates.as_ref() {
            let (reranked_jobs, _adjusted_count) = apply_learned_reranking(
                state,
                adjusted_jobs,
                &aggregates.behavior,
                &aggregates.funnel,
                &feedback_by_job_id,
                &deterministic_score_by_job_id,
            );
            adjusted_jobs = reranked_jobs;
        }
    }

    if state.trained_reranker_enabled {
        let (reranked_jobs, _adjusted_count) = apply_trained_reranking(
            state,
            adjusted_jobs,
            &deterministic_score_by_job_id,
            &behavior_score_by_job_id,
        );
        adjusted_jobs = reranked_jobs;
    }

    Ok(adjusted_jobs)
}

/// Read the persisted fit score for a job (previously computed via GET /fit or POST /match).
pub async fn get_job_match(
    State(state): State<AppState>,
    Path(job_id): Path<String>,
) -> Result<axum::Json<FitScoreResponse>, ApiError> {
    let Some(resume) = state
        .resumes_service
        .get_active()
        .await
        .map_err(|error| ApiError::from_repository(error, "match_query_failed"))?
    else {
        return Err(ApiError::not_found(
            "active_resume_not_found",
            "No active resume was found",
        ));
    };

    let Some(score) = state
        .fit_scores_repository
        .get_for_job_and_resume(&job_id, &resume.id)
        .await
        .map_err(|error| ApiError::from_repository(error, "match_query_failed"))?
    else {
        return Err(ApiError::not_found(
            "match_result_not_found",
            format!("No fit score exists for job '{job_id}' — call GET /fit first"),
        ));
    };

    Ok(axum::Json(FitScoreResponse::from(score)))
}

pub(crate) async fn load_feedback_state(
    state: &AppState,
    profile_id: Option<&str>,
    jobs: &[crate::domain::job::model::JobView],
) -> Result<Vec<JobFeedbackState>, ApiError> {
    let Some(profile_id) = profile_id.filter(|value| !value.trim().is_empty()) else {
        return Ok(vec![JobFeedbackState::default(); jobs.len()]);
    };

    ensure_profile_exists(state, profile_id).await?;

    let job_ids = jobs
        .iter()
        .map(|job| job.job.id.clone())
        .collect::<Vec<_>>();
    let normalized_company_names = jobs
        .iter()
        .map(|job| FeedbackService::normalize_company_name(&job.job.company_name))
        .collect::<Vec<_>>();

    let job_feedback = state
        .feedback_service
        .list_job_feedback_for_jobs(profile_id, &job_ids)
        .await
        .map_err(|error| ApiError::from_repository(error, "feedback_query_failed"))?;
    let company_feedback = state
        .feedback_service
        .list_company_feedback_for_names(profile_id, &normalized_company_names)
        .await
        .map_err(|error| ApiError::from_repository(error, "feedback_query_failed"))?;

    Ok(build_feedback_states(jobs, job_feedback, company_feedback))
}

pub(crate) fn build_feedback_states(
    jobs: &[crate::domain::job::model::JobView],
    job_feedback: Vec<JobFeedbackRecord>,
    company_feedback: Vec<CompanyFeedbackRecord>,
) -> Vec<JobFeedbackState> {
    let job_feedback_by_job_id = job_feedback
        .into_iter()
        .map(|record| (record.job_id.clone(), record))
        .collect::<std::collections::HashMap<_, _>>();
    let company_feedback_by_name = company_feedback
        .into_iter()
        .map(|record| (record.normalized_company_name.clone(), record))
        .collect::<std::collections::HashMap<_, _>>();

    jobs.iter()
        .map(|job| {
            let normalized_company_name =
                FeedbackService::normalize_company_name(&job.job.company_name);
            JobFeedbackState::from_sources(
                job_feedback_by_job_id.get(&job.job.id),
                company_feedback_by_name.get(&normalized_company_name),
            )
        })
        .collect()
}

/// Return a fit score for a job against the active resume.
///
/// Cache-first: returns the persisted score when one exists so repeat calls are
/// instant.  On a cache miss the score is computed locally (no API call), then
/// persisted for subsequent requests.
pub async fn get_job_fit(
    State(state): State<AppState>,
    Path(job_id): Path<String>,
) -> Result<axum::Json<FitScoreResponse>, ApiError> {
    let Some(resume) = state
        .resumes_service
        .get_active()
        .await
        .map_err(|error| ApiError::from_repository(error, "fit_query_failed"))?
    else {
        return Err(ApiError::not_found(
            "active_resume_not_found",
            "No active resume was found",
        ));
    };

    // Return persisted score when available — the common path after the first visit.
    if let Ok(Some(cached)) = state
        .fit_scores_repository
        .get_for_job_and_resume(&job_id, &resume.id)
        .await
    {
        return Ok(axum::Json(FitScoreResponse::from(cached)));
    }

    // Cache miss: fetch the job, compute, persist, then return.
    let Some(job) = state
        .jobs_service
        .get_by_id(&job_id)
        .await
        .map_err(|error| ApiError::from_repository(error, "fit_query_failed"))?
    else {
        return Err(ApiError::not_found(
            "job_not_found",
            format!("Job '{job_id}' was not found"),
        ));
    };

    let candidate = state.profile_analysis_service.analyze(&resume.raw_text);
    // Best-effort: load stored profile for salary/work-mode prefs.
    // If absent, those components default to neutral (0.5).
    let profile = state.profiles_service.get_latest().await.ok().flatten();

    let score = state
        .ranking_service
        .compute(&candidate, &job, profile.as_ref());

    if let Err(error) = state.fit_scores_repository.upsert(&score, &resume.id).await {
        warn!(job_id = %job_id, resume_id = %resume.id, error = %error, "failed to persist fit score");
    }

    Ok(axum::Json(FitScoreResponse::from(score)))
}

/// Force-recompute and persist a fit score for a job (same as GET /fit but via POST).
pub async fn score_job_match(
    State(state): State<AppState>,
    Path(job_id): Path<String>,
) -> Result<axum::Json<FitScoreResponse>, ApiError> {
    let Some(job) = state
        .jobs_service
        .get_by_id(&job_id)
        .await
        .map_err(|error| ApiError::from_repository(error, "match_query_failed"))?
    else {
        return Err(ApiError::not_found(
            "job_not_found",
            format!("Job '{job_id}' was not found"),
        ));
    };

    let Some(resume) = state
        .resumes_service
        .get_active()
        .await
        .map_err(|error| ApiError::from_repository(error, "match_query_failed"))?
    else {
        return Err(ApiError::not_found(
            "active_resume_not_found",
            "No active resume was found",
        ));
    };

    let candidate = state.profile_analysis_service.analyze(&resume.raw_text);
    let profile = state.profiles_service.get_latest().await.ok().flatten();
    let score = state
        .ranking_service
        .compute(&candidate, &job, profile.as_ref());

    if let Err(error) = state.fit_scores_repository.upsert(&score, &resume.id).await {
        warn!(job_id = %job_id, resume_id = %resume.id, error = %error, "failed to persist fit score");
    }

    Ok(axum::Json(FitScoreResponse::from(score)))
}

#[cfg(test)]
mod tests {
    use axum::Json;
    use axum::extract::{Path, Query, State};
    use axum::http::Uri;
    use axum::response::IntoResponse;
    use axum::{body, http::StatusCode};
    use serde_json::{Value, json};

    use crate::api::error::ApiJson;
    use crate::domain::job::model::{
        Job, JobFeedSummary, JobLifecycleStage, JobSourceVariant, JobView,
    };
    use crate::domain::profile::model::{Profile, ProfileAnalysis};
    use crate::domain::role::RoleId;
    use crate::domain::source::SourceId;
    use crate::services::applications::{ApplicationsService, ApplicationsServiceStub};
    use crate::services::jobs::{JobsService, JobsServiceStub};
    use crate::services::profiles::{ProfilesService, ProfilesServiceStub};
    use crate::services::resumes::{ResumesService, ResumesServiceStub};
    use crate::state::AppState;

    use super::{
        BulkProfileJobMatchRequest, JobContextQuery, RecentJobsQuery, bulk_profile_job_match,
        get_job_by_id, get_ml_job_lifecycle, get_profile_job_match, get_recent_jobs,
    };

    fn sample_job_view(id: &str) -> JobView {
        JobView {
            job: Job {
                id: id.to_string(),
                title: "Platform Engineer".to_string(),
                company_name: "SignalHire".to_string(),
                location: Some("Remote, Europe".to_string()),
                remote_type: Some("remote".to_string()),
                seniority: Some("senior".to_string()),
                description_text: "Rust and Postgres".to_string(),
                salary_min: Some(5000),
                salary_max: Some(6500),
                salary_currency: Some("USD".to_string()),
                posted_at: Some("2026-04-14T08:00:00Z".to_string()),
                last_seen_at: "2026-04-16T09:00:00Z".to_string(),
                is_active: true,
            },
            first_seen_at: "2026-04-14T08:00:00Z".to_string(),
            inactivated_at: None,
            reactivated_at: Some("2026-04-16T09:00:00Z".to_string()),
            lifecycle_stage: JobLifecycleStage::Reactivated,
            primary_variant: Some(JobSourceVariant {
                source: "mock_source".to_string(),
                source_job_id: "platform-001".to_string(),
                source_url: "https://mock-source.example/jobs/platform-001".to_string(),
                raw_payload: None,
                fetched_at: "2026-04-16T09:00:00Z".to_string(),
                last_seen_at: "2026-04-16T09:00:00Z".to_string(),
                is_active: true,
                inactivated_at: None,
            }),
        }
    }

    fn sample_profile() -> Profile {
        Profile {
            id: "profile-1".to_string(),
            name: "Jane Doe".to_string(),
            email: "jane@example.com".to_string(),
            location: Some("Kyiv".to_string()),
            raw_text:
                "Senior frontend engineer with React, TypeScript and design system experience"
                    .to_string(),
            analysis: Some(ProfileAnalysis {
                summary: "Senior frontend engineer".to_string(),
                primary_role: RoleId::FrontendDeveloper,
                seniority: "senior".to_string(),
                skills: vec!["react".to_string(), "typescript".to_string()],
                keywords: vec!["frontend".to_string(), "design system".to_string()],
            }),
            salary_min_usd: None,
            salary_max_usd: None,
            preferred_work_mode: None,
            created_at: "2026-04-14T08:00:00Z".to_string(),
            updated_at: "2026-04-14T08:00:00Z".to_string(),
            skills_updated_at: Some("2026-04-14T08:00:00Z".to_string()),
        }
    }

    #[tokio::test]
    async fn returns_service_unavailable_when_database_is_missing() {
        let result = get_job_by_id(
            State(AppState::without_database()),
            Path("job-123".to_string()),
            Query(JobContextQuery::default()),
        )
        .await;

        let response = match result {
            Ok(_) => panic!("handler should fail without a configured database"),
            Err(error) => error.into_response(),
        };

        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    async fn returns_not_found_for_unknown_job() {
        let state = AppState::for_services(
            ProfilesService::for_tests(ProfilesServiceStub::default()),
            JobsService::for_tests(JobsServiceStub::default()),
            ApplicationsService::for_tests(ApplicationsServiceStub::default()),
            ResumesService::for_tests(ResumesServiceStub::default()),
        );
        let result = get_job_by_id(
            State(state),
            Path("missing-job".to_string()),
            Query(JobContextQuery::default()),
        )
        .await;

        let response = match result {
            Ok(_) => panic!("handler should return not found for unknown job"),
            Err(error) => error.into_response(),
        };

        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        let body = body::to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("response body should be readable");
        let payload: Value =
            serde_json::from_slice(&body).expect("response body should be valid json");

        assert_eq!(payload["code"], json!("job_not_found"));
    }

    #[tokio::test]
    async fn rejects_invalid_recent_jobs_limit() {
        let result = get_recent_jobs(
            State(AppState::without_database()),
            Query(RecentJobsQuery {
                limit: Some(0),
                lifecycle: None,
                source: None,
                profile_id: None,
            }),
        )
        .await;

        let response = match result {
            Ok(Json(_)) => panic!("handler should reject invalid limit"),
            Err(error) => error.into_response(),
        };

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = body::to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("response body should be readable");
        let payload: Value =
            serde_json::from_slice(&body).expect("response body should be valid json");

        assert_eq!(payload["code"], json!("invalid_limit"));
    }

    #[tokio::test]
    async fn profile_job_match_returns_canonical_fit_diagnostics() {
        let frontend_job = JobView {
            job: Job {
                title: "Senior Front-end React Developer".to_string(),
                description_text: "Ship frontend design system features with React and TypeScript. Partner with product and design on accessibility and performance improvements. Own component architecture, testing quality, and release readiness across a shared UI platform used by multiple product teams. Drive performance budgets, documentation standards, and cross-team adoption for reusable components, tokens, and frontend platform tooling.".to_string(),
                ..sample_job_view("job-frontend-1").job
            },
            ..sample_job_view("job-frontend-1")
        };
        let state = AppState::for_services(
            ProfilesService::for_tests(
                ProfilesServiceStub::default().with_profile(sample_profile()),
            ),
            JobsService::for_tests(JobsServiceStub::default().with_job_view(frontend_job)),
            ApplicationsService::for_tests(ApplicationsServiceStub::default()),
            ResumesService::for_tests(ResumesServiceStub::default()),
        );

        let Json(response) = get_profile_job_match(
            State(state),
            Path(("profile-1".to_string(), "job-frontend-1".to_string())),
        )
        .await
        .expect("profile match should succeed");

        assert!(response.score > 0);
        assert!(response.matched_skills.contains(&"react".to_string()));
        assert!(
            response
                .positive_reasons
                .iter()
                .any(|reason| reason.contains("Matched"))
        );
        assert_eq!(response.description_quality, "strong");
    }

    #[tokio::test]
    async fn bulk_profile_job_match_supports_dashboard_sorting() {
        let strong_job = JobView {
            job: Job {
                title: "Senior Front-end React Developer".to_string(),
                description_text: "Ship frontend design system features with React and TypeScript. Collaborate with product, accessibility, and platform teams to improve shared components, design tokens, and performance budgets across multiple customer-facing surfaces.".to_string(),
                ..sample_job_view("job-frontend-strong").job
            },
            ..sample_job_view("job-frontend-strong")
        };
        let weak_job = JobView {
            job: Job {
                title: "Senior UI Engineer".to_string(),
                description_text: "Improve shared product experiences and collaborate with design."
                    .to_string(),
                ..sample_job_view("job-frontend-weak").job
            },
            ..sample_job_view("job-frontend-weak")
        };
        let state = AppState::for_services(
            ProfilesService::for_tests(
                ProfilesServiceStub::default().with_profile(sample_profile()),
            ),
            JobsService::for_tests(
                JobsServiceStub::default()
                    .with_job_view(strong_job)
                    .with_job_view(weak_job),
            ),
            ApplicationsService::for_tests(ApplicationsServiceStub::default()),
            ResumesService::for_tests(ResumesServiceStub::default()),
        );

        let Json(response) = bulk_profile_job_match(
            State(state),
            Path("profile-1".to_string()),
            ApiJson(BulkProfileJobMatchRequest {
                job_ids: vec![
                    "job-frontend-weak".to_string(),
                    "job-frontend-strong".to_string(),
                ],
            }),
        )
        .await
        .expect("bulk profile match should succeed");

        assert_eq!(response.results.len(), 2);
        assert_eq!(response.results[0].job_id, "job-frontend-strong");
        assert!(response.results[0].score > response.results[1].score);
        assert!(response.meta.low_evidence_jobs <= response.meta.returned_jobs);
    }

    #[tokio::test]
    async fn returns_job_feed_summary_and_lifecycle_fields() {
        let state = AppState::for_services(
            ProfilesService::for_tests(ProfilesServiceStub::default()),
            JobsService::for_tests(
                JobsServiceStub::default()
                    .with_job_view(sample_job_view("job-123"))
                    .with_feed_summary(JobFeedSummary {
                        total_jobs: 3,
                        active_jobs: 2,
                        inactive_jobs: 1,
                        reactivated_jobs: 1,
                    }),
            ),
            ApplicationsService::for_tests(ApplicationsServiceStub::default()),
            ResumesService::for_tests(ResumesServiceStub::default()),
        );

        let response = get_recent_jobs(
            State(state),
            Query(RecentJobsQuery {
                limit: Some(20),
                lifecycle: None,
                source: None,
                profile_id: None,
            }),
        )
        .await
        .expect("recent jobs should succeed")
        .into_response();

        let body = body::to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("response body should be readable");
        let payload: Value =
            serde_json::from_slice(&body).expect("response body should be valid JSON");

        assert_eq!(payload["summary"]["reactivated_jobs"], json!(1));
        assert_eq!(payload["jobs"][0]["lifecycle_stage"], json!("reactivated"));
        assert_eq!(payload["jobs"][0]["location"], json!("Remote, Europe"));
        assert_eq!(
            payload["jobs"][0]["primary_variant"]["source"],
            json!("mock_source")
        );
        assert_eq!(
            payload["jobs"][0]["presentation"]["salary_label"],
            json!("5,000-6,500 USD")
        );
    }

    #[tokio::test]
    async fn returns_ml_lifecycle_payload() {
        let state = AppState::for_services(
            ProfilesService::for_tests(ProfilesServiceStub::default()),
            JobsService::for_tests(
                JobsServiceStub::default().with_job_view(sample_job_view("job-123")),
            ),
            ApplicationsService::for_tests(ApplicationsServiceStub::default()),
            ResumesService::for_tests(ResumesServiceStub::default()),
        );

        let response = get_ml_job_lifecycle(State(state), Path("job-123".to_string()))
            .await
            .expect("ML lifecycle route should succeed")
            .into_response();

        let body = body::to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("response body should be readable");
        let payload: Value =
            serde_json::from_slice(&body).expect("response body should be valid JSON");

        assert_eq!(payload["id"], json!("job-123"));
        assert_eq!(payload["lifecycle_stage"], json!("reactivated"));
        assert_eq!(payload["location"], json!("Remote, Europe"));
        assert_eq!(
            payload["primary_variant"]["source_url"],
            json!("https://mock-source.example/jobs/platform-001")
        );
        assert_eq!(payload["presentation"]["location_label"], json!("Europe"));
    }

    #[test]
    fn recent_jobs_query_accepts_known_source() {
        let uri: Uri = "/api/v1/jobs/recent?source=djinni"
            .parse()
            .expect("uri should parse");
        let Query(query) =
            Query::<RecentJobsQuery>::try_from_uri(&uri).expect("query should deserialize");

        assert_eq!(query.source, Some(SourceId::Djinni));
    }

    #[test]
    fn recent_jobs_query_rejects_unknown_source() {
        let uri: Uri = "/api/v1/jobs/recent?source=linkedin"
            .parse()
            .expect("uri should parse");

        let result = Query::<RecentJobsQuery>::try_from_uri(&uri);

        assert!(result.is_err());
    }
}
