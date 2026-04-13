use axum::extract::{Path, Query, State};
use serde::Deserialize;

use crate::api::dto::applications::{
    ActivityResponse, ApplicationDetailResponse, ApplicationResponse, CreateActivityRequest,
    CreateApplicationRequest, CreateNoteRequest, NoteResponse, RecentApplicationsResponse,
    UpdateApplicationRequest,
};
use crate::api::error::{ApiError, ApiJson};
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct RecentApplicationsQuery {
    pub limit: Option<i64>,
}

pub async fn get_application_by_id(
    State(state): State<AppState>,
    Path(application_id): Path<String>,
) -> Result<axum::Json<ApplicationDetailResponse>, ApiError> {
    let Some(application) = state
        .applications_service
        .get_detail_by_id(&application_id)
        .await
        .map_err(|error| ApiError::from_repository(error, "applications_query_failed"))?
    else {
        return Err(ApiError::not_found(
            "application_not_found",
            format!("Application '{application_id}' was not found"),
        ));
    };

    Ok(axum::Json(ApplicationDetailResponse::from(application)))
}

pub async fn get_recent_applications(
    State(state): State<AppState>,
    Query(query): Query<RecentApplicationsQuery>,
) -> Result<axum::Json<RecentApplicationsResponse>, ApiError> {
    let limit = query.limit.unwrap_or(20);

    if !(1..=100).contains(&limit) {
        return Err(ApiError::invalid_limit(limit));
    }

    let applications = state
        .applications_service
        .list_recent(limit)
        .await
        .map_err(|error| ApiError::from_repository(error, "applications_query_failed"))?;

    Ok(axum::Json(RecentApplicationsResponse {
        applications: applications
            .into_iter()
            .map(ApplicationResponse::from)
            .collect(),
    }))
}

pub async fn create_application(
    State(state): State<AppState>,
    ApiJson(payload): ApiJson<CreateApplicationRequest>,
) -> Result<(axum::http::StatusCode, axum::Json<ApplicationResponse>), ApiError> {
    let payload = payload.validate()?;

    let Some(_) = state
        .jobs_service
        .get_by_id(&payload.job_id)
        .await
        .map_err(|error| ApiError::from_repository(error, "applications_query_failed"))?
    else {
        return Err(ApiError::not_found(
            "job_not_found",
            format!("Job '{}' was not found", payload.job_id),
        ));
    };

    let mut application = state
        .applications_service
        .create(payload)
        .await
        .map_err(|error| ApiError::from_repository(error, "applications_query_failed"))?;

    if let Some(active_resume) = state
        .resumes_service
        .get_active()
        .await
        .map_err(|error| ApiError::from_repository(error, "applications_query_failed"))?
    {
        if let Some(updated) = state
            .applications_service
            .attach_resume(&application.id, &active_resume.id)
            .await
            .map_err(|error| ApiError::from_repository(error, "applications_query_failed"))?
        {
            application = updated;
        }
    }

    Ok((
        axum::http::StatusCode::CREATED,
        axum::Json(ApplicationResponse::from(application)),
    ))
}

pub async fn patch_application(
    State(state): State<AppState>,
    Path(application_id): Path<String>,
    ApiJson(payload): ApiJson<UpdateApplicationRequest>,
) -> Result<axum::Json<ApplicationResponse>, ApiError> {
    let update = payload.validate()?;
    let new_status = update.status.clone();

    let Some(application) = state
        .applications_service
        .update(&application_id, update)
        .await
        .map_err(|error| ApiError::from_repository(error, "applications_query_failed"))?
    else {
        return Err(ApiError::not_found(
            "application_not_found",
            format!("Application '{application_id}' was not found"),
        ));
    };

    // Fire-and-forget: create a follow-up reminder task if status changed.
    if let Some(ref status) = new_status {
        state
            .followup_service
            .on_status_change(&application_id, status)
            .await;
    }

    Ok(axum::Json(ApplicationResponse::from(application)))
}

pub async fn create_activity(
    State(state): State<AppState>,
    Path(application_id): Path<String>,
    ApiJson(payload): ApiJson<CreateActivityRequest>,
) -> Result<(axum::http::StatusCode, axum::Json<ActivityResponse>), ApiError> {
    // Verify the application exists first.
    let Some(_) = state
        .applications_service
        .get_by_id(&application_id)
        .await
        .map_err(|error| ApiError::from_repository(error, "activities_query_failed"))?
    else {
        return Err(ApiError::not_found(
            "application_not_found",
            format!("Application '{application_id}' was not found"),
        ));
    };

    let is_interview = payload.activity_type == "interview";
    let activity = state
        .activities_service
        .create(payload.validate(&application_id)?)
        .await
        .map_err(|error| ApiError::from_repository(error, "activities_query_failed"))?;

    // Fire-and-forget: create a thank-you note reminder after an interview activity.
    if is_interview {
        state
            .followup_service
            .on_interview_activity(&application_id)
            .await;
    }

    Ok((
        axum::http::StatusCode::CREATED,
        axum::Json(ActivityResponse::from(activity)),
    ))
}

