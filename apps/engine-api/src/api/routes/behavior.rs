use axum::extract::{Path, State};

use crate::api::dto::behavior::ProfileBehaviorSummaryResponse;
use crate::api::error::ApiError;
use crate::api::routes::feedback::ensure_profile_exists;
use crate::services::behavior::BehaviorService;
use crate::state::AppState;

pub async fn get_behavior_summary(
    State(state): State<AppState>,
    Path(profile_id): Path<String>,
) -> Result<axum::Json<ProfileBehaviorSummaryResponse>, ApiError> {
    ensure_profile_exists(&state, &profile_id).await?;

    let events = state
        .user_events_service
        .list_by_profile(&profile_id)
        .await
        .map_err(|error| ApiError::from_repository(error, "user_event_query_failed"))?;
    let behavior_service = BehaviorService::new();
    let aggregates = behavior_service.build_aggregates(events.iter());
    let summary = behavior_service.summarize(&aggregates);

    Ok(axum::Json(ProfileBehaviorSummaryResponse::from_summary(
        profile_id, summary,
    )))
}

#[cfg(test)]
mod tests {
    use axum::body;
    use axum::extract::{Path, State};
    use axum::response::IntoResponse;
    use serde_json::{Value, json};

    use crate::domain::profile::model::Profile;
    use crate::domain::user_event::model::{UserEventRecord, UserEventType};
    use crate::services::applications::{ApplicationsService, ApplicationsServiceStub};
    use crate::services::jobs::{JobsService, JobsServiceStub};
    use crate::services::profiles::{ProfilesService, ProfilesServiceStub};
    use crate::services::resumes::{ResumesService, ResumesServiceStub};
    use crate::services::user_events::{UserEventsService, UserEventsServiceStub};
    use crate::state::AppState;

    use super::get_behavior_summary;

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

    fn event(
        id: &str,
        event_type: UserEventType,
        source: Option<&str>,
        role_family: Option<&str>,
    ) -> UserEventRecord {
        UserEventRecord {
            id: id.to_string(),
            profile_id: "profile-1".to_string(),
            event_type,
            job_id: Some(format!("job-{id}")),
            company_name: Some("NovaLedger".to_string()),
            source: source.map(str::to_string),
            role_family: role_family.map(str::to_string),
            payload_json: None,
            created_at: "2026-04-15T00:00:00Z".to_string(),
        }
    }

    #[tokio::test]
    async fn behavior_summary_is_queryable_for_profile() {
        let state = AppState::for_services(
            ProfilesService::for_tests(
                ProfilesServiceStub::default().with_profile(sample_profile()),
            ),
            JobsService::for_tests(JobsServiceStub::default()),
            ApplicationsService::for_tests(ApplicationsServiceStub::default()),
            ResumesService::for_tests(ResumesServiceStub::default()),
        )
        .with_user_events_service(UserEventsService::for_tests(
            UserEventsServiceStub::default()
                .with_event(event(
                    "evt-1",
                    UserEventType::JobSaved,
                    Some("djinni"),
                    Some("engineering"),
                ))
                .with_event(event(
                    "evt-2",
                    UserEventType::ApplicationCreated,
                    Some("djinni"),
                    Some("engineering"),
                ))
                .with_event(event(
                    "evt-3",
                    UserEventType::JobHidden,
                    Some("work_ua"),
                    Some("product"),
                ))
                .with_event(event("evt-4", UserEventType::SearchRun, None, None)),
        ));

        let response = get_behavior_summary(State(state), Path("profile-1".to_string()))
            .await
            .expect("behavior summary should succeed")
            .into_response();

        let body = body::to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("response body should be readable");
        let payload: Value =
            serde_json::from_slice(&body).expect("response body should be valid JSON");

        assert_eq!(payload["profile_id"], json!("profile-1"));
        assert_eq!(payload["search_run_count"], json!(1));
        assert_eq!(payload["top_positive_sources"][0]["key"], json!("djinni"));
        assert_eq!(payload["top_negative_sources"][0]["key"], json!("work_ua"));
        assert_eq!(
            payload["top_positive_role_families"][0]["key"],
            json!("engineering")
        );
        assert_eq!(
            payload["source_signal_counts"].as_array().map(Vec::len),
            Some(2)
        );
    }
}
