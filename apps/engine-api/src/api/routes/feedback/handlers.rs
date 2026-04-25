use axum::Extension;
use axum::extract::{Path, State};
use axum::http::StatusCode;

use crate::api::middleware::auth::{AuthUser, check_profile_ownership};

use crate::api::dto::feedback::{
    CompanyFeedbackResponse, FeedbackOverviewResponse, FeedbackSummary, JobFeedbackResponse,
    SetInterestRatingRequest, SetLegitimacySignalRequest, SetSalaryFeedbackRequest,
    SetWorkModeFeedbackRequest, TagJobFeedbackRequest, UpdateCompanyFeedbackRequest,
};
use crate::api::error::{ApiError, ApiJson};
use crate::api::routes::events::{
    load_job_event_metadata, log_user_event_softly, record_labelable_job_softly,
};
use crate::domain::feedback::model::{CompanyFeedbackStatus, JobFeedbackFlags};
use crate::domain::user_event::model::{CreateUserEvent, UserEventType};
use crate::services::feedback::FeedbackService;
use crate::state::AppState;

pub async fn list_feedback(
    State(state): State<AppState>,
    auth: Option<Extension<AuthUser>>,
    Path(profile_id): Path<String>,
) -> Result<axum::Json<FeedbackOverviewResponse>, ApiError> {
    ensure_profile_exists(&state, auth.as_deref(), &profile_id).await?;

    let jobs = state
        .feedback_service
        .list_job_feedback(&profile_id)
        .await
        .map_err(|error| ApiError::from_repository(error, "feedback_query_failed"))?;

    let companies = state
        .feedback_service
        .list_company_feedback(&profile_id)
        .await
        .map_err(|error| ApiError::from_repository(error, "feedback_query_failed"))?;

    let jobs: Vec<JobFeedbackResponse> = jobs.into_iter().map(JobFeedbackResponse::from).collect();

    let companies: Vec<CompanyFeedbackResponse> = companies
        .into_iter()
        .map(CompanyFeedbackResponse::from)
        .collect();

    let summary = FeedbackSummary {
        saved_jobs_count: jobs.iter().filter(|j| j.saved).count(),
        hidden_jobs_count: jobs.iter().filter(|j| j.hidden).count(),
        bad_fit_jobs_count: jobs.iter().filter(|j| j.bad_fit).count(),
        whitelisted_companies_count: companies.iter().filter(|c| c.status == "whitelist").count(),
        blacklisted_companies_count: companies.iter().filter(|c| c.status == "blacklist").count(),
    };

    Ok(axum::Json(FeedbackOverviewResponse {
        profile_id,
        jobs,
        companies,
        summary,
    }))
}

pub async fn save_job(
    State(state): State<AppState>,
    auth: Option<Extension<AuthUser>>,
    Path((profile_id, job_id)): Path<(String, String)>,
) -> Result<axum::Json<JobFeedbackResponse>, ApiError> {
    update_job_feedback(
        state,
        auth.as_deref(),
        profile_id,
        job_id,
        UserEventType::JobSaved,
        JobFeedbackFlags {
            saved: true,
            ..JobFeedbackFlags::default()
        },
    )
    .await
}

pub async fn hide_job(
    State(state): State<AppState>,
    auth: Option<Extension<AuthUser>>,
    Path((profile_id, job_id)): Path<(String, String)>,
) -> Result<axum::Json<JobFeedbackResponse>, ApiError> {
    update_job_feedback(
        state,
        auth.as_deref(),
        profile_id,
        job_id,
        UserEventType::JobHidden,
        JobFeedbackFlags {
            hidden: true,
            ..JobFeedbackFlags::default()
        },
    )
    .await
}

pub async fn mark_job_bad_fit(
    State(state): State<AppState>,
    auth: Option<Extension<AuthUser>>,
    Path((profile_id, job_id)): Path<(String, String)>,
) -> Result<axum::Json<JobFeedbackResponse>, ApiError> {
    update_job_feedback(
        state,
        auth.as_deref(),
        profile_id,
        job_id,
        UserEventType::JobBadFit,
        JobFeedbackFlags {
            bad_fit: true,
            ..JobFeedbackFlags::default()
        },
    )
    .await
}