pub async fn create_note(
    State(state): State<AppState>,
    Path(application_id): Path<String>,
    ApiJson(payload): ApiJson<CreateNoteRequest>,
) -> Result<(axum::http::StatusCode, axum::Json<NoteResponse>), ApiError> {
    // Verify the application exists first.
    let Some(_) = state
        .applications_service
        .get_by_id(&application_id)
        .await
        .map_err(|error| ApiError::from_repository(error, "notes_query_failed"))?
    else {
        return Err(ApiError::not_found(
            "application_not_found",
            format!("Application '{application_id}' was not found"),
        ));
    };

    let note = state
        .applications_service
        .create_note(payload.validate(&application_id)?)
        .await
        .map_err(|error| ApiError::from_repository(error, "notes_query_failed"))?;

    Ok((
        axum::http::StatusCode::CREATED,
        axum::Json(NoteResponse::from(note)),
    ))
}

#[cfg(test)]
mod tests {
    use axum::Json;
    use axum::extract::{Path, Query, State};
    use axum::response::IntoResponse;
    use axum::{body, http::StatusCode};
    use serde_json::{Value, json};

    use crate::api::error::ApiJson;
    use crate::services::applications::{ApplicationsService, ApplicationsServiceStub};
    use crate::services::jobs::{JobsService, JobsServiceStub};
    use crate::services::profiles::{ProfilesService, ProfilesServiceStub};
    use crate::services::resumes::{ResumesService, ResumesServiceStub};
    use crate::state::AppState;

    use super::{
        RecentApplicationsQuery, create_application, get_application_by_id,
        get_recent_applications, patch_application,
    };

    #[tokio::test]
    async fn returns_service_unavailable_when_database_is_missing() {
        let result = get_application_by_id(
            State(AppState::without_database()),
            Path("application-123".to_string()),
        )
        .await;

        let response = match result {
            Ok(_) => panic!("handler should fail without a configured database"),
            Err(error) => error.into_response(),
        };

        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    async fn returns_not_found_for_unknown_application() {
        let state = AppState::for_services(
            ProfilesService::for_tests(ProfilesServiceStub::default()),
            JobsService::for_tests(JobsServiceStub::default().with_job(
                crate::domain::job::model::Job {
                    id: "job-1".to_string(),
                    title: "Backend Rust Engineer".to_string(),
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
                },
            )),
            ApplicationsService::for_tests(ApplicationsServiceStub::default()),
            ResumesService::for_tests(ResumesServiceStub::default()),
        );
        let result =
            get_application_by_id(State(state), Path("missing-application".to_string())).await;

        let response = match result {
            Ok(_) => panic!("handler should return not found for unknown application"),
            Err(error) => error.into_response(),
        };

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn rejects_invalid_recent_applications_limit() {
        let result = get_recent_applications(
            State(AppState::without_database()),
            Query(RecentApplicationsQuery { limit: Some(0) }),
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
    async fn rejects_invalid_patch_payload() {
        let state = AppState::for_services(
            ProfilesService::for_tests(ProfilesServiceStub::default()),
            JobsService::for_tests(JobsServiceStub::default()),
            ApplicationsService::for_tests(ApplicationsServiceStub::default()),
            ResumesService::for_tests(ResumesServiceStub::default()),
        );

        let response = patch_application(
            State(state),
            Path("application-1".to_string()),
            ApiJson(crate::api::dto::applications::UpdateApplicationRequest::default()),
        )
        .await
        .expect_err("handler should reject empty patch")
        .into_response();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn creates_application() {
        let state = AppState::for_services(
            ProfilesService::for_tests(ProfilesServiceStub::default()),
            JobsService::for_tests(JobsServiceStub::default().with_job(
                crate::domain::job::model::Job {
                    id: "job-1".to_string(),
                    title: "Backend Rust Engineer".to_string(),
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
                },
            )),
            ApplicationsService::for_tests(ApplicationsServiceStub::default()),
            ResumesService::for_tests(ResumesServiceStub::default()),
        );

        let result = create_application(
            State(state),
            ApiJson(crate::api::dto::applications::CreateApplicationRequest {
                job_id: "job-1".to_string(),
                status: "saved".to_string(),
                applied_at: None,
            }),
        )
        .await
        .expect("handler should create application");

        assert_eq!(result.0, StatusCode::CREATED);
    }
}
