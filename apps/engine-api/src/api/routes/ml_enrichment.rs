use axum::Extension;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use serde_json::Value;
use tracing::warn;

use crate::api::error::{ApiError, ApiJson};
use crate::api::middleware::auth::AuthUser;
use crate::services::ml_gateway::MlGatewayError;
use crate::state::AppState;

pub async fn enrich(
    State(state): State<AppState>,
    auth: Option<Extension<AuthUser>>,
    Path(kind): Path<String>,
    ApiJson(payload): ApiJson<Value>,
) -> Result<axum::Json<Value>, ApiError> {
    let Some(Extension(auth)) = auth else {
        return Err(ApiError::unauthorized(
            "auth_required",
            "Authentication is required for ML enrichment",
        ));
    };
    let upstream_path = enrichment_path(&kind).ok_or_else(|| {
        ApiError::not_found(
            "enrichment_not_found",
            format!("Unknown enrichment operation '{kind}'"),
        )
    })?;

    validate_profile_scope(&payload, &auth.profile_id)?;

    state
        .ml_gateway
        .post_json(upstream_path, &payload)
        .await
        .map(axum::Json)
        .map_err(|error| {
            warn!(operation = kind, error = ?error, "ml enrichment call failed");
            match error {
                MlGatewayError::Http(_) => ApiError::bad_gateway(
                    "ml_sidecar_unavailable",
                    "ML enrichment service is currently unavailable",
                ),
                MlGatewayError::Upstream { status, detail } => {
                    warn!(%status, %detail, "ml enrichment upstream rejected request");
                    ApiError::bad_gateway(
                        "ml_sidecar_error",
                        "ML enrichment service returned an error",
                    )
                }
                MlGatewayError::Decode(_) => ApiError::bad_gateway(
                    "ml_sidecar_decode_error",
                    "ML enrichment service returned an unexpected response",
                ),
            }
        })
}

pub async fn health(State(state): State<AppState>) -> Result<axum::Json<Value>, ApiError> {
    state
        .ml_gateway
        .get_json("/health")
        .await
        .map(axum::Json)
        .map_err(map_gateway_error)
}

pub async fn bootstrap_reranker(
    State(state): State<AppState>,
    auth: Option<Extension<AuthUser>>,
    ApiJson(payload): ApiJson<Value>,
) -> Result<(StatusCode, axum::Json<Value>), ApiError> {
    let auth = require_auth(auth)?;
    validate_profile_scope(&payload, &auth.profile_id)?;
    state
        .ml_gateway
        .post_json("/api/v1/reranker/bootstrap", &payload)
        .await
        .map(|value| (StatusCode::ACCEPTED, axum::Json(value)))
        .map_err(map_gateway_error)
}

pub async fn reranker_bootstrap_status(
    State(state): State<AppState>,
    auth: Option<Extension<AuthUser>>,
    Path(task_id): Path<String>,
) -> Result<axum::Json<Value>, ApiError> {
    let auth = require_auth(auth)?;
    let response = state
        .ml_gateway
        .get_json(&format!("/api/v1/reranker/bootstrap/{task_id}"))
        .await
        .map_err(map_gateway_error)?;
    validate_profile_scope(&response, &auth.profile_id)?;
    Ok(axum::Json(response))
}

pub async fn invalidate_rerank(
    State(state): State<AppState>,
    auth: Option<Extension<AuthUser>>,
    ApiJson(payload): ApiJson<Value>,
) -> Result<StatusCode, ApiError> {
    let auth = require_auth(auth)?;
    validate_profile_scope(&payload, &auth.profile_id)?;
    state
        .ml_gateway
        .post_no_content("/api/v1/rerank/invalidate", &payload)
        .await
        .map(|()| StatusCode::NO_CONTENT)
        .map_err(map_gateway_error)
}

fn require_auth(auth: Option<Extension<AuthUser>>) -> Result<AuthUser, ApiError> {
    auth.map(|Extension(auth)| auth).ok_or_else(|| {
        ApiError::unauthorized(
            "auth_required",
            "Authentication is required for ML operations",
        )
    })
}

fn map_gateway_error(error: MlGatewayError) -> ApiError {
    warn!(error = ?error, "ml gateway call failed");
    match error {
        MlGatewayError::Http(_) => ApiError::bad_gateway(
            "ml_sidecar_unavailable",
            "ML service is currently unavailable",
        ),
        MlGatewayError::Upstream { status, detail } => {
            warn!(%status, %detail, "ml gateway upstream rejected request");
            ApiError::bad_gateway("ml_sidecar_error", "ML service returned an error")
        }
        MlGatewayError::Decode(_) => ApiError::bad_gateway(
            "ml_sidecar_decode_error",
            "ML service returned an unexpected response",
        ),
    }
}

fn enrichment_path(kind: &str) -> Option<&'static str> {
    match kind {
        "profile-insights" => Some("/api/v1/enrichment/profile-insights"),
        "weekly-guidance" => Some("/api/v1/enrichment/weekly-guidance"),
        "job-fit-explanation" => Some("/api/v1/enrichment/job-fit-explanation"),
        "application-coach" => Some("/api/v1/enrichment/application-coach"),
        "cover-letter-draft" => Some("/api/v1/enrichment/cover-letter-draft"),
        "interview-prep" => Some("/api/v1/enrichment/interview-prep"),
        "resume-match" => Some("/api/v1/enrichment/resume-match"),
        _ => None,
    }
}

fn validate_profile_scope(payload: &Value, authenticated_profile_id: &str) -> Result<(), ApiError> {
    let Some(profile_id) = payload.get("profile_id").and_then(Value::as_str) else {
        return Ok(());
    };

    if profile_id == authenticated_profile_id {
        Ok(())
    } else {
        Err(ApiError::forbidden(
            "profile_scope_mismatch",
            "The enrichment payload belongs to another profile",
        ))
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{enrichment_path, validate_profile_scope};

    #[test]
    fn only_known_enrichment_operations_are_forwarded() {
        assert_eq!(
            enrichment_path("resume-match"),
            Some("/api/v1/enrichment/resume-match")
        );
        assert_eq!(enrichment_path("../health"), None);
        assert_eq!(enrichment_path("rerank"), None);
    }

    #[test]
    fn rejects_payload_for_another_profile() {
        let error = validate_profile_scope(&json!({ "profile_id": "profile_other" }), "profile_1");
        assert!(error.is_err());
    }

    #[test]
    fn accepts_matching_or_profile_independent_payload() {
        assert!(validate_profile_scope(&json!({ "profile_id": "profile_1" }), "profile_1").is_ok());
        assert!(validate_profile_scope(&json!({ "resume_text": "Rust" }), "profile_1").is_ok());
    }
}
