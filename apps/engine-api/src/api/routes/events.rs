use axum::extract::{Path, State};
use serde_json::json;
use tracing::warn;

use crate::api::dto::events::{LogUserEventRequest, UserEventResponse, UserEventSummaryResponse};
use crate::api::error::{ApiError, ApiJson};
use crate::api::routes::feedback::ensure_profile_exists;
use crate::domain::user_event::model::{CreateUserEvent, UserEventType};
use crate::state::AppState;

pub async fn log_user_event(
    State(state): State<AppState>,
    Path(profile_id): Path<String>,
    ApiJson(payload): ApiJson<LogUserEventRequest>,
) -> Result<(axum::http::StatusCode, axum::Json<UserEventResponse>), ApiError> {
    ensure_profile_exists(&state, &profile_id).await?;

    let mut event = payload.validate(profile_id)?;
    if let Some(job_id) = event.job_id.as_deref() {
        let metadata = load_job_event_metadata(&state, job_id).await?;
        if event.company_name.is_none() {
            event.company_name = metadata.company_name;
        }
        if event.source.is_none() {
            event.source = metadata.source;
        }
        if event.role_family.is_none() {
            event.role_family = metadata.role_family;
        }
    }

    let event = state
        .user_events_service
        .log_event(event)
        .await
        .map_err(|error| ApiError::from_repository(error, "user_event_write_failed"))?;
    maybe_record_labelable_outcome(
        &state,
        &event.profile_id,
        event.job_id.as_deref(),
        event.event_type,
    )
    .await;

    Ok((
        axum::http::StatusCode::CREATED,
        axum::Json(UserEventResponse::from(event)),
    ))
}

pub async fn get_user_event_summary(
    State(state): State<AppState>,
    Path(profile_id): Path<String>,
) -> Result<axum::Json<UserEventSummaryResponse>, ApiError> {
    ensure_profile_exists(&state, &profile_id).await?;

    let summary = state
        .user_events_service
        .summary_by_profile(&profile_id)
        .await
        .map_err(|error| ApiError::from_repository(error, "user_event_query_failed"))?;

    Ok(axum::Json(UserEventSummaryResponse::from_summary(
        profile_id, summary,
    )))
}

#[derive(Clone, Debug, Default)]
pub(crate) struct JobEventMetadata {
    pub company_name: Option<String>,
    pub source: Option<String>,
    pub role_family: Option<String>,
}

pub(crate) async fn load_job_event_metadata(
    state: &AppState,
    job_id: &str,
) -> Result<JobEventMetadata, ApiError> {
    let maybe_job_view = state
        .jobs_service
        .get_view_by_id(job_id)
        .await
        .map_err(|error| ApiError::from_repository(error, "jobs_query_failed"))?;

    if let Some(job_view) = maybe_job_view {
        let source = job_view
            .primary_variant
            .as_ref()
            .map(|variant| variant.source.clone());
        let role_family = state.search_ranking.infer_role_family(&job_view);

        return Ok(JobEventMetadata {
            company_name: Some(job_view.job.company_name),
            source,
            role_family,
        });
    }

    let Some(job) = state
        .jobs_service
        .get_by_id(job_id)
        .await
        .map_err(|error| ApiError::from_repository(error, "jobs_query_failed"))?
    else {
        return Err(ApiError::not_found(
            "job_not_found",
            format!("Job '{job_id}' was not found"),
        ));
    };

    let role_family = state.search_ranking.infer_role_family_for_job(&job);

    Ok(JobEventMetadata {
        company_name: Some(job.company_name),
        source: None,
        role_family,
    })
}

pub(crate) async fn log_user_event_softly(state: &AppState, event: CreateUserEvent) {
    let details = json!({
        "profile_id": event.profile_id,
        "event_type": event.event_type.as_str(),
        "job_id": event.job_id,
    });

    match state.user_events_service.log_event(event).await {
        Ok(record) => {
            maybe_record_labelable_outcome(
                state,
                &record.profile_id,
                record.job_id.as_deref(),
                record.event_type,
            )
            .await;
        }
        Err(error) => {
            warn!(error = %error, details = %details, "failed to log user event");
        }
    }
}

