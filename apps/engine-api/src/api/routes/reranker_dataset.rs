use axum::Extension;
use axum::extract::{Path, State};

use crate::api::dto::reranker_dataset::OutcomeDatasetResponse;
use crate::api::error::ApiError;
use crate::api::middleware::auth::{AuthUser, check_profile_ownership};
use crate::api::routes::jobs::load_feedback_state;
use crate::services::behavior::BehaviorService;
use crate::services::funnel::FunnelService;
use crate::services::outcome_dataset::{
    OutcomeDatasetError, OutcomeDatasetService, application_ids_by_job_id, outcome_job_ids,
};
use crate::state::AppState;

pub async fn get_reranker_dataset(
    State(state): State<AppState>,
    auth: Option<Extension<AuthUser>>,
    Path(profile_id): Path<String>,
) -> Result<axum::Json<OutcomeDatasetResponse>, ApiError> {
    check_profile_ownership(auth.as_deref(), &profile_id)?;
    let Some(profile) = state
        .profile_records
        .get_by_id(&profile_id)
        .await
        .map_err(|error| ApiError::from_repository(error, "profiles_query_failed"))?
    else {
        return Err(ApiError::not_found(
            "profile_not_found",
            format!("Profile '{profile_id}' was not found"),
        ));
    };

    let events = state
        .user_events_service
        .list_by_profile(&profile_id)
        .await
        .map_err(|error| ApiError::from_repository(error, "user_event_query_failed"))?;
    let feedback = state
        .feedback_service
        .list_job_feedback(&profile_id)
        .await
        .map_err(|error| ApiError::from_repository(error, "feedback_query_failed"))?;
    let feedback_updated_at_by_job_id = feedback
        .iter()
        .map(|record| (record.job_id.clone(), record.updated_at.clone()))
        .collect();
    let feedback_job_ids = feedback
        .iter()
        .map(|record| record.job_id.clone())
        .collect::<Vec<_>>();
    let job_ids = outcome_job_ids(&events, &feedback_job_ids);
    let mut jobs = Vec::new();

    for job_id in job_ids {
        if let Some(job) = state
            .jobs_service
            .get_view_by_id(&job_id)
            .await
            .map_err(|error| ApiError::from_repository(error, "jobs_query_failed"))?
        {
            jobs.push(job);
        }
    }

    let feedback_states = load_feedback_state(&state, Some(&profile_id), &jobs).await?;
    let jobs_with_feedback = jobs
        .into_iter()
        .zip(feedback_states.into_iter())
        .collect::<Vec<_>>();
    let behavior = BehaviorService::new().build_aggregates(events.iter());
    let funnel = FunnelService::new().build_aggregates(events.iter());
    let applications_by_job_id = load_profile_applications_by_job_id(&state, &events).await?;
    let dataset = OutcomeDatasetService::new()
        .build(
            &profile,
            &events,
            jobs_with_feedback,
            &applications_by_job_id,
            &feedback_updated_at_by_job_id,
            &state.search_ranking,
            &behavior,
            &funnel,
        )
        .map_err(outcome_dataset_error)?;

    Ok(axum::Json(OutcomeDatasetResponse::from(dataset)))
}

async fn load_profile_applications_by_job_id(
    state: &AppState,
    events: &[crate::domain::user_event::model::UserEventRecord],
) -> Result<
    std::collections::BTreeMap<String, crate::domain::application::model::Application>,
    ApiError,
> {
    let mut applications_by_job_id = std::collections::BTreeMap::new();

    for (job_id, application_id) in application_ids_by_job_id(events) {
        if let Some(application) = state
            .applications_service
            .get_by_id(&application_id)
            .await
            .map_err(|error| ApiError::from_repository(error, "applications_query_failed"))?
        {
            applications_by_job_id.insert(job_id, application);
        }
    }

    Ok(applications_by_job_id)
}

fn outcome_dataset_error(error: OutcomeDatasetError) -> ApiError {
    match error {
        OutcomeDatasetError::ProfileAnalysisRequired => ApiError::bad_request(
            "profile_analysis_required",
            "Profile analysis is required before exporting a reranker dataset",
        ),
    }
}

#[cfg(test)]
mod tests {
    use axum::body;
    use axum::extract::{Path, State};
    use axum::response::IntoResponse;
    use serde_json::{Value, json};

