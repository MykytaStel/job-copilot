use axum::extract::State;

use crate::api::dto::profile::AnalyzeProfileResponse;
use crate::api::dto::search_profile::{
    BuildSearchProfileRequest, BuildSearchProfileResponse, SearchProfileResponse,
};
use crate::api::error::{ApiError, ApiJson};
use crate::state::AppState;

pub async fn build_search_profile(
    State(state): State<AppState>,
    ApiJson(payload): ApiJson<BuildSearchProfileRequest>,
) -> Result<axum::Json<BuildSearchProfileResponse>, ApiError> {
    let input = payload.validate()?;
    let analyzed_profile = state.profile_analysis_service.analyze(&input.raw_text);
    let search_profile = state
        .search_profile_service
        .build(&analyzed_profile, &input.preferences);

    Ok(axum::Json(BuildSearchProfileResponse {
        analyzed_profile: AnalyzeProfileResponse::from(analyzed_profile),
        search_profile: SearchProfileResponse::from(search_profile),
    }))
}

#[cfg(test)]
mod tests {
    use axum::body;
    use axum::extract::State;
    use axum::http::StatusCode;
    use axum::response::IntoResponse;
    use serde_json::{Value, json};

    use crate::api::dto::search_profile::{BuildSearchProfileRequest, SearchPreferencesRequest};
    use crate::api::error::ApiJson;
    use crate::services::applications::{ApplicationsService, ApplicationsServiceStub};
    use crate::services::jobs::{JobsService, JobsServiceStub};
    use crate::services::profiles::{ProfilesService, ProfilesServiceStub};
    use crate::services::resumes::{ResumesService, ResumesServiceStub};
    use crate::state::AppState;

    use super::build_search_profile;

    fn test_state() -> AppState {
        AppState::for_services(
            ProfilesService::for_tests(ProfilesServiceStub::default()),
            JobsService::for_tests(JobsServiceStub::default()),
            ApplicationsService::for_tests(ApplicationsServiceStub::default()),
            ResumesService::for_tests(ResumesServiceStub::default()),
        )
    }

    #[tokio::test]
    async fn builds_search_profile_from_raw_text() {
        let response = build_search_profile(
            State(test_state()),
            ApiJson(BuildSearchProfileRequest {
                raw_text: "Senior React Native developer with product experience".to_string(),
                preferences: SearchPreferencesRequest {
                    target_regions: vec![crate::domain::search::profile::TargetRegion::Ua],
                    work_modes: vec![crate::domain::search::profile::WorkMode::Remote],
                    preferred_roles: vec!["frontend_engineer".to_string()],
                    allowed_sources: vec!["djinni".to_string(), "work_ua".to_string()],
                    include_keywords: vec!["product company".to_string()],
                    exclude_keywords: vec!["gambling".to_string()],
                },
            }),
        )
        .await
        .expect("handler should build search profile")
        .into_response();

        assert_eq!(response.status(), StatusCode::OK);

        let body = body::to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("response body should be readable");
        let payload: Value =
            serde_json::from_slice(&body).expect("response body should be valid json");

        assert_eq!(
            payload["analyzed_profile"]["primary_role"],
            json!("mobile_engineer")
        );
        assert_eq!(
            payload["search_profile"]["allowed_sources"],
            json!(["djinni", "work_ua"])
        );
        assert!(
            payload["search_profile"]["target_roles"]
                .as_array()
                .expect("target_roles should be an array")
                .contains(&json!("frontend_engineer"))
        );
    }

    #[tokio::test]
    async fn rejects_invalid_allowed_sources() {
        let result = build_search_profile(
            State(test_state()),
            ApiJson(BuildSearchProfileRequest {
                raw_text: "Senior frontend engineer".to_string(),
                preferences: SearchPreferencesRequest {
                    allowed_sources: vec!["linkedin".to_string()],
                    ..SearchPreferencesRequest::default()
                },
            }),
        )
        .await;

        let response = match result {
            Ok(_) => panic!("handler should reject unknown allowed_sources"),
            Err(error) => error.into_response(),
        };

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn rejects_invalid_preferred_roles() {
        let result = build_search_profile(
            State(test_state()),
            ApiJson(BuildSearchProfileRequest {
                raw_text: "Senior frontend engineer".to_string(),
                preferences: SearchPreferencesRequest {
                    preferred_roles: vec!["frontend_specialist".to_string()],
                    ..SearchPreferencesRequest::default()
                },
            }),
        )
        .await;

        let response = match result {
            Ok(_) => panic!("handler should reject unknown preferred_roles"),
            Err(error) => error.into_response(),
        };

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }
}
