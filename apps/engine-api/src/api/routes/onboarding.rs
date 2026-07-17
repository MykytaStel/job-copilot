use axum::Extension;
use axum::extract::{Path, State};
use serde::Deserialize;

use crate::api::error::{ApiError, ApiJson};
use crate::api::middleware::auth::{AuthUser, check_profile_ownership};
use crate::domain::profile::onboarding::{OnboardingOutcome, OnboardingStep, ProfileOnboarding};
use crate::state::AppState;

#[derive(Deserialize)]
pub struct AdvanceOnboardingRequest {
    pub step: OnboardingStep,
    pub outcome: OnboardingOutcome,
}

pub async fn get_profile_onboarding(
    State(state): State<AppState>,
    auth: Option<Extension<AuthUser>>,
    Path(profile_id): Path<String>,
) -> Result<axum::Json<ProfileOnboarding>, ApiError> {
    ensure_profile_access(&state, auth.as_deref(), &profile_id).await?;
    let onboarding = state
        .profile_onboarding
        .get_or_create(&profile_id)
        .await
        .map_err(|error| ApiError::from_repository(error, "profile_onboarding_query_failed"))?;

    Ok(axum::Json(onboarding))
}

pub async fn advance_profile_onboarding(
    State(state): State<AppState>,
    auth: Option<Extension<AuthUser>>,
    Path(profile_id): Path<String>,
    ApiJson(payload): ApiJson<AdvanceOnboardingRequest>,
) -> Result<axum::Json<ProfileOnboarding>, ApiError> {
    ensure_profile_access(&state, auth.as_deref(), &profile_id).await?;
    let onboarding = state
        .profile_onboarding
        .advance(&profile_id, payload.step, payload.outcome)
        .await
        .map_err(|error| ApiError::from_repository(error, "profile_onboarding_update_failed"))?;

    Ok(axum::Json(onboarding))
}

async fn ensure_profile_access(
    state: &AppState,
    auth: Option<&AuthUser>,
    profile_id: &str,
) -> Result<(), ApiError> {
    check_profile_ownership(auth, profile_id)?;
    let profile = state
        .profile_records
        .get_by_id(profile_id)
        .await
        .map_err(|error| ApiError::from_repository(error, "profiles_query_failed"))?;
    if profile.is_none() {
        return Err(ApiError::not_found(
            "profile_not_found",
            "Profile not found",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use axum::Extension;
    use axum::extract::{Path, State};

    use crate::api::error::ApiJson;
    use crate::api::middleware::auth::AuthUser;
    use crate::domain::profile::model::Profile;
    use crate::domain::profile::onboarding::{OnboardingOutcome, OnboardingStep};
    use crate::services::applications::{ApplicationsService, ApplicationsServiceStub};
    use crate::services::jobs::{JobsService, JobsServiceStub};
    use crate::services::profiles::{ProfilesService, ProfilesServiceStub};
    use crate::services::resumes::{ResumesService, ResumesServiceStub};
    use crate::state::AppState;

    use super::{AdvanceOnboardingRequest, advance_profile_onboarding, get_profile_onboarding};

    fn sample_profile() -> Profile {
        Profile {
            id: "profile-1".to_string(),
            name: "Jane Doe".to_string(),
            email: "jane@example.com".to_string(),
            location: None,
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
            search_preferences: None,
            created_at: "2026-07-17T00:00:00Z".to_string(),
            updated_at: "2026-07-17T00:00:00Z".to_string(),
            skills_updated_at: None,
            portfolio_url: None,
            github_url: None,
            linkedin_url: None,
        }
    }

    fn state() -> AppState {
        AppState::for_services(
            ProfilesService::for_tests(
                ProfilesServiceStub::default().with_profile(sample_profile()),
            ),
            JobsService::for_tests(JobsServiceStub::default()),
            ApplicationsService::for_tests(ApplicationsServiceStub::default()),
            ResumesService::for_tests(ResumesServiceStub::default()),
        )
    }

    fn auth() -> Option<Extension<AuthUser>> {
        Some(Extension(AuthUser {
            profile_id: "profile-1".to_string(),
        }))
    }

    #[tokio::test]
    async fn progress_is_profile_scoped_and_persisted() {
        let state = state();

        let axum::Json(updated) = advance_profile_onboarding(
            State(state.clone()),
            auth(),
            Path("profile-1".to_string()),
            ApiJson(AdvanceOnboardingRequest {
                step: OnboardingStep::Cv,
                outcome: OnboardingOutcome::Completed,
            }),
        )
        .await
        .expect("step should advance");
        let axum::Json(loaded) =
            get_profile_onboarding(State(state), auth(), Path("profile-1".to_string()))
                .await
                .expect("state should load");

        assert_eq!(updated.current_step, OnboardingStep::Profile);
        assert_eq!(loaded, updated);
    }

    #[tokio::test]
    async fn rejects_profile_owner_mismatch() {
        let error = get_profile_onboarding(
            State(state()),
            Some(Extension(AuthUser {
                profile_id: "other-profile".to_string(),
            })),
            Path("profile-1".to_string()),
        )
        .await
        .expect_err("owner mismatch should fail");

        assert_eq!(
            axum::response::IntoResponse::into_response(error).status(),
            axum::http::StatusCode::FORBIDDEN
        );
    }
}