pub(crate) async fn record_labelable_job_softly(state: &AppState, profile_id: &str, job_id: &str) {
    if let Err(error) = state
        .profile_ml_state
        .record_labelable_job(profile_id, job_id)
        .await
    {
        warn!(
            error = %error,
            profile_id,
            job_id,
            "failed to record labelable job for ML state"
        );
    }
}

async fn maybe_record_labelable_outcome(
    state: &AppState,
    profile_id: &str,
    job_id: Option<&str>,
    event_type: UserEventType,
) {
    let Some(job_id) = job_id.filter(|value| !value.trim().is_empty()) else {
        return;
    };

    if !matches!(
        event_type,
        UserEventType::JobOpened
            | UserEventType::JobSaved
            | UserEventType::JobHidden
            | UserEventType::JobBadFit
            | UserEventType::ApplicationCreated
    ) {
        return;
    }

    if let Err(error) = state
        .profile_ml_state
        .record_labelable_job(profile_id, job_id)
        .await
    {
        warn!(
            error = %error,
            profile_id,
            job_id,
            event_type = event_type.as_str(),
            "failed to record labelable outcome for ML state"
        );
    }
}

#[cfg(test)]
mod tests {
    use axum::Json;
    use axum::extract::{Path, State};
    use serde_json::json;

    use crate::api::dto::events::LogUserEventRequest;
    use crate::api::error::ApiJson;
    use crate::domain::job::model::{Job, JobLifecycleStage, JobSourceVariant, JobView};
    use crate::domain::profile::ml::ProfileMlState;
    use crate::domain::profile::model::Profile;
    use crate::domain::user_event::model::{UserEventRecord, UserEventType};
    use crate::services::applications::{ApplicationsService, ApplicationsServiceStub};
    use crate::services::jobs::{JobsService, JobsServiceStub};
    use crate::services::profile_ml_state::{ProfileMlStateService, ProfileMlStateServiceStub};
    use crate::services::profiles::{ProfilesService, ProfilesServiceStub};
    use crate::services::resumes::{ResumesService, ResumesServiceStub};
    use crate::services::user_events::{UserEventsService, UserEventsServiceStub};
    use crate::state::AppState;

    use super::{get_user_event_summary, log_user_event};

    fn sample_profile() -> Profile {
        Profile {
            id: "profile-1".to_string(),
            name: "Jane Doe".to_string(),
            email: "jane@example.com".to_string(),
            location: Some("Kyiv".to_string()),
            raw_text: "Senior backend engineer".to_string(),
            analysis: None,
            years_of_experience: None,
            salary_min: None,
            salary_max: None,
            salary_currency: "USD".to_string(),
            languages: vec![],
            preferred_work_mode: None,
            search_preferences: None,
            created_at: "2026-04-14T00:00:00Z".to_string(),
            updated_at: "2026-04-14T00:00:00Z".to_string(),
            skills_updated_at: None,
        }
    }

