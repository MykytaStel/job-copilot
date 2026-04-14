use axum::extract::{Query, State};
use serde::Deserialize;

use crate::api::dto::search::{RunSearchRequest, RunSearchResponse, SearchResponse};
use crate::api::error::{ApiError, ApiJson};
use crate::state::AppState;

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

    let jobs = state
        .jobs_service
        .list_filtered_views(fetch_limit, Some("active"), None)
        .await
        .map_err(|error| ApiError::from_repository(error, "search_run_failed"))?;
    let result =
        state
            .search_matching_service
            .run(&input.search_profile, jobs, input.limit as usize);

    Ok(axum::Json(RunSearchResponse::from_result(result)))
}

#[cfg(test)]
mod tests {
    use axum::extract::{Query, State};
    use axum::response::IntoResponse;
    use axum::{Json, body};
    use serde_json::{Value, json};

    use crate::api::dto::search::{RunSearchRequest, SearchProfileRequest};
    use crate::api::error::ApiJson;
    use crate::domain::job::model::{Job, JobLifecycleStage, JobSourceVariant, JobView};
    use crate::domain::search::profile::{TargetRegion, WorkMode};
    use crate::services::applications::{ApplicationsService, ApplicationsServiceStub};
    use crate::services::jobs::{JobsService, JobsServiceStub};
    use crate::services::profiles::{ProfilesService, ProfilesServiceStub};
    use crate::services::resumes::{ResumesService, ResumesServiceStub};
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
    async fn run_search_returns_ranked_jobs_with_fit_reasons() {
        let state = AppState::for_services(
            ProfilesService::for_tests(ProfilesServiceStub::default()),
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
}