pub async fn unsave_job(
    State(state): State<AppState>,
    auth: Option<Extension<AuthUser>>,
    Path((profile_id, job_id)): Path<(String, String)>,
) -> Result<StatusCode, ApiError> {
    clear_job_feedback_flags(
        state,
        auth.as_deref(),
        profile_id,
        job_id,
        UserEventType::JobUnsaved,
        JobFeedbackFlags {
            saved: true,
            ..JobFeedbackFlags::default()
        },
    )
    .await
}

pub async fn unhide_job(
    State(state): State<AppState>,
    auth: Option<Extension<AuthUser>>,
    Path((profile_id, job_id)): Path<(String, String)>,
) -> Result<StatusCode, ApiError> {
    clear_job_feedback_flags(
        state,
        auth.as_deref(),
        profile_id,
        job_id,
        UserEventType::JobUnhidden,
        JobFeedbackFlags {
            hidden: true,
            ..JobFeedbackFlags::default()
        },
    )
    .await
}

pub async fn unmark_job_bad_fit(
    State(state): State<AppState>,
    auth: Option<Extension<AuthUser>>,
    Path((profile_id, job_id)): Path<(String, String)>,
) -> Result<StatusCode, ApiError> {
    clear_job_feedback_flags(
        state,
        auth.as_deref(),
        profile_id,
        job_id,
        UserEventType::JobBadFitRemoved,
        JobFeedbackFlags {
            bad_fit: true,
            ..JobFeedbackFlags::default()
        },
    )
    .await
}

pub async fn add_company_whitelist(
    State(state): State<AppState>,
    auth: Option<Extension<AuthUser>>,
    Path(profile_id): Path<String>,
    ApiJson(payload): ApiJson<UpdateCompanyFeedbackRequest>,
) -> Result<axum::Json<CompanyFeedbackResponse>, ApiError> {
    update_company_feedback(
        state,
        auth.as_deref(),
        profile_id,
        payload,
        CompanyFeedbackStatus::Whitelist,
    )
    .await
}

pub async fn remove_company_whitelist(
    State(state): State<AppState>,
    auth: Option<Extension<AuthUser>>,
    Path(profile_id): Path<String>,
    ApiJson(payload): ApiJson<UpdateCompanyFeedbackRequest>,
) -> Result<StatusCode, ApiError> {
    remove_company_feedback(
        state,
        auth.as_deref(),
        profile_id,
        payload,
        CompanyFeedbackStatus::Whitelist,
    )
    .await
}

pub async fn add_company_blacklist(
    State(state): State<AppState>,
    auth: Option<Extension<AuthUser>>,
    Path(profile_id): Path<String>,
    ApiJson(payload): ApiJson<UpdateCompanyFeedbackRequest>,
) -> Result<axum::Json<CompanyFeedbackResponse>, ApiError> {
    update_company_feedback(
        state,
        auth.as_deref(),
        profile_id,
        payload,
        CompanyFeedbackStatus::Blacklist,
    )
    .await
}

pub async fn remove_company_blacklist(
    State(state): State<AppState>,
    auth: Option<Extension<AuthUser>>,
    Path(profile_id): Path<String>,
    ApiJson(payload): ApiJson<UpdateCompanyFeedbackRequest>,
) -> Result<StatusCode, ApiError> {
    remove_company_feedback(
        state,
        auth.as_deref(),
        profile_id,
        payload,
        CompanyFeedbackStatus::Blacklist,
    )
    .await
}

pub async fn set_job_salary_signal(
    State(state): State<AppState>,
    auth: Option<Extension<AuthUser>>,
    Path((profile_id, job_id)): Path<(String, String)>,
    ApiJson(payload): ApiJson<SetSalaryFeedbackRequest>,
) -> Result<axum::Json<JobFeedbackResponse>, ApiError> {
    ensure_profile_exists(&state, auth.as_deref(), &profile_id).await?;
    let signal = payload.validate()?;
    let feedback = state
        .feedback_service
        .set_salary_signal(&profile_id, &job_id, signal)
        .await
        .map_err(|error| ApiError::from_repository(error, "feedback_write_failed"))?;
    Ok(axum::Json(JobFeedbackResponse::from(feedback)))
}