    use crate::domain::feedback::model::JobFeedbackRecord;
    use crate::domain::job::model::{Job, JobLifecycleStage, JobSourceVariant, JobView};
    use crate::domain::profile::model::{Profile, ProfileAnalysis};
    use crate::domain::role::RoleId;
    use crate::domain::user_event::model::{UserEventRecord, UserEventType};
    use crate::services::applications::{ApplicationsService, ApplicationsServiceStub};
    use crate::services::feedback::{FeedbackService, FeedbackServiceStub};
    use crate::services::jobs::{JobsService, JobsServiceStub};
    use crate::services::profiles::{ProfilesService, ProfilesServiceStub};
    use crate::services::resumes::{ResumesService, ResumesServiceStub};
    use crate::services::user_events::{UserEventsService, UserEventsServiceStub};
    use crate::state::AppState;

    use super::get_reranker_dataset;

    fn sample_profile(analysis: Option<ProfileAnalysis>) -> Profile {
        Profile {
            id: "profile-1".to_string(),
            name: "Jane Doe".to_string(),
            email: "jane@example.com".to_string(),
            location: Some("Kyiv".to_string()),
            raw_text: "Senior backend engineer with Rust and Postgres".to_string(),
            analysis,
            years_of_experience: None,
            salary_min: None,
            salary_max: None,
            salary_currency: "USD".to_string(),
            languages: vec![],
            preferred_locations: vec![],
            work_mode_preference: "any".to_string(),
            preferred_language: None,
            search_preferences: None,
            created_at: "2026-04-14T00:00:00Z".to_string(),
            updated_at: "2026-04-14T00:00:00Z".to_string(),
            skills_updated_at: None,
            portfolio_url: None,
            github_url: None,
            linkedin_url: None,
        }
    }

    fn sample_analysis() -> ProfileAnalysis {
        ProfileAnalysis {
            summary: "Senior backend engineer".to_string(),
            primary_role: RoleId::BackendEngineer,
            seniority: "senior".to_string(),
            skills: vec!["Rust".to_string(), "Postgres".to_string()],
            keywords: vec!["backend".to_string(), "platform".to_string()],
        }
    }

    fn sample_job_view(job_id: &str, title: &str, source: &str) -> JobView {
        JobView {
            job: Job {
                id: job_id.to_string(),
                title: title.to_string(),
                company_name: "NovaLedger".to_string(),
                location: None,
                remote_type: Some("remote".to_string()),
                seniority: Some("senior".to_string()),
                description_text: "Build backend Rust services on Postgres".to_string(),
                salary_min: None,
                salary_max: None,
                salary_currency: None,
                language: None,
                posted_at: Some("2026-04-14T00:00:00Z".to_string()),
                last_seen_at: "2026-04-15T00:00:00Z".to_string(),
                is_active: true,
            },
            first_seen_at: "2026-04-14T00:00:00Z".to_string(),
            inactivated_at: None,
            reactivated_at: None,
            lifecycle_stage: JobLifecycleStage::Active,
            primary_variant: Some(JobSourceVariant {
                source: source.to_string(),
                source_job_id: format!("{source}-{job_id}"),
                source_url: format!("https://example.com/{job_id}"),
                raw_payload: None,
                fetched_at: "2026-04-14T00:00:00Z".to_string(),
                last_seen_at: "2026-04-15T00:00:00Z".to_string(),
                is_active: true,
                inactivated_at: None,
            }),
        }
    }

    fn event(
        id: &str,
        profile_id: &str,
        job_id: &str,
        event_type: UserEventType,
    ) -> UserEventRecord {
        UserEventRecord {
            id: id.to_string(),
            profile_id: profile_id.to_string(),
            event_type,
            job_id: Some(job_id.to_string()),
            company_name: Some("NovaLedger".to_string()),
            source: Some("djinni".to_string()),
            role_family: Some("engineering".to_string()),
            payload_json: None,
            created_at: "2026-04-15T00:00:00Z".to_string(),
        }
    }

    fn base_state(profile: Profile) -> AppState {
        AppState::for_services(
            ProfilesService::for_tests(ProfilesServiceStub::default().with_profile(profile)),
            JobsService::for_tests(
                JobsServiceStub::default()
                    .with_job_view(sample_job_view(
                        "job-applied",
                        "Senior Backend Engineer",
                        "djinni",
                    ))
                    .with_job_view(sample_job_view("job-hidden", "Support Engineer", "work_ua"))
                    .with_job_view(sample_job_view(
                        "job-saved",
                        "Rust Platform Engineer",
                        "djinni",
                    ))
                    .with_job_view(sample_job_view("job-viewed", "Backend Engineer", "djinni")),
            ),
            ApplicationsService::for_tests(ApplicationsServiceStub::default()),
            ResumesService::for_tests(ResumesServiceStub::default()),
        )
    }

