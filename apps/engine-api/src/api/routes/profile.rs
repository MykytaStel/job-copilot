use axum::extract::{Path, State};
use axum::http::StatusCode;

use crate::api::dto::profile::{
    AnalyzeProfileResponse, CreateProfileRequest, ProfileResponse, UpdateProfileRequest,
};
use crate::api::dto::search_profile::{
    BuildSearchProfileRequest, BuildSearchProfileResponse, SearchProfileResponse,
};
use crate::api::error::{ApiError, ApiJson};
use crate::domain::profile::model::ProfileAnalysis;
use crate::state::AppState;

pub async fn create_profile(
    State(state): State<AppState>,
    ApiJson(payload): ApiJson<CreateProfileRequest>,
) -> Result<(StatusCode, axum::Json<ProfileResponse>), ApiError> {
    let profile = state
        .profiles_service
        .create(payload.validate()?)
        .await
        .map_err(|error| ApiError::from_repository(error, "profiles_query_failed"))?;

    Ok((
        StatusCode::CREATED,
        axum::Json(ProfileResponse::from(profile)),
    ))
}

pub async fn get_profile_by_id(
    State(state): State<AppState>,
    Path(profile_id): Path<String>,
) -> Result<axum::Json<ProfileResponse>, ApiError> {
    let Some(profile) = state
        .profiles_service
        .get_by_id(&profile_id)
        .await
        .map_err(|error| ApiError::from_repository(error, "profiles_query_failed"))?
    else {
        return Err(profile_not_found(&profile_id));
    };

    Ok(axum::Json(ProfileResponse::from(profile)))
}

pub async fn update_profile(
    State(state): State<AppState>,
    Path(profile_id): Path<String>,
    ApiJson(payload): ApiJson<UpdateProfileRequest>,
) -> Result<axum::Json<ProfileResponse>, ApiError> {
    let Some(profile) = state
        .profiles_service
        .update(&profile_id, payload.validate()?)
        .await
        .map_err(|error| ApiError::from_repository(error, "profiles_query_failed"))?
    else {
        return Err(profile_not_found(&profile_id));
    };

    Ok(axum::Json(ProfileResponse::from(profile)))
}

pub async fn analyze_profile(
    State(state): State<AppState>,
    Path(profile_id): Path<String>,
) -> Result<axum::Json<AnalyzeProfileResponse>, ApiError> {
    let Some(profile) = state
        .profiles_service
        .get_by_id(&profile_id)
        .await
        .map_err(|error| ApiError::from_repository(error, "profiles_query_failed"))?
    else {
        return Err(profile_not_found(&profile_id));
    };

    let analyzed_profile = state.profile_analysis_service.analyze(&profile.raw_text);

    state
        .profiles_service
        .save_analysis(
            &profile_id,
            ProfileAnalysis {
                summary: analyzed_profile.summary.clone(),
                primary_role: analyzed_profile.primary_role,
                seniority: analyzed_profile.seniority.clone(),
                skills: analyzed_profile.skills.clone(),
                keywords: analyzed_profile.keywords.clone(),
            },
        )
        .await
        .map_err(|error| ApiError::from_repository(error, "profiles_query_failed"))?;

    Ok(axum::Json(AnalyzeProfileResponse::from(analyzed_profile)))
}

pub async fn build_search_profile(
    State(state): State<AppState>,
    Path(profile_id): Path<String>,
    ApiJson(payload): ApiJson<BuildSearchProfileRequest>,
) -> Result<axum::Json<BuildSearchProfileResponse>, ApiError> {
    let Some(profile) = state
        .profiles_service
        .get_by_id(&profile_id)
        .await
        .map_err(|error| ApiError::from_repository(error, "profiles_query_failed"))?
    else {
        return Err(profile_not_found(&profile_id));
    };

    let preferences = payload.preferences.validate()?;
    let analyzed_profile = state.profile_analysis_service.analyze(&profile.raw_text);
    let search_profile = state
        .search_profile_service
        .build(&analyzed_profile, &preferences);

    Ok(axum::Json(BuildSearchProfileResponse {
        analyzed_profile: AnalyzeProfileResponse::from(analyzed_profile),
        search_profile: SearchProfileResponse::from(search_profile),
    }))
}

fn profile_not_found(profile_id: &str) -> ApiError {
    ApiError::not_found(
        "profile_not_found",
        format!("Profile '{profile_id}' was not found"),
    )
}

#[cfg(test)]
mod tests {
    use axum::body;
    use axum::extract::{Path, State};
    use axum::http::StatusCode;
    use axum::response::IntoResponse;
    use serde_json::{Value, json};