pub async fn set_job_interest_rating(
    State(state): State<AppState>,
    auth: Option<Extension<AuthUser>>,
    Path((profile_id, job_id)): Path<(String, String)>,
    ApiJson(payload): ApiJson<SetInterestRatingRequest>,
) -> Result<axum::Json<JobFeedbackResponse>, ApiError> {
    ensure_profile_exists(&state, auth.as_deref(), &profile_id).await?;
    let rating = payload.validate()?;
    let feedback = state
        .feedback_service
        .set_interest_rating(&profile_id, &job_id, rating)
        .await
        .map_err(|error| ApiError::from_repository(error, "feedback_write_failed"))?;
    Ok(axum::Json(JobFeedbackResponse::from(feedback)))
}

pub async fn set_job_work_mode_signal(
    State(state): State<AppState>,
    auth: Option<Extension<AuthUser>>,
    Path((profile_id, job_id)): Path<(String, String)>,
    ApiJson(payload): ApiJson<SetWorkModeFeedbackRequest>,
) -> Result<axum::Json<JobFeedbackResponse>, ApiError> {
    ensure_profile_exists(&state, auth.as_deref(), &profile_id).await?;
    let signal = payload.validate()?;
    let feedback = state
        .feedback_service
        .set_work_mode_signal(&profile_id, &job_id, signal)
        .await
        .map_err(|error| ApiError::from_repository(error, "feedback_write_failed"))?;
    if matches!(
        signal,
        crate::domain::feedback::model::WorkModeFeedbackSignal::DealBreaker
    ) {
        record_labelable_job_softly(&state, &profile_id, &job_id).await;
    }
    Ok(axum::Json(JobFeedbackResponse::from(feedback)))
}

pub async fn set_job_legitimacy_signal(
    State(state): State<AppState>,
    auth: Option<Extension<AuthUser>>,
    Path((profile_id, job_id)): Path<(String, String)>,
    ApiJson(payload): ApiJson<SetLegitimacySignalRequest>,
) -> Result<axum::Json<JobFeedbackResponse>, ApiError> {
    ensure_profile_exists(&state, auth.as_deref(), &profile_id).await?;
    let signal = payload.validate()?;
    let feedback = state
        .feedback_service
        .set_legitimacy_signal(&profile_id, &job_id, signal)
        .await
        .map_err(|error| ApiError::from_repository(error, "feedback_write_failed"))?;
    if matches!(
        signal,
        crate::domain::feedback::model::LegitimacySignal::Spam
            | crate::domain::feedback::model::LegitimacySignal::Suspicious
    ) {
        record_labelable_job_softly(&state, &profile_id, &job_id).await;
    }
    Ok(axum::Json(JobFeedbackResponse::from(feedback)))
}

pub async fn tag_job_feedback(
    State(state): State<AppState>,
    auth: Option<Extension<AuthUser>>,
    Path((profile_id, job_id)): Path<(String, String)>,
    ApiJson(payload): ApiJson<TagJobFeedbackRequest>,
) -> Result<axum::http::StatusCode, ApiError> {
    ensure_profile_exists(&state, auth.as_deref(), &profile_id).await?;
    let tags = payload.validate()?;
    state
        .feedback_service
        .upsert_job_feedback_tags(&profile_id, &job_id, tags)
        .await
        .map_err(|error| ApiError::from_repository(error, "feedback_write_failed"))?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}

pub(crate) async fn ensure_profile_exists(
    state: &AppState,
    auth: Option<&AuthUser>,
    profile_id: &str,
) -> Result<(), ApiError> {
    check_profile_ownership(auth, profile_id)?;

    let Some(_) = state
        .profile_records
        .get_by_id(profile_id)
        .await
        .map_err(|error| ApiError::from_repository(error, "profiles_query_failed"))?
    else {
        return Err(ApiError::not_found(
            "profile_not_found",
            format!("Profile '{profile_id}' was not found"),
        ));
    };

    Ok(())
}

