use axum::extract::{Path, State};
use axum::http::StatusCode;

use crate::api::dto::feedback::{
    CompanyFeedbackResponse, FeedbackOverviewResponse, FeedbackSummary, JobFeedbackResponse,
    UpdateCompanyFeedbackRequest,
};
use crate::api::error::{ApiError, ApiJson};
use crate::api::routes::events::{load_job_event_metadata, log_user_event_softly};
use crate::domain::feedback::model::{CompanyFeedbackStatus, JobFeedbackFlags};
use crate::domain::user_event::model::{CreateUserEvent, UserEventType};
use crate::services::feedback::FeedbackService;
use crate::state::AppState;

pub async fn list_feedback(
    State(state): State<AppState>,
    Path(profile_id): Path<String>,
) -> Result<axum::Json<FeedbackOverviewResponse>, ApiError> {
    ensure_profile_exists(&state, &profile_id).await?;

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
    Path((profile_id, job_id)): Path<(String, String)>,
) -> Result<axum::Json<JobFeedbackResponse>, ApiError> {
    update_job_feedback(
        state,
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
    Path((profile_id, job_id)): Path<(String, String)>,
) -> Result<axum::Json<JobFeedbackResponse>, ApiError> {
    update_job_feedback(
        state,
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
    Path((profile_id, job_id)): Path<(String, String)>,
) -> Result<axum::Json<JobFeedbackResponse>, ApiError> {
    update_job_feedback(
        state,
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
    Path((profile_id, job_id)): Path<(String, String)>,
) -> Result<StatusCode, ApiError> {
    clear_job_feedback_flags(
        state,
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
    Path((profile_id, job_id)): Path<(String, String)>,
) -> Result<StatusCode, ApiError> {
    clear_job_feedback_flags(
        state,
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
    Path((profile_id, job_id)): Path<(String, String)>,
) -> Result<StatusCode, ApiError> {
    clear_job_feedback_flags(
        state,
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

async fn clear_job_feedback_flags(
    state: AppState,
    profile_id: String,
    job_id: String,
    event_type: UserEventType,
    flags: JobFeedbackFlags,
) -> Result<StatusCode, ApiError> {
    ensure_profile_exists(&state, &profile_id).await?;
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

pub async fn add_company_whitelist(
    State(state): State<AppState>,
    Path(profile_id): Path<String>,
    ApiJson(payload): ApiJson<UpdateCompanyFeedbackRequest>,
) -> Result<axum::Json<CompanyFeedbackResponse>, ApiError> {
    update_company_feedback(state, profile_id, payload, CompanyFeedbackStatus::Whitelist).await
}

pub async fn remove_company_whitelist(
    State(state): State<AppState>,
    Path(profile_id): Path<String>,
    ApiJson(payload): ApiJson<UpdateCompanyFeedbackRequest>,
) -> Result<StatusCode, ApiError> {
    remove_company_feedback(state, profile_id, payload, CompanyFeedbackStatus::Whitelist).await
}

pub async fn add_company_blacklist(
    State(state): State<AppState>,
    Path(profile_id): Path<String>,
    ApiJson(payload): ApiJson<UpdateCompanyFeedbackRequest>,
) -> Result<axum::Json<CompanyFeedbackResponse>, ApiError> {
    update_company_feedback(state, profile_id, payload, CompanyFeedbackStatus::Blacklist).await
}

pub async fn remove_company_blacklist(
    State(state): State<AppState>,
    Path(profile_id): Path<String>,
    ApiJson(payload): ApiJson<UpdateCompanyFeedbackRequest>,
) -> Result<StatusCode, ApiError> {
    remove_company_feedback(state, profile_id, payload, CompanyFeedbackStatus::Blacklist).await
}

async fn update_job_feedback(
    state: AppState,
    profile_id: String,
    job_id: String,
    event_type: UserEventType,
    flags: JobFeedbackFlags,
) -> Result<axum::Json<JobFeedbackResponse>, ApiError> {
    ensure_profile_exists(&state, &profile_id).await?;
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
    profile_id: String,
    payload: UpdateCompanyFeedbackRequest,
    status: CompanyFeedbackStatus,
) -> Result<axum::Json<CompanyFeedbackResponse>, ApiError> {
    ensure_profile_exists(&state, &profile_id).await?;

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
    profile_id: String,
    payload: UpdateCompanyFeedbackRequest,
    status: CompanyFeedbackStatus,
) -> Result<StatusCode, ApiError> {
    ensure_profile_exists(&state, &profile_id).await?;

    let company_name = payload.validate_company_name()?;
    let normalized_company_name = FeedbackService::normalize_company_name(&company_name);
    state
        .feedback_service
        .remove_company_feedback(&profile_id, &normalized_company_name, status)
        .await
        .map_err(|error| ApiError::from_repository(error, "feedback_write_failed"))?;

    Ok(StatusCode::NO_CONTENT)
}

pub(crate) async fn ensure_profile_exists(
    state: &AppState,
    profile_id: &str,
) -> Result<(), ApiError> {
    let Some(_) = state
        .profiles_service
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

#[cfg(test)]
mod tests {
    use axum::extract::{Path, State};
    use axum::response::IntoResponse;
    use axum::{Json, body};
    use serde_json::{Value, json};

    use crate::api::dto::feedback::UpdateCompanyFeedbackRequest;
    use crate::api::error::ApiJson;
    use crate::domain::feedback::model::{
        CompanyFeedbackRecord, CompanyFeedbackStatus, JobFeedbackRecord,
    };
    use crate::domain::job::model::{Job, JobLifecycleStage, JobSourceVariant, JobView};
    use crate::domain::profile::model::Profile;
    use crate::domain::user_event::model::UserEventType;
    use crate::services::applications::{ApplicationsService, ApplicationsServiceStub};
    use crate::services::feedback::{FeedbackService, FeedbackServiceStub};
    use crate::services::jobs::{JobsService, JobsServiceStub};
    use crate::services::profiles::{ProfilesService, ProfilesServiceStub};
    use crate::services::resumes::{ResumesService, ResumesServiceStub};
    use crate::services::user_events::{UserEventsService, UserEventsServiceStub};
    use crate::state::AppState;

    use super::{
        add_company_blacklist, hide_job, list_feedback, mark_job_bad_fit, save_job, unhide_job,
        unmark_job_bad_fit, unsave_job,
    };

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

    fn sample_job(job_id: &str, company_name: &str) -> Job {
        Job {
            id: job_id.to_string(),
            title: "Senior Backend Developer".to_string(),
            company_name: company_name.to_string(),
            remote_type: Some("remote".to_string()),
            seniority: Some("senior".to_string()),
            description_text: "Rust and Postgres".to_string(),
            salary_min: None,
            salary_max: None,
            salary_currency: None,
            posted_at: None,
            last_seen_at: "2026-04-14T00:00:00Z".to_string(),
            is_active: true,
        }
    }

    fn sample_job_view(job_id: &str, company_name: &str) -> JobView {
        JobView {
            job: sample_job(job_id, company_name),
            first_seen_at: "2026-04-12T00:00:00Z".to_string(),
            inactivated_at: None,
            reactivated_at: None,
            lifecycle_stage: JobLifecycleStage::Active,
            primary_variant: Some(JobSourceVariant {
                source: "djinni".to_string(),
                source_job_id: format!("djinni-{job_id}"),
                source_url: format!("https://djinni.co/jobs/{job_id}"),
                raw_payload: None,
                fetched_at: "2026-04-14T00:00:00Z".to_string(),
                last_seen_at: "2026-04-14T00:00:00Z".to_string(),
                is_active: true,
                inactivated_at: None,
            }),
        }
    }

    fn test_state() -> AppState {
        AppState::for_services(
            ProfilesService::for_tests(
                ProfilesServiceStub::default().with_profile(sample_profile()),
            ),
            JobsService::for_tests(
                JobsServiceStub::default()
                    .with_job(sample_job("job-1", "NovaLedger"))
                    .with_job_view(sample_job_view("job-1", "NovaLedger")),
            ),
            ApplicationsService::for_tests(ApplicationsServiceStub::default()),
            ResumesService::for_tests(ResumesServiceStub::default()),
        )
    }

    #[tokio::test]
    async fn save_and_bad_fit_persist_in_feedback_overview() {
        let state = test_state();

        let _ = save_job(
            State(state.clone()),
            Path(("profile-1".to_string(), "job-1".to_string())),
        )
        .await
        .expect("save should succeed");
        let _ = mark_job_bad_fit(
            State(state.clone()),
            Path(("profile-1".to_string(), "job-1".to_string())),
        )
        .await
        .expect("bad fit should succeed");

        let response = list_feedback(State(state), Path("profile-1".to_string()))
            .await
            .expect("listing feedback should succeed")
            .into_response();

        let body = body::to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("feedback body should be readable");
        let payload: Value =
            serde_json::from_slice(&body).expect("feedback body should be valid JSON");

        assert_eq!(payload["jobs"].as_array().map(Vec::len), Some(1));
        assert_eq!(payload["jobs"][0]["saved"], json!(true));
        assert_eq!(payload["jobs"][0]["bad_fit"], json!(true));
    }

    #[tokio::test]
    async fn add_company_blacklist_is_visible_in_feedback_overview() {
        let state = test_state();

        let _ = add_company_blacklist(
            State(state.clone()),
            Path("profile-1".to_string()),
            ApiJson(UpdateCompanyFeedbackRequest {
                company_name: "NovaLedger".to_string(),
            }),
        )
        .await
        .expect("blacklist should succeed");

        let response = list_feedback(State(state), Path("profile-1".to_string()))
            .await
            .expect("listing feedback should succeed")
            .into_response();

        let body = body::to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("feedback body should be readable");
        let payload: Value =
            serde_json::from_slice(&body).expect("feedback body should be valid JSON");

        assert_eq!(payload["companies"].as_array().map(Vec::len), Some(1));
        assert_eq!(payload["companies"][0]["status"], json!("blacklist"));
    }

    #[tokio::test]
    async fn list_feedback_uses_existing_stub_records() {
        let state = test_state().with_feedback_service(FeedbackService::for_tests(
            FeedbackServiceStub::default()
                .with_job_feedback(JobFeedbackRecord {
                    profile_id: "profile-1".to_string(),
                    job_id: "job-1".to_string(),
                    saved: true,
                    hidden: false,
                    bad_fit: false,
                    created_at: "2026-04-14T00:00:00Z".to_string(),
                    updated_at: "2026-04-14T00:00:00Z".to_string(),
                })
                .with_company_feedback(CompanyFeedbackRecord {
                    profile_id: "profile-1".to_string(),
                    company_name: "NovaLedger".to_string(),
                    normalized_company_name: "novaledger".to_string(),
                    status: CompanyFeedbackStatus::Blacklist,
                    created_at: "2026-04-14T00:00:00Z".to_string(),
                    updated_at: "2026-04-14T00:00:00Z".to_string(),
                }),
        ));

        let Json(response) = list_feedback(State(state), Path("profile-1".to_string()))
            .await
            .expect("listing feedback should succeed");

        assert_eq!(response.jobs.len(), 1);
        assert_eq!(response.companies.len(), 1);
        assert!(response.jobs[0].saved);
        assert_eq!(response.companies[0].status, "blacklist");
    }

    #[tokio::test]
    async fn unsave_job_clears_saved_flag() {
        let state = test_state().with_feedback_service(FeedbackService::for_tests(
            FeedbackServiceStub::default().with_job_feedback(JobFeedbackRecord {
                profile_id: "profile-1".to_string(),
                job_id: "job-1".to_string(),
                saved: true,
                hidden: false,
                bad_fit: false,
                created_at: "2026-04-14T00:00:00Z".to_string(),
                updated_at: "2026-04-14T00:00:00Z".to_string(),
            }),
        ));

        unsave_job(
            State(state.clone()),
            Path(("profile-1".to_string(), "job-1".to_string())),
        )
        .await
        .expect("unsave should succeed");

        let Json(overview) = list_feedback(State(state), Path("profile-1".to_string()))
            .await
            .expect("listing feedback should succeed");

        assert_eq!(overview.jobs.len(), 1);
        assert!(
            !overview.jobs[0].saved,
            "saved should be cleared after unsave"
        );
    }

    #[tokio::test]
    async fn unhide_job_clears_hidden_flag() {
        let state = test_state().with_feedback_service(FeedbackService::for_tests(
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

        unhide_job(
            State(state.clone()),
            Path(("profile-1".to_string(), "job-1".to_string())),
        )
        .await
        .expect("unhide should succeed");

        let Json(overview) = list_feedback(State(state), Path("profile-1".to_string()))
            .await
            .expect("listing feedback should succeed");

        assert_eq!(overview.jobs.len(), 1);
        assert!(
            !overview.jobs[0].hidden,
            "hidden should be cleared after unhide"
        );
    }

    #[tokio::test]
    async fn unmark_bad_fit_clears_bad_fit_flag() {
        let state = test_state().with_feedback_service(FeedbackService::for_tests(
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

        unmark_job_bad_fit(
            State(state.clone()),
            Path(("profile-1".to_string(), "job-1".to_string())),
        )
        .await
        .expect("unmark bad fit should succeed");

        let Json(overview) = list_feedback(State(state), Path("profile-1".to_string()))
            .await
            .expect("listing feedback should succeed");

        assert_eq!(overview.jobs.len(), 1);
        assert!(
            !overview.jobs[0].bad_fit,
            "bad_fit should be cleared after unmark"
        );
    }

    #[tokio::test]
    async fn undo_on_nonexistent_feedback_succeeds_idempotently() {
        // Clearing a flag when no feedback row exists should return 204, not an error.
        let state = test_state();

        let result = unsave_job(
            State(state),
            Path(("profile-1".to_string(), "job-1".to_string())),
        )
        .await;

        assert!(
            result.is_ok(),
            "unsave on a job with no feedback should succeed"
        );
    }

    #[tokio::test]
    async fn undo_preserves_other_flags() {
        // Unsaving should not clear hidden or bad_fit flags.
        let state = test_state().with_feedback_service(FeedbackService::for_tests(
            FeedbackServiceStub::default().with_job_feedback(JobFeedbackRecord {
                profile_id: "profile-1".to_string(),
                job_id: "job-1".to_string(),
                saved: true,
                hidden: true,
                bad_fit: true,
                created_at: "2026-04-14T00:00:00Z".to_string(),
                updated_at: "2026-04-14T00:00:00Z".to_string(),
            }),
        ));

        unsave_job(
            State(state.clone()),
            Path(("profile-1".to_string(), "job-1".to_string())),
        )
        .await
        .expect("unsave should succeed");

        let Json(overview) = list_feedback(State(state), Path("profile-1".to_string()))
            .await
            .expect("listing feedback should succeed");

        assert!(!overview.jobs[0].saved, "saved should be cleared");
        assert!(overview.jobs[0].hidden, "hidden should be untouched");
        assert!(overview.jobs[0].bad_fit, "bad_fit should be untouched");
    }

    #[tokio::test]
    async fn feedback_overview_summary_counts_are_correct() {
        // Set up: one job that is saved + bad_fit, one whitelisted company, one blacklisted.
        let state = test_state().with_feedback_service(FeedbackService::for_tests(
            FeedbackServiceStub::default()
                .with_job_feedback(JobFeedbackRecord {
                    profile_id: "profile-1".to_string(),
                    job_id: "job-1".to_string(),
                    saved: true,
                    hidden: false,
                    bad_fit: true,
                    created_at: "2026-04-14T00:00:00Z".to_string(),
                    updated_at: "2026-04-14T00:00:00Z".to_string(),
                })
                .with_company_feedback(CompanyFeedbackRecord {
                    profile_id: "profile-1".to_string(),
                    company_name: "GoodCorp".to_string(),
                    normalized_company_name: "goodcorp".to_string(),
                    status: CompanyFeedbackStatus::Whitelist,
                    created_at: "2026-04-14T00:00:00Z".to_string(),
                    updated_at: "2026-04-14T00:00:00Z".to_string(),
                })
                .with_company_feedback(CompanyFeedbackRecord {
                    profile_id: "profile-1".to_string(),
                    company_name: "BadCorp".to_string(),
                    normalized_company_name: "badcorp".to_string(),
                    status: CompanyFeedbackStatus::Blacklist,
                    created_at: "2026-04-14T00:00:00Z".to_string(),
                    updated_at: "2026-04-14T00:00:00Z".to_string(),
                }),
        ));

        let Json(overview) = list_feedback(State(state), Path("profile-1".to_string()))
            .await
            .expect("listing feedback should succeed");

        assert_eq!(overview.summary.saved_jobs_count, 1);
        assert_eq!(overview.summary.hidden_jobs_count, 0);
        assert_eq!(overview.summary.bad_fit_jobs_count, 1);
        assert_eq!(overview.summary.whitelisted_companies_count, 1);
        assert_eq!(overview.summary.blacklisted_companies_count, 1);
    }

    #[tokio::test]
    async fn feedback_actions_create_expected_user_events() {
        let state = test_state();

        let _ = save_job(
            State(state.clone()),
            Path(("profile-1".to_string(), "job-1".to_string())),
        )
        .await
        .expect("save should succeed");
        unsave_job(
            State(state.clone()),
            Path(("profile-1".to_string(), "job-1".to_string())),
        )
        .await
        .expect("unsave should succeed");
        let _ = hide_job(
            State(state.clone()),
            Path(("profile-1".to_string(), "job-1".to_string())),
        )
        .await
        .expect("hide should succeed");
        unhide_job(
            State(state.clone()),
            Path(("profile-1".to_string(), "job-1".to_string())),
        )
        .await
        .expect("unhide should succeed");
        let _ = mark_job_bad_fit(
            State(state.clone()),
            Path(("profile-1".to_string(), "job-1".to_string())),
        )
        .await
        .expect("bad fit should succeed");
        unmark_job_bad_fit(
            State(state.clone()),
            Path(("profile-1".to_string(), "job-1".to_string())),
        )
        .await
        .expect("remove bad fit should succeed");

        let events = state
            .user_events_service
            .list_by_profile("profile-1")
            .await
            .expect("events should be queryable");
        let event_types: Vec<UserEventType> =
            events.into_iter().map(|event| event.event_type).collect();

        assert!(event_types.contains(&UserEventType::JobSaved));
        assert!(event_types.contains(&UserEventType::JobUnsaved));
        assert!(event_types.contains(&UserEventType::JobHidden));
        assert!(event_types.contains(&UserEventType::JobUnhidden));
        assert!(event_types.contains(&UserEventType::JobBadFit));
        assert!(event_types.contains(&UserEventType::JobBadFitRemoved));
    }

    #[tokio::test]
    async fn company_blacklist_creates_user_event() {
        let state = test_state();

        let _ = add_company_blacklist(
            State(state.clone()),
            Path("profile-1".to_string()),
            ApiJson(UpdateCompanyFeedbackRequest {
                company_name: "NovaLedger".to_string(),
            }),
        )
        .await
        .expect("blacklist should succeed");

        let events = state
            .user_events_service
            .list_by_profile("profile-1")
            .await
            .expect("events should be queryable");

        assert!(
            events
                .iter()
                .any(|event| event.event_type == UserEventType::CompanyBlacklisted),
            "company blacklist action should emit an immutable user event"
        );
    }

    #[tokio::test]
    async fn save_job_still_succeeds_when_event_logging_fails_softly() {
        let state = test_state().with_user_events_service(UserEventsService::for_tests(
            UserEventsServiceStub::default().with_database_disabled(),
        ));

        let result = save_job(
            State(state.clone()),
            Path(("profile-1".to_string(), "job-1".to_string())),
        )
        .await;

        assert!(
            result.is_ok(),
            "feedback write should not fail when event logging is unavailable"
        );
    }
}
