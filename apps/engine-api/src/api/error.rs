use axum::Json;
use axum::body::Bytes;
use axum::extract::FromRequest;
use axum::extract::Request;
use axum::extract::rejection::JsonRejection;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::{Value, json};
use tracing::error;

use crate::db::repositories::RepositoryError;

#[derive(Debug, Serialize)]
pub struct ApiErrorResponse {
    pub code: &'static str,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<Value>,
}

#[derive(Debug)]
pub struct ApiError {
    status: StatusCode,
    body: ApiErrorResponse,
}

pub struct ApiJson<T>(pub T);

pub struct OptionalApiJson<T>(pub T);

impl ApiError {
    pub fn bad_request(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            body: ApiErrorResponse {
                code,
                message: message.into(),
                details: None,
            },
        }
    }

    pub fn bad_request_with_details(
        code: &'static str,
        message: impl Into<String>,
        details: Value,
    ) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            body: ApiErrorResponse {
                code,
                message: message.into(),
                details: Some(details),
            },
        }
    }

    pub fn unprocessable_entity_with_details(
        code: &'static str,
        message: impl Into<String>,
        details: Value,
    ) -> Self {
        Self {
            status: StatusCode::UNPROCESSABLE_ENTITY,
            body: ApiErrorResponse {
                code,
                message: message.into(),
                details: Some(details),
            },
        }
    }

    pub fn not_found(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            body: ApiErrorResponse {
                code,
                message: message.into(),
                details: None,
            },
        }
    }

    pub fn service_unavailable(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::SERVICE_UNAVAILABLE,
            body: ApiErrorResponse {
                code,
                message: message.into(),
                details: None,
            },
        }
    }

    pub fn internal(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            body: ApiErrorResponse {
                code,
                message: message.into(),
                details: None,
            },
        }
    }

    pub fn unauthorized(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::UNAUTHORIZED,
            body: ApiErrorResponse {
                code,
                message: message.into(),
                details: None,
            },
        }
    }

    pub fn forbidden(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::FORBIDDEN,
            body: ApiErrorResponse {
                code,
                message: message.into(),
                details: None,
            },
        }
    }

    pub fn bad_gateway(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::BAD_GATEWAY,
            body: ApiErrorResponse {
                code,
                message: message.into(),
                details: None,
            },
        }
    }

    pub fn conflict(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::CONFLICT,
            body: ApiErrorResponse {
                code,
                message: message.into(),
                details: None,
            },
        }
    }

    pub fn invalid_limit(limit: i64) -> Self {
        Self::bad_request_with_details(
            "invalid_limit",
            format!("Query parameter 'limit' must be between 1 and 100, got {limit}"),
            json!({
                "field": "limit",
                "min": 1,
                "max": 100,
                "received": limit,
            }),
        )
    }

    pub fn invalid_period(period: i64) -> Self {
        Self::bad_request_with_details(
            "invalid_period",
            format!("Query parameter 'period' must be between 1 and 365, got {period}"),
            json!({
                "field": "period",
                "min": 1,
                "max": 365,
                "received": period,
            }),
        )
    }

    pub fn from_repository(error: RepositoryError, query_failed_code: &'static str) -> Self {
        match error {
            RepositoryError::DatabaseDisabled => {
                Self::service_unavailable("database_unavailable", "Database is not configured")
            }
            RepositoryError::Conflict { message } => Self::conflict("resource_conflict", message),
            other => {
                error!(
                    code = query_failed_code,
                    error = %other,
                    "repository error surfaced as internal API error"
                );
                Self::internal(query_failed_code, other.to_string())
            }
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (self.status, Json(self.body)).into_response()
    }
}

impl<S, T> FromRequest<S> for ApiJson<T>
where
    Json<T>: FromRequest<S, Rejection = JsonRejection>,
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        match Json::<T>::from_request(req, state).await {
            Ok(Json(value)) => Ok(Self(value)),
            Err(rejection) => Err(ApiError::bad_request("invalid_json", rejection.body_text())),
        }
    }
}

impl<S, T> FromRequest<S> for OptionalApiJson<T>
where
    Bytes: FromRequest<S>,
    S: Send + Sync,
    T: DeserializeOwned + Default,
{
    type Rejection = ApiError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let bytes = Bytes::from_request(req, state)
            .await
            .map_err(|_| ApiError::bad_request("invalid_body", "Request body is invalid"))?;

        if bytes.is_empty() {
            return Ok(Self(T::default()));
        }

        serde_json::from_slice(&bytes)
            .map(Self)
            .map_err(|error| ApiError::bad_request("invalid_json", error.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::ApiError;
    use crate::db::repositories::RepositoryError;
    use axum::http::StatusCode;
    use axum::response::IntoResponse;

    #[test]
    fn maps_repository_conflict_to_http_409() {
        let response = ApiError::from_repository(
            RepositoryError::Conflict {
                message: "application already exists for this profile and job".to_string(),
            },
            "applications_query_failed",
        )
        .into_response();

        assert_eq!(response.status(), StatusCode::CONFLICT);
    }
}