    use crate::api::dto::profile::CreateProfileRequest;
    use crate::api::dto::search_profile::BuildSearchProfileRequest;
    use crate::api::error::ApiJson;
    use crate::services::applications::{ApplicationsService, ApplicationsServiceStub};
    use crate::services::jobs::{JobsService, JobsServiceStub};
    use crate::services::profiles::{ProfilesService, ProfilesServiceStub};
    use crate::services::resumes::{ResumesService, ResumesServiceStub};
    use crate::state::AppState;

    use super::{
        analyze_profile, build_search_profile, create_profile, get_profile_by_id, update_profile,
    };

    #[tokio::test]
    async fn creates_profile() {
        let state = AppState::for_services(
            ProfilesService::for_tests(ProfilesServiceStub::default()),
            JobsService::for_tests(JobsServiceStub::default()),
            ApplicationsService::for_tests(ApplicationsServiceStub::default()),
            ResumesService::for_tests(ResumesServiceStub::default()),
        );

        let result = create_profile(
            State(state),
            ApiJson(CreateProfileRequest {
                name: "Jane Doe".to_string(),
                email: "jane@example.com".to_string(),
                location: Some("Kyiv".to_string()),
                raw_text: "Senior frontend engineer with React and TypeScript".to_string(),
            }),
        )
        .await
        .expect("handler should create profile");

        assert_eq!(result.0, StatusCode::CREATED);
    }

    #[tokio::test]
    async fn returns_not_found_for_unknown_profile() {
        let state = AppState::for_services(
            ProfilesService::for_tests(ProfilesServiceStub::default()),
            JobsService::for_tests(JobsServiceStub::default()),
            ApplicationsService::for_tests(ApplicationsServiceStub::default()),
            ResumesService::for_tests(ResumesServiceStub::default()),
        );

        let response = get_profile_by_id(State(state), Path("missing-profile".to_string()))
            .await
            .expect_err("handler should reject unknown profile")
            .into_response();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn analyzes_persisted_profile() {
        let state = AppState::for_services(
            ProfilesService::for_tests(ProfilesServiceStub::default()),
            JobsService::for_tests(JobsServiceStub::default()),
            ApplicationsService::for_tests(ApplicationsServiceStub::default()),
            ResumesService::for_tests(ResumesServiceStub::default()),
        );
        let _ = create_profile(
            State(state.clone()),
            ApiJson(CreateProfileRequest {
                name: "Jane Doe".to_string(),
                email: "jane@example.com".to_string(),
                location: Some("Kyiv".to_string()),
                raw_text: "Senior React Native developer with TypeScript".to_string(),
            }),
        )
        .await
        .expect("setup should create profile");

        let response = analyze_profile(State(state), Path("profile_test_001".to_string()))
            .await
            .expect("handler should analyze stored profile")
            .into_response();

        let body = body::to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("response body should be readable");
        let payload: Value =
            serde_json::from_slice(&body).expect("response body should be valid json");

        assert_eq!(payload["primary_role"], json!("react_native_developer"));
    }

    #[tokio::test]
    async fn builds_search_profile_from_persisted_profile() {
        let state = AppState::for_services(
            ProfilesService::for_tests(ProfilesServiceStub::default()),
            JobsService::for_tests(JobsServiceStub::default()),
            ApplicationsService::for_tests(ApplicationsServiceStub::default()),
            ResumesService::for_tests(ResumesServiceStub::default()),
        );
        let _ = create_profile(
            State(state.clone()),
            ApiJson(CreateProfileRequest {
                name: "Jane Doe".to_string(),
                email: "jane@example.com".to_string(),
                location: Some("Kyiv".to_string()),
                raw_text: "Senior React Native developer with product experience".to_string(),
            }),
        )
        .await
        .expect("setup should create profile");

        let response = build_search_profile(
            State(state),
            Path("profile_test_001".to_string()),
            ApiJson(BuildSearchProfileRequest {
                preferences: crate::api::dto::search_profile::SearchPreferencesRequest {
                    preferred_roles: vec!["frontend_developer".to_string()],
                    ..Default::default()
                },
            }),
        )
        .await
        .expect("handler should build search profile")
        .into_response();

        let body = body::to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("response body should be readable");
        let payload: Value =
            serde_json::from_slice(&body).expect("response body should be valid json");

        assert!(
            payload["search_profile"]["target_roles"]
                .as_array()
                .expect("target_roles should be an array")
                .contains(&json!("frontend_developer"))
        );
    }

    #[tokio::test]
    async fn rejects_invalid_patch() {
        let state = AppState::for_services(
            ProfilesService::for_tests(ProfilesServiceStub::default()),
            JobsService::for_tests(JobsServiceStub::default()),
            ApplicationsService::for_tests(ApplicationsServiceStub::default()),
            ResumesService::for_tests(ResumesServiceStub::default()),
        );

        let response = update_profile(
            State(state),
            Path("profile_test_001".to_string()),
            ApiJson(crate::api::dto::profile::UpdateProfileRequest::default()),
        )
        .await
        .expect_err("handler should reject empty patch")
        .into_response();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }
}
