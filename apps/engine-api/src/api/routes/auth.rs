use axum::extract::State;
use axum::http::StatusCode;

use crate::api::dto::auth::{AuthResponse, LoginRequest, RegisterRequest};
use crate::api::error::{ApiError, ApiJson};
use crate::domain::profile::model::CreateProfile;
use crate::services::tokens::issue_token;
use crate::state::AppState;

pub async fn register(
    State(state): State<AppState>,
    ApiJson(payload): ApiJson<RegisterRequest>,
) -> Result<(StatusCode, axum::Json<AuthResponse>), ApiError> {
    let Some(ref secret) = state.jwt_secret else {
        return Err(ApiError::service_unavailable(
            "auth_not_configured",
            "JWT_SECRET is not set; authentication is disabled in this environment",
        ));
    };

    let payload = payload.validate()?;

    let profile = state
        .profile_records
        .create(CreateProfile {
            name: payload.name,
            email: payload.email,
            raw_text: payload.raw_text,
            location: None,
            years_of_experience: None,
            salary_min: None,
            salary_max: None,
            salary_currency: "USD".to_string(),
            languages: vec![],
            preferred_locations: vec![],
            work_mode_preference: "any".to_string(),
            search_preferences: None,
            portfolio_url: None,
            github_url: None,
            linkedin_url: None,
        })
        .await
        .map_err(|error| ApiError::from_repository(error, "profiles_query_failed"))?;

    let issued = issue_token(&profile.id, secret)?;

    Ok((
        StatusCode::CREATED,
        axum::Json(AuthResponse {
            token: issued.token,
            profile_id: profile.id,
            expires_at: issued.expires_at.to_rfc3339(),
        }),
    ))
}

pub async fn login(
    State(state): State<AppState>,
    ApiJson(payload): ApiJson<LoginRequest>,
) -> Result<axum::Json<AuthResponse>, ApiError> {
    let Some(ref secret) = state.jwt_secret else {
        return Err(ApiError::service_unavailable(
            "auth_not_configured",
            "JWT_SECRET is not set; authentication is disabled in this environment",
        ));
    };

    let payload = payload.validate()?;

    let profile = state
        .profile_records
        .get_by_email(&payload.email)
        .await
        .map_err(|error| ApiError::from_repository(error, "profiles_query_failed"))?
        .ok_or_else(|| {
            ApiError::not_found(
                "profile_not_found",
                "No profile found for this email address",
            )
        })?;

    let issued = issue_token(&profile.id, secret)?;

    Ok(axum::Json(AuthResponse {
        token: issued.token,
        profile_id: profile.id,
        expires_at: issued.expires_at.to_rfc3339(),
    }))
}
