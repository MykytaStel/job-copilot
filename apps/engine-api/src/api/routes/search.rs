use axum::extract::{Query, State};
use serde::Deserialize;

use crate::api::dto::search::SearchResponse;
use crate::api::error::ApiError;
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

#[cfg(test)]
mod tests {
    use axum::extract::{Query, State};
    use axum::response::IntoResponse;
    use axum::{Json, body};
    use serde_json::Value;

    use crate::services::applications::{ApplicationsService, ApplicationsServiceStub};
    use crate::services::jobs::{JobsService, JobsServiceStub};
    use crate::services::profiles::{ProfilesService, ProfilesServiceStub};
    use crate::services::resumes::{ResumesService, ResumesServiceStub};
    use crate::state::AppState;

    use super::{SearchQuery, search};

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
}