    #[tokio::test]
    async fn reranker_dataset_exports_profile_scoped_labeled_examples() {
        let state = base_state(sample_profile(Some(sample_analysis())))
            .with_feedback_service(FeedbackService::for_tests(
                FeedbackServiceStub::default().with_job_feedback(JobFeedbackRecord {
                    profile_id: "profile-1".to_string(),
                    job_id: "job-saved".to_string(),
                    saved: true,
                    hidden: false,
                    bad_fit: false,
                    salary_signal: None,
                    interest_rating: None,
                    work_mode_signal: None,
                    legitimacy_signal: None,
                    created_at: "2026-04-14T00:00:00Z".to_string(),
                    updated_at: "2026-04-14T00:00:00Z".to_string(),
                }),
            ))
            .with_user_events_service(UserEventsService::for_tests(
                UserEventsServiceStub::default()
                    .with_event(event(
                        "evt-1",
                        "profile-1",
                        "job-applied",
                        UserEventType::ApplicationCreated,
                    ))
                    .with_event(event(
                        "evt-2",
                        "profile-1",
                        "job-hidden",
                        UserEventType::JobHidden,
                    ))
                    .with_event(event(
                        "evt-3",
                        "other-profile",
                        "job-saved",
                        UserEventType::ApplicationCreated,
                    ))
                    .with_event(event(
                        "evt-4",
                        "profile-1",
                        "job-viewed",
                        UserEventType::JobOpened,
                    )),
            ));

        let response = get_reranker_dataset(State(state), None, Path("profile-1".to_string()))
            .await
            .expect("dataset export should succeed")
            .into_response();
        let body = body::to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("response body should be readable");
        let payload: Value =
            serde_json::from_slice(&body).expect("response body should be valid JSON");

        assert_eq!(payload["profile_id"], json!("profile-1"));
        assert_eq!(payload["label_policy_version"], json!("outcome_label_v3"));
        assert_eq!(payload["examples"].as_array().map(Vec::len), Some(4));
        assert_eq!(payload["examples"][0]["job_id"], json!("job-applied"));
        assert_eq!(payload["examples"][0]["label"], json!("positive"));
        assert_eq!(payload["examples"][0]["label_score"], json!(2));
        assert_eq!(
            payload["examples"][0]["label_observed_at"],
            json!("2026-04-15T00:00:00Z")
        );
        assert_eq!(payload["examples"][0]["signals"]["applied"], json!(true));
        assert_eq!(
            payload["examples"][0]["signals"]["applied_event_count"],
            json!(1)
        );
        assert!(payload["examples"][0]["signals"]["outcome"].is_null());
        assert_eq!(payload["examples"][1]["job_id"], json!("job-hidden"));
        assert_eq!(payload["examples"][1]["label"], json!("negative"));
        assert_eq!(payload["examples"][1]["signals"]["dismissed"], json!(true));
        assert_eq!(
            payload["examples"][1]["label_reasons"],
            json!(["dismissed", "hidden"])
        );
        assert_eq!(payload["examples"][2]["job_id"], json!("job-saved"));
        assert_eq!(payload["examples"][2]["label"], json!("medium"));
        assert_eq!(
            payload["examples"][2]["label_observed_at"],
            json!("2026-04-14T00:00:00Z")
        );
        assert_eq!(
            payload["examples"][2]["signals"]["explicit_feedback"],
            json!(true)
        );
        assert_eq!(payload["examples"][2]["signals"]["saved"], json!(true));
        assert!(payload["examples"][2]["signals"]["interest_rating"].is_null());
        assert_eq!(payload["examples"][3]["job_id"], json!("job-viewed"));
        assert_eq!(payload["examples"][3]["label_reasons"], json!(["viewed"]));
        assert_eq!(payload["examples"][3]["signals"]["viewed"], json!(true));
        assert_eq!(
            payload["examples"][3]["signals"]["viewed_event_count"],
            json!(1)
        );
        assert!(payload["examples"][2]["ranking"]["deterministic_score"].is_number());
        assert!(payload["examples"][2]["ranking"]["learned_reranker_score"].is_number());
    }

    #[tokio::test]
    async fn reranker_dataset_requires_profile_analysis() {
        let state = base_state(sample_profile(None));

        let response = get_reranker_dataset(State(state), None, Path("profile-1".to_string()))
            .await
            .expect_err("dataset export should require analysis")
            .into_response();

        assert_eq!(response.status(), axum::http::StatusCode::BAD_REQUEST);
    }
}
