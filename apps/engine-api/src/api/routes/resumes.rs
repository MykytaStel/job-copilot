use axum::extract::{Path, State};

use crate::api::dto::resumes::{ResumeVersionResponse, UploadResumeRequest};
use crate::api::error::{ApiError, ApiJson};
use crate::state::AppState;

pub async fn list_resumes(
    State(state): State<AppState>,
) -> Result<axum::Json<Vec<ResumeVersionResponse>>, ApiError> {
    let resumes = state
        .resumes_service
        .list()
        .await
        .map_err(|error| ApiError::from_repository(error, "resumes_query_failed"))?;

    Ok(axum::Json(
        resumes
            .into_iter()
            .map(ResumeVersionResponse::from)
            .collect(),
    ))
}

pub async fn get_active_resume(
    State(state): State<AppState>,
) -> Result<axum::Json<ResumeVersionResponse>, ApiError> {
    let Some(resume) = state
        .resumes_service
        .get_active()
        .await
        .map_err(|error| ApiError::from_repository(error, "resumes_query_failed"))?
    else {
        return Err(ApiError::not_found(
            "active_resume_not_found",
            "No active resume was found",
        ));
    };

    Ok(axum::Json(ResumeVersionResponse::from(resume)))
}

pub async fn upload_resume(
    State(state): State<AppState>,
    ApiJson(payload): ApiJson<UploadResumeRequest>,
) -> Result<(axum::http::StatusCode, axum::Json<ResumeVersionResponse>), ApiError> {
    let resume = state
        .resumes_service
        .upload(payload.validate()?)
        .await
        .map_err(|error| ApiError::from_repository(error, "resumes_query_failed"))?;

    Ok((
        axum::http::StatusCode::CREATED,
        axum::Json(ResumeVersionResponse::from(resume)),
    ))
}

pub async fn activate_resume(
    State(state): State<AppState>,
    Path(resume_id): Path<String>,
) -> Result<axum::Json<ResumeVersionResponse>, ApiError> {
    let Some(resume) = state
        .resumes_service
        .activate(&resume_id)
        .await
        .map_err(|error| ApiError::from_repository(error, "resumes_query_failed"))?
    else {
        return Err(ApiError::not_found(
            "resume_not_found",
            format!("Resume '{resume_id}' was not found"),
        ));
    };

    Ok(axum::Json(ResumeVersionResponse::from(resume)))
}

pub async fn delete_resume(
    State(state): State<AppState>,
    Path(resume_id): Path<String>,
) -> Result<axum::http::StatusCode, ApiError> {
    let deleted = state
        .resumes_service
        .delete(&resume_id)
        .await
        .map_err(|error| ApiError::from_repository(error, "resumes_query_failed"))?;

    if !deleted {
        return Err(ApiError::not_found(
            "resume_not_found",
            format!("Resume '{resume_id}' was not found"),
        ));
    }

    Ok(axum::http::StatusCode::NO_CONTENT)
}
