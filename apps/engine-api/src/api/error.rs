use axum::Json;
use axum::extract::FromRequest;
use axum::extract::Request;
use axum::extract::rejection::JsonRejection;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use serde_json::{Value, json};

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

    pub fn from_repository(error: RepositoryError, query_failed_code: &'static str) -> Self {
        match error {
            RepositoryError::DatabaseDisabled => {
                Self::service_unavailable("database_unavailable", "Database is not configured")
            }
            other => Self::internal(query_failed_code, other.to_string()),
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
