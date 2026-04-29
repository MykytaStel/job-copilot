use axum::extract::State;
use axum::http::Request;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode};
use serde::Deserialize;
use tracing::warn;

use crate::api::error::ApiError;
use crate::state::AppState;

/// Returns 403 if `auth` is present and its `profile_id` does not match `profile_id`.
/// When `auth` is absent (JWT not configured) the check is skipped so dev mode works.
pub(crate) fn check_profile_ownership(
    auth: Option<&AuthUser>,
    profile_id: &str,
) -> Result<(), ApiError> {
    if let Some(user) = auth
        && user.profile_id != profile_id
    {
        return Err(ApiError::forbidden(
            "profile_access_denied",
            "You do not have access to this profile",
        ));
    }
    Ok(())
}

/// Identity injected into request extensions after successful JWT validation.
/// Handlers in phase 1.2 will extract this to enforce profile ownership.
#[derive(Clone, Debug)]
pub struct AuthUser {
    pub profile_id: String,
}

#[derive(Debug, Deserialize)]
struct Claims {
    sub: String,
    #[allow(dead_code)]
    exp: usize,
}

pub async fn auth_middleware(
    State(state): State<AppState>,
    mut req: Request<axum::body::Body>,
    next: Next,
) -> Response {
    let Some(ref secret) = state.jwt_secret else {
        warn!("JWT_SECRET not configured; authentication enforcement is disabled");
        return next.run(req).await;
    };

    let auth_header = req
        .headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok());

    let token = match auth_header {
        Some(h) if h.starts_with("Bearer ") => &h["Bearer ".len()..],
        _ => {
            return ApiError::unauthorized(
                "missing_token",
                "Authorization header with Bearer token is required",
            )
            .into_response();
        }
    };

    let token_data = match decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::new(Algorithm::HS256),
    ) {
        Ok(data) => data,
        Err(_) => {
            return ApiError::unauthorized("invalid_token", "Invalid or expired token")
                .into_response();
        }
    };

    req.extensions_mut().insert(AuthUser {
        profile_id: token_data.claims.sub,
    });

    next.run(req).await
}

#[cfg(test)]
mod tests {
    use axum::Router;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use axum::middleware;
    use axum::response::{IntoResponse, Response};
    use axum::routing::get;
    use tower::util::ServiceExt;

    use crate::state::AppState;

    use super::AuthUser;

    async fn echo_profile_id(req: axum::extract::Request) -> String {
        req.extensions()
            .get::<AuthUser>()
            .map(|u| u.profile_id.clone())
            .unwrap_or_else(|| "none".to_string())
    }

    fn app_with_secret(secret: Option<String>) -> Router {
        let mut state = AppState::without_database();
        state.jwt_secret = secret;

        Router::new()
            .route("/test", get(echo_profile_id))
            .route_layer(middleware::from_fn_with_state(
                state.clone(),
                super::auth_middleware,
            ))
            .with_state(state)
    }

    fn make_token(secret: &str, profile_id: &str) -> String {
        use jsonwebtoken::{Algorithm, EncodingKey, Header, encode};
        use serde::Serialize;

        #[derive(Serialize)]
        struct Claims {
            sub: String,
            exp: usize,
        }

        let claims = Claims {
            sub: profile_id.to_string(),
            exp: 9_999_999_999,
        };

        encode(
            &Header::new(Algorithm::HS256),
            &claims,
            &EncodingKey::from_secret(secret.as_bytes()),
        )
        .expect("test token should encode")
    }

    async fn send(app: Router, req: Request<Body>) -> Response<Body> {
        app.oneshot(req).await.expect("request should complete")
    }

    #[tokio::test]
    async fn passes_through_when_no_secret_configured() {
        let app = app_with_secret(None);
        let req = Request::builder().uri("/test").body(Body::empty()).unwrap();
        let resp = send(app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn rejects_missing_authorization_header() {
        let app = app_with_secret(Some("secret".to_string()));
        let req = Request::builder().uri("/test").body(Body::empty()).unwrap();
        let resp = send(app, req).await;
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn rejects_invalid_token() {
        let app = app_with_secret(Some("secret".to_string()));
        let req = Request::builder()
            .uri("/test")
            .header("Authorization", "Bearer notavalidtoken")
            .body(Body::empty())
            .unwrap();
        let resp = send(app, req).await;
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn accepts_valid_token_and_injects_auth_user() {
        let secret = "testsecret";
        let token = make_token(secret, "profile_abc");
        let app = app_with_secret(Some(secret.to_string()));
        let req = Request::builder()
            .uri("/test")
            .header("Authorization", format!("Bearer {token}"))
            .body(Body::empty())
            .unwrap();
        let resp = send(app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let body = axum::body::to_bytes(resp.into_body(), 1024).await.unwrap();
        assert_eq!(body.as_ref(), b"profile_abc");
    }

    #[test]
    fn check_ownership_passes_when_no_auth() {
        assert!(super::check_profile_ownership(None, "any-profile").is_ok());
    }

    #[test]
    fn check_ownership_passes_when_ids_match() {
        let user = AuthUser {
            profile_id: "profile-1".to_string(),
        };
        assert!(super::check_profile_ownership(Some(&user), "profile-1").is_ok());
    }

    #[test]
    fn check_ownership_rejects_mismatched_id() {
        let user = AuthUser {
            profile_id: "profile-1".to_string(),
        };
        let err = super::check_profile_ownership(Some(&user), "profile-2")
            .expect_err("mismatched id should be rejected");
        let resp = err.into_response();
        assert_eq!(resp.status(), axum::http::StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn rejects_token_signed_with_wrong_secret() {
        let token = make_token("wrong_secret", "profile_abc");
        let app = app_with_secret(Some("correct_secret".to_string()));
        let req = Request::builder()
            .uri("/test")
            .header("Authorization", format!("Bearer {token}"))
            .body(Body::empty())
            .unwrap();
        let resp = send(app, req).await;
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }
}
