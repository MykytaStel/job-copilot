use axum::Extension;
use axum::extract::State;
use axum::http::StatusCode;
use serde::Deserialize;
use serde_json::json;

use crate::api::error::{ApiError, ApiJson};
use crate::api::middleware::auth::AuthUser;
use crate::api::routes::feedback::ensure_profile_exists;
use crate::domain::profile::model::UpdateProfile;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct ResetDataRequest {
    pub profile_id: Option<String>,
}

pub async fn reset_profile_data(
    State(state): State<AppState>,
    auth: Option<Extension<AuthUser>>,
    ApiJson(payload): ApiJson<ResetDataRequest>,
) -> Result<StatusCode, ApiError> {
    let profile_id = resolve_reset_profile_id(auth.as_deref(), payload.profile_id)?;
    ensure_profile_exists(&state, auth.as_deref(), &profile_id).await?;

    state
        .feedback_service
        .delete_all_for_profile(&profile_id)
        .await
        .map_err(|error| ApiError::from_repository(error, "feedback_reset_failed"))?;

    state
        .applications_service
        .delete_by_profile(&profile_id)
        .await
        .map_err(|error| ApiError::from_repository(error, "applications_reset_failed"))?;

    state
        .profile_records
        .update(
            &profile_id,
            UpdateProfile {
                search_preferences: Some(None),
                ..Default::default()
            },
        )
        .await
        .map_err(|error| ApiError::from_repository(error, "profiles_query_failed"))?;

    Ok(StatusCode::NO_CONTENT)
}

fn resolve_reset_profile_id(
    auth: Option<&AuthUser>,
    requested_profile_id: Option<String>,
) -> Result<String, ApiError> {
    let requested_profile_id = requested_profile_id
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());

    if let Some(user) = auth {
        if let Some(requested_profile_id) = requested_profile_id
            && requested_profile_id != user.profile_id
        {
            return Err(ApiError::forbidden(
                "profile_access_denied",
                "You do not have access to this profile",
            ));
        }

        return Ok(user.profile_id.clone());
    }

    requested_profile_id.ok_or_else(|| {
        ApiError::bad_request_with_details(
            "missing_profile_id",
            "Field 'profile_id' is required when authentication is disabled",
            json!({ "field": "profile_id" }),
        )
    })
}

#[cfg(test)]
mod tests {
    use axum::Extension;
    use axum::extract::State;
    use axum::http::StatusCode;
    use axum::response::IntoResponse;

    use crate::api::error::ApiJson;
    use crate::api::middleware::auth::AuthUser;
    use crate::domain::application::model::Application;
    use crate::domain::feedback::model::{
        CompanyFeedbackRecord, CompanyFeedbackStatus, JobFeedbackFlags,
    };
    use crate::domain::profile::model::Profile;
    use crate::domain::search::profile::SearchPreferences;
    use crate::services::applications::{ApplicationsService, ApplicationsServiceStub};
    use crate::services::feedback::{FeedbackService, FeedbackServiceStub};
    use crate::services::jobs::{JobsService, JobsServiceStub};
    use crate::services::profiles::{ProfilesService, ProfilesServiceStub};
    use crate::services::resumes::{ResumesService, ResumesServiceStub};
    use crate::state::AppState;

    use super::{ResetDataRequest, reset_profile_data};

    #[tokio::test]
    async fn reset_deletes_profile_data_without_deleting_profile() {
        let state = test_state();

        state
            .feedback_service
            .upsert_job_feedback(
                "profile-1",
                "job-1",
                JobFeedbackFlags {
                    saved: true,
                    hidden: true,
                    bad_fit: true,
                    reason: None,
                },
            )
            .await
            .expect("feedback seed should succeed");
        state
            .feedback_service
            .upsert_company_feedback(
                "profile-1",
                "Blocked Co",
                "blocked co",
                CompanyFeedbackStatus::Blacklist,
            )
            .await
            .expect("company feedback seed should succeed");

        let status = reset_profile_data(
            State(state.clone()),
            Some(Extension(AuthUser {
                profile_id: "profile-1".to_string(),
            })),
            ApiJson(ResetDataRequest {
                profile_id: Some("profile-1".to_string()),
            }),
        )
        .await
        .expect("reset should succeed");

        assert_eq!(status, StatusCode::NO_CONTENT);

        let profile = state
            .profile_records
            .get_by_id("profile-1")
            .await
            .expect("profile lookup should succeed")
            .expect("profile should remain");
        assert_eq!(profile.name, "Jane Doe");
        assert!(profile.search_preferences.is_none());

        assert!(
            state
                .feedback_service
                .list_job_feedback("profile-1")
                .await
                .expect("feedback lookup should succeed")
                .is_empty()
        );
        assert!(
            state
                .feedback_service
                .list_company_feedback("profile-1")
                .await
                .expect("company feedback lookup should succeed")
                .is_empty()
        );
        assert!(
            state
                .applications_service
                .list_by_profile("profile-1")
                .await
                .expect("applications lookup should succeed")
                .is_empty()
        );
    }

    #[tokio::test]
    async fn reset_rejects_profile_mismatch() {
        let state = test_state();

        let error = reset_profile_data(
            State(state),
            Some(Extension(AuthUser {
                profile_id: "profile-1".to_string(),
            })),
            ApiJson(ResetDataRequest {
                profile_id: Some("profile-2".to_string()),
            }),
        )
        .await
        .expect_err("profile mismatch should be rejected");

        assert_eq!(error.into_response().status(), StatusCode::FORBIDDEN);
    }

    fn test_state() -> AppState {
        let profile = Profile {
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
            preferred_locations: vec![],
            experience: vec![],
            work_mode_preference: "any".to_string(),
            preferred_language: None,
            search_preferences: Some(SearchPreferences::default()),
            created_at: "2026-04-14T00:00:00Z".to_string(),
            updated_at: "2026-04-14T00:00:00Z".to_string(),
            skills_updated_at: None,
            portfolio_url: None,
            github_url: None,
            linkedin_url: None,
        };

        let application = Application {
            id: "application-1".to_string(),
            job_id: "job-1".to_string(),
            resume_id: None,
            status: "applied".to_string(),
            applied_at: None,
            due_date: None,
            outcome: None,
            outcome_date: None,
            rejection_stage: None,
            updated_at: "2026-04-14T00:00:00Z".to_string(),
        };

        let mut state = AppState::for_services(
            ProfilesService::for_tests(ProfilesServiceStub::default().with_profile(profile)),
            JobsService::for_tests(JobsServiceStub::default()),
            ApplicationsService::for_tests(
                ApplicationsServiceStub::default()
                    .with_profile_application("profile-1", application),
            ),
            ResumesService::for_tests(ResumesServiceStub::default()),
        );
        state.feedback_service = FeedbackService::for_tests(
            FeedbackServiceStub::default().with_company_feedback(CompanyFeedbackRecord {
                profile_id: "profile-2".to_string(),
                company_name: "Other Co".to_string(),
                normalized_company_name: "other co".to_string(),
                status: CompanyFeedbackStatus::Blacklist,
                notes: String::new(),
                created_at: "2026-04-14T00:00:00Z".to_string(),
                updated_at: "2026-04-14T00:00:00Z".to_string(),
            }),
        );
        state
    }
}
