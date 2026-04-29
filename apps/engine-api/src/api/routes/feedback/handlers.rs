use axum::{
    Extension,
    extract::{Path, Query, State},
    http::{StatusCode, header},
    response::{IntoResponse, Response},
};
use chrono::Utc;
use serde::Deserialize;
use std::collections::HashMap;

use crate::api::middleware::auth::{AuthUser, check_profile_ownership};

use crate::api::dto::feedback::{
    BulkFeedbackActionResponse, BulkHideJobsByCompanyRequest, CompanyFeedbackResponse,
    FeedbackOverviewResponse, FeedbackStatsResponse, FeedbackSummary, FeedbackTimelineItemResponse,
    FeedbackTimelineResponse, JobFeedbackResponse, MarkJobBadFitRequest, SetInterestRatingRequest,
    SetLegitimacySignalRequest, SetSalaryFeedbackRequest, SetWorkModeFeedbackRequest,
    TagJobFeedbackRequest, UpdateCompanyFeedbackNotesRequest, UpdateCompanyFeedbackRequest,
};
use crate::api::error::{ApiError, ApiJson, OptionalApiJson};
use crate::api::routes::events::{
    load_job_event_metadata, log_user_event_softly, record_labelable_job_softly,
};
use crate::domain::feedback::model::{CompanyFeedbackStatus, JobFeedbackFlags, JobFeedbackRecord};
use crate::domain::job::model::JobView;
use crate::domain::user_event::model::{CreateUserEvent, UserEventType};
use crate::services::feedback::FeedbackService;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct RemoveCompanyBlacklistBySlugQuery {
    pub profile_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCompanyFeedbackBySlugQuery {
    pub profile_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct BulkHideJobsByCompanyQuery {
    pub profile_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct JobFeedbackActionQuery {
    pub profile_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct FeedbackStatsQuery {
    pub profile_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct FeedbackTimelineQuery {
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

#[derive(Debug, Deserialize)]
pub struct ExportFeedbackQuery {
    #[serde(rename = "type")]
    pub export_type: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FeedbackExportType {
    Saved,
    Hidden,
    BadFit,
    Companies,
}

impl FeedbackExportType {
    fn parse(value: &str) -> Option<Self> {
        match value.trim() {
            "saved" => Some(Self::Saved),
            "hidden" => Some(Self::Hidden),
            "bad_fit" => Some(Self::BadFit),
            "companies" => Some(Self::Companies),
            _ => None,
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Saved => "saved",
            Self::Hidden => "hidden",
            Self::BadFit => "bad_fit",
            Self::Companies => "companies",
        }
    }
}

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

pub async fn export_feedback_csv(
    State(state): State<AppState>,
    auth: Option<Extension<AuthUser>>,
    Query(query): Query<ExportFeedbackQuery>,
) -> Result<Response, ApiError> {
    let auth = auth.ok_or_else(|| {
        ApiError::unauthorized(
            "missing_auth",
            "Authentication is required to export feedback",
        )
    })?;
    let profile_id = auth.profile_id.clone();
    ensure_profile_exists(&state, Some(&auth), &profile_id).await?;

    let export_type = FeedbackExportType::parse(&query.export_type).ok_or_else(|| {
        ApiError::bad_request_with_details(
            "invalid_feedback_export_type",
            "Unsupported feedback export type",
            serde_json::json!({
                "field": "type",
                "allowed_values": ["saved", "hidden", "bad_fit", "companies"],
                "received": query.export_type,
            }),
        )
    })?;

    let csv = match export_type {
        FeedbackExportType::Saved | FeedbackExportType::Hidden | FeedbackExportType::BadFit => {
            build_job_feedback_export_csv(&state, &profile_id, export_type).await?
        }
        FeedbackExportType::Companies => {
            build_company_feedback_export_csv(&state, &profile_id).await?
        }
    };

    let date = Utc::now().format("%Y-%m-%d");
    let filename = format!("feedback-{}-{date}.csv", export_type.as_str());

    Ok((
        [
            (header::CONTENT_TYPE, "text/csv; charset=utf-8".to_string()),
            (
                header::CONTENT_DISPOSITION,
                format!("attachment; filename=\"{filename}\""),
            ),
        ],
        csv,
    )
        .into_response())
}

pub async fn get_feedback_stats(
    State(state): State<AppState>,
    auth: Option<Extension<AuthUser>>,
    Query(query): Query<FeedbackStatsQuery>,
) -> Result<axum::Json<FeedbackStatsResponse>, ApiError> {
    let profile_id = resolve_action_profile_id(auth.as_deref(), query.profile_id)?;
    ensure_profile_exists(&state, auth.as_deref(), &profile_id).await?;

    let event_summary = state
        .user_events_service
        .summary_by_profile_since_days(&profile_id, 7)
        .await
        .map_err(|error| ApiError::from_repository(error, "user_event_query_failed"))?;

    let company_feedback = state
        .feedback_service
        .list_company_feedback(&profile_id)
        .await
        .map_err(|error| ApiError::from_repository(error, "feedback_query_failed"))?;

    Ok(axum::Json(FeedbackStatsResponse {
        saved_this_week_count: event_summary.save_count,
        hidden_this_week_count: event_summary.hide_count,
        bad_fit_this_week_count: event_summary.bad_fit_count,
        whitelisted_companies_count: company_feedback
            .iter()
            .filter(|c| c.status == CompanyFeedbackStatus::Whitelist)
            .count(),
        blacklisted_companies_count: company_feedback
            .iter()
            .filter(|c| c.status == CompanyFeedbackStatus::Blacklist)
            .count(),
    }))
}

pub async fn list_feedback_timeline(
    State(state): State<AppState>,
    auth: Option<Extension<AuthUser>>,
    Path(profile_id): Path<String>,
    Query(query): Query<FeedbackTimelineQuery>,
) -> Result<axum::Json<FeedbackTimelineResponse>, ApiError> {
    ensure_profile_exists(&state, auth.as_deref(), &profile_id).await?;

    let limit = query.limit.unwrap_or(20).clamp(1, 100);
    let offset = query.offset.unwrap_or(0);
    let page = state
        .user_events_service
        .list_feedback_history_by_profile(&profile_id, limit, offset)
        .await
        .map_err(|error| ApiError::from_repository(error, "user_event_query_failed"))?;

    let mut items = Vec::with_capacity(page.records.len());
    for event in page.records {
        items.push(feedback_timeline_item(&state, event).await?);
    }

    let next_offset = if offset + items.len() < page.total_count {
        Some(offset + items.len())
    } else {
        None
    };

    Ok(axum::Json(FeedbackTimelineResponse {
        profile_id,
        items,
        limit,
        offset,
        total_count: page.total_count,
        next_offset,
    }))
}

async fn feedback_timeline_item(
    state: &AppState,
    event: crate::domain::user_event::model::UserEventRecord,
) -> Result<FeedbackTimelineItemResponse, ApiError> {
    let (job_title, company_name) = match event.job_id.as_deref() {
        Some(job_id) => {
            let job_view = load_export_job_view(state, job_id).await?;
            if let Some(view) = job_view {
                (view.job.title, view.job.company_name)
            } else {
                (
                    "Unknown job".to_string(),
                    event
                        .company_name
                        .clone()
                        .unwrap_or_else(|| "Unknown company".to_string()),
                )
            }
        }
        None => (
            "Unknown job".to_string(),
            event
                .company_name
                .clone()
                .unwrap_or_else(|| "Unknown company".to_string()),
        ),
    };

    let reason = event
        .payload_json
        .as_ref()
        .and_then(|payload| payload.get("reason"))
        .and_then(|reason| reason.as_str())
        .map(str::trim)
        .filter(|reason| !reason.is_empty())
        .map(str::to_string);

    Ok(FeedbackTimelineItemResponse {
        id: event.id,
        event_type: event.event_type.as_str().to_string(),
        job_id: event.job_id,
        job_title,
        company_name,
        reason,
        created_at: event.created_at,
    })
}

async fn build_job_feedback_export_csv(
    state: &AppState,
    profile_id: &str,
    export_type: FeedbackExportType,
) -> Result<String, ApiError> {
    let records = state
        .feedback_service
        .list_job_feedback(profile_id)
        .await
        .map_err(|error| ApiError::from_repository(error, "feedback_query_failed"))?;

    let records: Vec<JobFeedbackRecord> = records
        .into_iter()
        .filter(|record| match export_type {
            FeedbackExportType::Saved => record.saved,
            FeedbackExportType::Hidden => record.hidden,
            FeedbackExportType::BadFit => record.bad_fit,
            FeedbackExportType::Companies => false,
        })
        .collect();

    let tags_by_job_id = if export_type == FeedbackExportType::BadFit {
        let job_ids: Vec<String> = records.iter().map(|record| record.job_id.clone()).collect();
        state
            .feedback_service
            .list_feedback_tags_for_jobs(profile_id, &job_ids)
            .await
            .map_err(|error| ApiError::from_repository(error, "feedback_tags_query_failed"))?
            .into_iter()
            .fold(HashMap::<String, Vec<String>>::new(), |mut acc, tag| {
                acc.entry(tag.job_id)
                    .or_default()
                    .push(tag.tag.as_str().to_string());
                acc
            })
    } else {
        HashMap::new()
    };

    let header = match export_type {
        FeedbackExportType::Saved => vec!["job_title", "company", "source", "saved_at", "url"],
        FeedbackExportType::Hidden => vec!["job_title", "company", "source", "hidden_at"],
        FeedbackExportType::BadFit => {
            vec!["job_title", "company", "source", "marked_at", "reason"]
        }
        FeedbackExportType::Companies => Vec::new(),
    }
    .into_iter()
    .map(str::to_string)
    .collect();
    let mut rows = vec![header];

    for record in records {
        let job_view = load_export_job_view(state, &record.job_id).await?;
        let job = job_view.as_ref().map(|view| &view.job);
        let variant = job_view
            .as_ref()
            .and_then(|view| view.primary_variant.as_ref());
        let title = job.map(|job| job.title.as_str()).unwrap_or("");
        let company = job.map(|job| job.company_name.as_str()).unwrap_or("");
        let source = variant.map(|variant| variant.source.as_str()).unwrap_or("");

        let row = match export_type {
            FeedbackExportType::Saved => vec![
                title.to_string(),
                company.to_string(),
                source.to_string(),
                record.updated_at,
                variant
                    .map(|variant| variant.source_url.clone())
                    .unwrap_or_default(),
            ],
            FeedbackExportType::Hidden => vec![
                title.to_string(),
                company.to_string(),
                source.to_string(),
                record.updated_at,
            ],
            FeedbackExportType::BadFit => vec![
                title.to_string(),
                company.to_string(),
                source.to_string(),
                record.updated_at,
                tags_by_job_id
                    .get(&record.job_id)
                    .map(|tags| tags.join(";"))
                    .unwrap_or_default(),
            ],
            FeedbackExportType::Companies => Vec::new(),
        };

        rows.push(row);
    }

    Ok(render_csv(rows))
}

async fn build_company_feedback_export_csv(
    state: &AppState,
    profile_id: &str,
) -> Result<String, ApiError> {
    let records = state
        .feedback_service
        .list_company_feedback(profile_id)
        .await
        .map_err(|error| ApiError::from_repository(error, "feedback_query_failed"))?;

    let mut rows = vec![vec![
        "company".to_string(),
        "status".to_string(),
        "notes".to_string(),
        "date".to_string(),
    ]];

    for record in records {
        rows.push(vec![
            record.company_name,
            record.status.as_str().to_string(),
            record.notes,
            record.updated_at,
        ]);
    }

    Ok(render_csv(rows))
}

async fn load_export_job_view(state: &AppState, job_id: &str) -> Result<Option<JobView>, ApiError> {
    state
        .jobs_service
        .get_view_by_id(job_id)
        .await
        .map_err(|error| ApiError::from_repository(error, "jobs_query_failed"))
}

fn render_csv(rows: Vec<Vec<String>>) -> String {
    let mut csv = rows
        .into_iter()
        .map(|row| {
            row.into_iter()
                .map(|field| escape_csv_field(&field))
                .collect::<Vec<_>>()
                .join(",")
        })
        .collect::<Vec<_>>()
        .join("\n");
    csv.push('\n');
    csv
}

fn escape_csv_field(value: &str) -> String {
    if value.contains([',', '"', '\n', '\r']) {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
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
    OptionalApiJson(payload): OptionalApiJson<MarkJobBadFitRequest>,
) -> Result<axum::Json<JobFeedbackResponse>, ApiError> {
    update_job_feedback(
        state,
        auth.as_deref(),
        profile_id,
        job_id,
        UserEventType::JobBadFit,
        JobFeedbackFlags {
            bad_fit: true,
            reason: payload.validate_reason(),
            ..JobFeedbackFlags::default()
        },
    )
    .await
}

pub async fn mark_job_bad_fit_by_query(
    State(state): State<AppState>,
    auth: Option<Extension<AuthUser>>,
    Query(query): Query<JobFeedbackActionQuery>,
    Path(job_id): Path<String>,
    OptionalApiJson(payload): OptionalApiJson<MarkJobBadFitRequest>,
) -> Result<axum::Json<JobFeedbackResponse>, ApiError> {
    let profile_id = resolve_action_profile_id(auth.as_deref(), query.profile_id)?;

    update_job_feedback(
        state,
        auth.as_deref(),
        profile_id,
        job_id,
        UserEventType::JobBadFit,
        JobFeedbackFlags {
            bad_fit: true,
            reason: payload.validate_reason(),
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

pub async fn undo_job_hide(
    State(state): State<AppState>,
    auth: Option<Extension<AuthUser>>,
    Path(job_id): Path<String>,
    Query(query): Query<JobFeedbackActionQuery>,
) -> Result<StatusCode, ApiError> {
    let profile_id = resolve_action_profile_id(auth.as_deref(), query.profile_id)?;

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

pub async fn undo_job_bad_fit(
    State(state): State<AppState>,
    auth: Option<Extension<AuthUser>>,
    Path(job_id): Path<String>,
    Query(query): Query<JobFeedbackActionQuery>,
) -> Result<StatusCode, ApiError> {
    let profile_id = resolve_action_profile_id(auth.as_deref(), query.profile_id)?;

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

pub async fn remove_company_blacklist_by_slug(
    State(state): State<AppState>,
    auth: Option<Extension<AuthUser>>,
    Path(company_slug): Path<String>,
    Query(query): Query<RemoveCompanyBlacklistBySlugQuery>,
) -> Result<StatusCode, ApiError> {
    let profile_id = auth
        .as_ref()
        .map(|user| user.profile_id.clone())
        .or(query.profile_id)
        .ok_or_else(|| {
            ApiError::bad_request_with_details(
                "missing_profile_id",
                "profile_id query parameter is required when authentication is disabled",
                serde_json::json!({ "field": "profile_id" }),
            )
        })?;

    remove_company_feedback_by_normalized_name(
        state,
        auth.as_deref(),
        profile_id,
        company_slug,
        CompanyFeedbackStatus::Blacklist,
    )
    .await
}

pub async fn update_company_feedback_notes_by_slug(
    State(state): State<AppState>,
    auth: Option<Extension<AuthUser>>,
    Path(company_slug): Path<String>,
    Query(query): Query<UpdateCompanyFeedbackBySlugQuery>,
    ApiJson(payload): ApiJson<UpdateCompanyFeedbackNotesRequest>,
) -> Result<axum::Json<CompanyFeedbackResponse>, ApiError> {
    let profile_id = auth
        .as_ref()
        .map(|user| user.profile_id.clone())
        .or(query.profile_id)
        .ok_or_else(|| {
            ApiError::bad_request_with_details(
                "missing_profile_id",
                "profile_id query parameter is required when authentication is disabled",
                serde_json::json!({ "field": "profile_id" }),
            )
        })?;

    update_company_feedback_notes_by_normalized_name(
        state,
        auth.as_deref(),
        profile_id,
        company_slug,
        payload,
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
    set_job_interest_rating_for_profile(state, auth.as_deref(), profile_id, job_id, payload).await
}

pub async fn patch_job_interest_rating_by_query(
    State(state): State<AppState>,
    auth: Option<Extension<AuthUser>>,
    Query(query): Query<JobFeedbackActionQuery>,
    Path(job_id): Path<String>,
    ApiJson(payload): ApiJson<SetInterestRatingRequest>,
) -> Result<axum::Json<JobFeedbackResponse>, ApiError> {
    let profile_id = resolve_action_profile_id(auth.as_deref(), query.profile_id)?;
    set_job_interest_rating_for_profile(state, auth.as_deref(), profile_id, job_id, payload).await
}

async fn set_job_interest_rating_for_profile(
    state: AppState,
    auth: Option<&AuthUser>,
    profile_id: String,
    job_id: String,
    payload: SetInterestRatingRequest,
) -> Result<axum::Json<JobFeedbackResponse>, ApiError> {
    ensure_profile_exists(&state, auth, &profile_id).await?;
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

pub async fn bulk_hide_jobs_by_company(
    State(state): State<AppState>,
    auth: Option<Extension<AuthUser>>,
    Query(query): Query<BulkHideJobsByCompanyQuery>,
    ApiJson(payload): ApiJson<BulkHideJobsByCompanyRequest>,
) -> Result<axum::Json<BulkFeedbackActionResponse>, ApiError> {
    let profile_id = auth
        .as_ref()
        .map(|user| user.profile_id.clone())
        .or(query.profile_id)
        .ok_or_else(|| {
            ApiError::bad_request_with_details(
                "missing_profile_id",
                "profile_id query parameter is required when authentication is disabled",
                serde_json::json!({ "field": "profile_id" }),
            )
        })?;

    ensure_profile_exists(&state, auth.as_deref(), &profile_id).await?;

    let company_name = payload.validate_company_name()?;
    let normalized_company_name = FeedbackService::normalize_company_name(&company_name);
    let affected_count = state
        .feedback_service
        .bulk_hide_jobs_by_company(&profile_id, &normalized_company_name)
        .await
        .map_err(|error| ApiError::from_repository(error, "feedback_bulk_hide_failed"))?;

    log_user_event_softly(
        &state,
        CreateUserEvent {
            profile_id,
            event_type: UserEventType::CompanyBlacklisted,
            job_id: None,
            company_name: Some(company_name),
            source: None,
            role_family: None,
            payload_json: Some(serde_json::json!({ "affected_count": affected_count })),
        },
    )
    .await;

    Ok(axum::Json(BulkFeedbackActionResponse { affected_count }))
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

fn resolve_action_profile_id(
    auth: Option<&AuthUser>,
    query_profile_id: Option<String>,
) -> Result<String, ApiError> {
    auth.map(|user| user.profile_id.clone())
        .or(query_profile_id)
        .ok_or_else(|| {
            ApiError::bad_request_with_details(
                "missing_profile_id",
                "profile_id query parameter is required when authentication is disabled",
                serde_json::json!({ "field": "profile_id" }),
            )
        })
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
    let event_payload = flags
        .reason
        .as_ref()
        .map(|reason| serde_json::json!({ "reason": reason }));

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
            payload_json: event_payload,
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

async fn remove_company_feedback_by_normalized_name(
    state: AppState,
    auth: Option<&AuthUser>,
    profile_id: String,
    company_slug: String,
    status: CompanyFeedbackStatus,
) -> Result<StatusCode, ApiError> {
    ensure_profile_exists(&state, auth, &profile_id).await?;

    let normalized_company_name = FeedbackService::normalize_company_name(&company_slug);
    if normalized_company_name.is_empty() {
        return Err(ApiError::bad_request_with_details(
            "invalid_company_slug",
            "company_slug must not be empty",
            serde_json::json!({ "field": "company_slug" }),
        ));
    }

    state
        .feedback_service
        .remove_company_feedback(&profile_id, &normalized_company_name, status)
        .await
        .map_err(|error| ApiError::from_repository(error, "feedback_write_failed"))?;

    Ok(StatusCode::NO_CONTENT)
}

async fn update_company_feedback_notes_by_normalized_name(
    state: AppState,
    auth: Option<&AuthUser>,
    profile_id: String,
    company_slug: String,
    payload: UpdateCompanyFeedbackNotesRequest,
) -> Result<axum::Json<CompanyFeedbackResponse>, ApiError> {
    ensure_profile_exists(&state, auth, &profile_id).await?;

    let normalized_company_name = FeedbackService::normalize_company_name(&company_slug);
    if normalized_company_name.is_empty() {
        return Err(ApiError::bad_request_with_details(
            "invalid_company_slug",
            "company_slug must not be empty",
            serde_json::json!({ "field": "company_slug" }),
        ));
    }

    let notes = payload.validate_notes()?;
    let feedback = state
        .feedback_service
        .update_company_feedback_notes(&profile_id, &normalized_company_name, &notes)
        .await
        .map_err(|error| ApiError::from_repository(error, "feedback_write_failed"))?
        .ok_or_else(|| {
            ApiError::not_found(
                "company_feedback_not_found",
                format!("Company feedback '{normalized_company_name}' was not found"),
            )
        })?;

    Ok(axum::Json(CompanyFeedbackResponse::from(feedback)))
}

pub async fn clear_all_hidden_jobs(
    State(state): State<AppState>,
    auth: Option<Extension<AuthUser>>,
    Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
    ensure_profile_exists(&state, auth.as_deref(), &id).await?;

    state
        .feedback_service
        .clear_all_hidden_jobs(&id)
        .await
        .map_err(|error| ApiError::from_repository(error, "hidden_feedback_clear_failed"))?;

    Ok(StatusCode::NO_CONTENT)
}