    fn sample_job(job_id: &str) -> Job {
        Job {
            id: job_id.to_string(),
            title: "Senior Backend Developer".to_string(),
            company_name: "NovaLedger".to_string(),
            location: None,
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

    fn sample_job_view(job_id: &str) -> JobView {
        JobView {
            job: sample_job(job_id),
            first_seen_at: "2026-04-10T00:00:00Z".to_string(),
            inactivated_at: None,
            reactivated_at: None,
            lifecycle_stage: JobLifecycleStage::Active,
            primary_variant: Some(JobSourceVariant {
                source: "djinni".to_string(),
                source_job_id: "djinni-1".to_string(),
                source_url: "https://djinni.co/jobs/1".to_string(),
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
                    .with_job(sample_job("job-1"))
                    .with_job_view(sample_job_view("job-1")),
            ),
            ApplicationsService::for_tests(ApplicationsServiceStub::default()),
            ResumesService::for_tests(ResumesServiceStub::default()),
        )
    }

    #[tokio::test]
    async fn explicit_event_logging_persists_payload_and_job_metadata() {
        let state = test_state();

        let (_, Json(response)) = log_user_event(
            State(state.clone()),
            Path("profile-1".to_string()),
            ApiJson(LogUserEventRequest {
                event_type: "fit_explanation_requested".to_string(),
                job_id: Some("job-1".to_string()),
                company_name: None,
                source: None,
                role_family: None,
                payload_json: Some(json!({
                    "surface": "profile_page",
                    "deterministic_fit_score": 81
                })),
            }),
        )
        .await
        .expect("event logging should succeed");

        assert_eq!(response.event_type, "fit_explanation_requested");
        assert_eq!(response.company_name.as_deref(), Some("NovaLedger"));
        assert_eq!(response.source.as_deref(), Some("djinni"));
        assert_eq!(
            response.payload_json,
            Some(json!({
                "surface": "profile_page",
                "deterministic_fit_score": 81
            }))
        );

        let events = state
            .user_events_service
            .list_by_profile("profile-1")
            .await
            .expect("events should be queryable");
        assert_eq!(events.len(), 1);
    }

    #[tokio::test]
    async fn event_summary_counts_learning_signals() {
        let state = test_state().with_user_events_service(UserEventsService::for_tests(
            UserEventsServiceStub::default()
                .with_event(UserEventRecord {
                    id: "evt-1".to_string(),
                    profile_id: "profile-1".to_string(),
                    event_type: UserEventType::JobSaved,
                    job_id: Some("job-1".to_string()),
                    company_name: Some("NovaLedger".to_string()),
                    source: Some("djinni".to_string()),
                    role_family: None,
                    payload_json: None,
                    created_at: "2026-04-15T00:00:00Z".to_string(),
                })
                .with_event(UserEventRecord {
                    id: "evt-2".to_string(),
                    profile_id: "profile-1".to_string(),
                    event_type: UserEventType::SearchRun,
                    job_id: None,
                    company_name: None,
                    source: None,
                    role_family: Some("engineering".to_string()),
                    payload_json: Some(json!({ "returned_jobs": 20 })),
                    created_at: "2026-04-15T00:00:01Z".to_string(),
                })
                .with_event(UserEventRecord {
                    id: "evt-3".to_string(),
                    profile_id: "profile-1".to_string(),
                    event_type: UserEventType::InterviewPrepRequested,
                    job_id: Some("job-1".to_string()),
                    company_name: Some("NovaLedger".to_string()),
                    source: Some("djinni".to_string()),
                    role_family: None,
                    payload_json: None,
                    created_at: "2026-04-15T00:00:02Z".to_string(),
                }),
        ));

        let Json(summary) = get_user_event_summary(State(state), Path("profile-1".to_string()))
            .await
            .expect("summary should succeed");

        assert_eq!(summary.save_count, 1);
        assert_eq!(summary.search_run_count, 1);
        assert_eq!(summary.interview_prep_requested_count, 1);
        assert_eq!(summary.hide_count, 0);
    }

    #[tokio::test]
    async fn labelable_events_increment_ml_counter_only_once_per_job() {
        let state = test_state().with_profile_ml_state_service(ProfileMlStateService::for_tests(
            ProfileMlStateServiceStub::default().with_state(ProfileMlState {
                profile_id: "profile-1".to_string(),
                ..ProfileMlState::default()
            }),
        ));

        let request = || LogUserEventRequest {
            event_type: "job_opened".to_string(),
            job_id: Some("job-1".to_string()),
            company_name: None,
            source: None,
            role_family: None,
            payload_json: None,
        };

        let _ = log_user_event(
            State(state.clone()),
            Path("profile-1".to_string()),
            ApiJson(request()),
        )
        .await
        .expect("first event should succeed");
        let _ = log_user_event(
            State(state.clone()),
            Path("profile-1".to_string()),
            ApiJson(request()),
        )
        .await
        .expect("duplicate event should still succeed");

        let ml_state = state
            .profile_ml_state
            .get_by_profile_id("profile-1")
            .await
            .expect("ML state should be queryable")
            .expect("ML state should exist");
        assert_eq!(ml_state.examples_since_retrain, 1);
    }
}
