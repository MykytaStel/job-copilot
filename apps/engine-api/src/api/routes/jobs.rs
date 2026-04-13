use axum::extract::{Path, Query, State};
use serde::Deserialize;
use tracing::warn;

use crate::api::dto::jobs::{JobResponse, RecentJobsResponse};
use crate::api::dto::ranking::FitScoreResponse;
use crate::api::error::ApiError;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct RecentJobsQuery {
    pub limit: Option<i64>,
}

pub async fn get_job_by_id(
    State(state): State<AppState>,
    Path(job_id): Path<String>,
) -> Result<axum::Json<JobResponse>, ApiError> {
    let Some(job) = state
        .jobs_service
        .get_by_id(&job_id)
        .await
        .map_err(|error| ApiError::from_repository(error, "jobs_query_failed"))?
    else {
        return Err(ApiError::not_found(
            "job_not_found",
            format!("Job '{job_id}' was not found"),
        ));
    };

    Ok(axum::Json(JobResponse::from(job)))
}

pub async fn get_recent_jobs(
    State(state): State<AppState>,
    Query(query): Query<RecentJobsQuery>,
) -> Result<axum::Json<RecentJobsResponse>, ApiError> {
    let limit = query.limit.unwrap_or(20);

    if !(1..=100).contains(&limit) {
        return Err(ApiError::invalid_limit(limit));
    }

    let jobs = state
        .jobs_service
        .list_recent(limit)
        .await
        .map_err(|error| ApiError::from_repository(error, "jobs_query_failed"))?;

    Ok(axum::Json(RecentJobsResponse {
        jobs: jobs.into_iter().map(JobResponse::from).collect(),
    }))
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

/// Compute a fast local fit score for a job against the active resume.
/// No external API call — runs in microseconds.
pub async fn get_job_fit(
    State(state): State<AppState>,
    Path(job_id): Path<String>,
) -> Result<axum::Json<FitScoreResponse>, ApiError> {
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

    let candidate = state.profile_analysis_service.analyze(&resume.raw_text);
    // Best-effort: load the user's stored profile for salary/work mode prefs.
    // If no profile exists yet, salary + workmode components default to neutral (0.5).
    let profile = state.profiles_service.get_latest().await.ok().flatten();

    let score = state
        .ranking_service
        .compute(&candidate, &job, profile.as_ref());

    // Persist the computed score — this is a cache write, not the primary result.
    // Log a warning if it fails but do not fail the request.
    if let Err(error) = state
        .fit_scores_repository
        .upsert(&score, &resume.id)
        .await
    {
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

    if let Err(error) = state
        .fit_scores_repository
        .upsert(&score, &resume.id)
        .await
    {
        warn!(job_id = %job_id, resume_id = %resume.id, error = %error, "failed to persist fit score");
    }

    Ok(axum::Json(FitScoreResponse::from(score)))
}

#[cfg(test)]
mod tests {
    use axum::Json;
    use axum::extract::{Path, Query, State};
    use axum::response::IntoResponse;
    use axum::{body, http::StatusCode};
    use serde_json::{Value, json};

    use crate::services::applications::{ApplicationsService, ApplicationsServiceStub};
    use crate::services::jobs::{JobsService, JobsServiceStub};
    use crate::services::profiles::{ProfilesService, ProfilesServiceStub};
    use crate::services::resumes::{ResumesService, ResumesServiceStub};
    use crate::state::AppState;

    use super::{RecentJobsQuery, get_job_by_id, get_recent_jobs};

    #[tokio::test]
    async fn returns_service_unavailable_when_database_is_missing() {
        let result = get_job_by_id(
            State(AppState::without_database()),
            Path("job-123".to_string()),
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
        let result = get_job_by_id(State(state), Path("missing-job".to_string())).await;

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
            Query(RecentJobsQuery { limit: Some(0) }),
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
}