async fn clear_job_feedback_flags(
    state: AppState,
    auth: Option<&AuthUser>,
    profile_id: String,
    job_id: String,
    event_type: UserEventType,
    flags: JobFeedbackFlags,
) -> Result<StatusCode, ApiError> {
    ensure_profile_exists(&state, auth, &profile_id).await?;

    let metadata = load_job_event_metadata(&state, &job_id).await?;

    state
        .feedback_service
        .clear_job_feedback(&profile_id, &job_id, flags)
        .await
        .map_err(|error| ApiError::from_repository(error, "feedback_write_failed"))?;

    log_user_event_softly(
        &state,
        CreateUserEvent {
            profile_id,
            event_type,
            job_id: Some(job_id),
            company_name: metadata.company_name,
            source: metadata.source,
            role_family: metadata.role_family,
            payload_json: None,
        },
    )
    .await;

    Ok(StatusCode::NO_CONTENT)
}

async fn update_job_feedback(
    state: AppState,
    auth: Option<&AuthUser>,
    profile_id: String,
    job_id: String,
    event_type: UserEventType,
    flags: JobFeedbackFlags,
) -> Result<axum::Json<JobFeedbackResponse>, ApiError> {
    ensure_profile_exists(&state, auth, &profile_id).await?;

    let metadata = load_job_event_metadata(&state, &job_id).await?;

    let feedback = state
        .feedback_service
        .upsert_job_feedback(&profile_id, &job_id, flags)
        .await
        .map_err(|error| ApiError::from_repository(error, "feedback_write_failed"))?;

    log_user_event_softly(
        &state,
        CreateUserEvent {
            profile_id,
            event_type,
            job_id: Some(job_id),
            company_name: metadata.company_name,
            source: metadata.source,
            role_family: metadata.role_family,
            payload_json: None,
        },
    )
    .await;

    Ok(axum::Json(JobFeedbackResponse::from(feedback)))
}

async fn update_company_feedback(
    state: AppState,
    auth: Option<&AuthUser>,
    profile_id: String,
    payload: UpdateCompanyFeedbackRequest,
    status: CompanyFeedbackStatus,
) -> Result<axum::Json<CompanyFeedbackResponse>, ApiError> {
    ensure_profile_exists(&state, auth, &profile_id).await?;

    let company_name = payload.validate_company_name()?;
    let normalized_company_name = FeedbackService::normalize_company_name(&company_name);
    let feedback = state
        .feedback_service
        .upsert_company_feedback(&profile_id, &company_name, &normalized_company_name, status)
        .await
        .map_err(|error| ApiError::from_repository(error, "feedback_write_failed"))?;

    let event_type = match status {
        CompanyFeedbackStatus::Whitelist => UserEventType::CompanyWhitelisted,
        CompanyFeedbackStatus::Blacklist => UserEventType::CompanyBlacklisted,
    };
    log_user_event_softly(
        &state,
        CreateUserEvent {
            profile_id,
            event_type,
            job_id: None,
            company_name: Some(company_name),
            source: None,
            role_family: None,
            payload_json: None,
        },
    )
    .await;

    Ok(axum::Json(CompanyFeedbackResponse::from(feedback)))
}

async fn remove_company_feedback(
    state: AppState,
    auth: Option<&AuthUser>,
    profile_id: String,
    payload: UpdateCompanyFeedbackRequest,
    status: CompanyFeedbackStatus,
) -> Result<StatusCode, ApiError> {
    ensure_profile_exists(&state, auth, &profile_id).await?;

    let company_name = payload.validate_company_name()?;
    let normalized_company_name = FeedbackService::normalize_company_name(&company_name);
    state
        .feedback_service
        .remove_company_feedback(&profile_id, &normalized_company_name, status)
        .await
        .map_err(|error| ApiError::from_repository(error, "feedback_write_failed"))?;

    Ok(StatusCode::NO_CONTENT)
}
