use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;

use crate::api::dto::profile::AnalyzeProfileResponse;
use crate::api::dto::search_profile::{
    BuildSearchProfileRequest, BuildSearchProfileResponse, BuildSearchProfileWarningResponse,
    SearchPreferencesValidationError, SearchProfileResponse,
};
use crate::state::AppState;

#[derive(Debug, Serialize)]
struct ValidationErrorResponse {
    code: &'static str,
    field: &'static str,
    error: &'static str,
    message: &'static str,
    invalid_values: Vec<String>,
    allowed_values: Vec<String>,
}

#[derive(Debug)]
pub(crate) struct SearchProfileRequestError {
    body: ValidationErrorResponse,
}

impl From<SearchPreferencesValidationError> for SearchProfileRequestError {
    fn from(error: SearchPreferencesValidationError) -> Self {
        Self {
            body: ValidationErrorResponse {
                code: "invalid_preferred_roles",
                field: "preferred_roles",
                error: "invalid_preferred_roles",
                message: "Unknown preferred_roles values",
                invalid_values: error.invalid_preferred_roles().to_vec(),
                allowed_values: error.allowed_preferred_roles(),
            },
        }
    }
}

impl IntoResponse for SearchProfileRequestError {
    fn into_response(self) -> Response {
        (StatusCode::BAD_REQUEST, Json(self.body)).into_response()
    }
}

pub async fn build_search_profile(
    State(state): State<AppState>,
    Json(payload): Json<BuildSearchProfileRequest>,
) -> Result<Json<BuildSearchProfileResponse>, SearchProfileRequestError> {
    let parsed_preferences = payload.preferences.parse()?;
    let analyzed_profile = state.profile_analysis_service.analyze(&payload.raw_text);
    let search_profile = state
        .search_profile_service
        .build(&analyzed_profile, &parsed_preferences.preferences);

    let mut warnings = Vec::new();

    if let Some(warning) = BuildSearchProfileWarningResponse::from_deprecated_preferred_roles(
        &parsed_preferences.deprecated_preferred_roles,
    ) {
        warnings.push(warning);
    }

    let response = BuildSearchProfileResponse {
        analyzed_profile: AnalyzeProfileResponse::from(analyzed_profile),
        search_profile: SearchProfileResponse::from(search_profile),
        warnings,
    };

    Ok(Json(response))
}

#[cfg(test)]
mod tests {
    use axum::Json;
    use axum::body;
    use axum::extract::State;
    use axum::http::StatusCode;
    use axum::response::IntoResponse;
    use serde_json::{Value, json};

    use crate::api::dto::search_profile::{BuildSearchProfileRequest, SearchPreferencesRequest};
    use crate::state::AppState;

    use super::build_search_profile;

    #[tokio::test]
    async fn returns_bad_request_for_unknown_preferred_roles() {
        let result = build_search_profile(
            State(AppState::without_database()),
            Json(BuildSearchProfileRequest {
                raw_text: "Senior frontend engineer".to_string(),
                preferences: SearchPreferencesRequest {
                    preferred_roles: vec![
                        "frontend_developer".to_string(),
                        "frontend_specialist".to_string(),
                    ],
                    ..SearchPreferencesRequest::default()
                },
            }),
        )
        .await;

        let response = match result {
            Ok(_) => panic!("handler should reject unknown preferred roles"),
            Err(error) => error.into_response(),
        };

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = body::to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("response body should be readable");
        let payload: Value =
            serde_json::from_slice(&body).expect("response body should be valid json");

        assert_eq!(payload["error"], json!("invalid_preferred_roles"));
        assert_eq!(payload["code"], json!("invalid_preferred_roles"));
        assert_eq!(payload["field"], json!("preferred_roles"));
        assert_eq!(payload["message"], json!("Unknown preferred_roles values"));
        assert_eq!(payload["invalid_values"], json!(["frontend_specialist"]));
        assert!(
            payload["allowed_values"]
                .as_array()
                .expect("allowed_values should be an array")
                .contains(&json!("frontend_developer"))
        );
    }

    #[tokio::test]
    async fn returns_success_for_known_preferred_roles() {
        let result = build_search_profile(
            State(AppState::without_database()),
            Json(BuildSearchProfileRequest {
                raw_text: "Senior React Native developer".to_string(),
                preferences: SearchPreferencesRequest {
                    preferred_roles: vec!["frontend_developer".to_string()],
                    ..SearchPreferencesRequest::default()
                },
            }),
        )
        .await;

        let response = match result {
            Ok(response) => response.into_response(),
            Err(_) => panic!("handler should accept known preferred roles"),
        };

        assert_eq!(response.status(), StatusCode::OK);

        let body = body::to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("response body should be readable");
        let payload: Value =
            serde_json::from_slice(&body).expect("response body should be valid json");

        assert_eq!(
            payload["search_profile"]["target_roles"],
            json!(["react_native_developer", "frontend_developer",])
        );
        assert!(payload.get("warnings").is_none());
    }

    #[tokio::test]
    async fn returns_warnings_for_deprecated_preferred_roles() {
        let result = build_search_profile(
            State(AppState::without_database()),
            Json(BuildSearchProfileRequest {
                raw_text: "Senior React Native developer".to_string(),
                preferences: SearchPreferencesRequest {
                    preferred_roles: vec!["front_end_developer".to_string()],
                    ..SearchPreferencesRequest::default()
                },
            }),
        )
        .await;

        let response = match result {
            Ok(response) => response.into_response(),
            Err(_) => panic!("handler should normalize deprecated preferred roles"),
        };

        assert_eq!(response.status(), StatusCode::OK);

        let body = body::to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("response body should be readable");
        let payload: Value =
            serde_json::from_slice(&body).expect("response body should be valid json");

        assert_eq!(
            payload["warnings"],
            json!([{
                "code": "deprecated_preferred_roles",
                "field": "preferred_roles",
                "message": "Deprecated preferred_roles values were normalized to canonical role ids",
                "replacements": [{
                    "received": "front_end_developer",
                    "normalized_to": "frontend_developer"
                }]
            }])
        );
    }
}
