use chrono::{DateTime, Duration, Utc};
use jsonwebtoken::{Algorithm, EncodingKey, Header, encode};
use serde::Serialize;

use crate::api::error::ApiError;

#[derive(Serialize)]
struct Claims {
    sub: String,
    exp: usize,
}

pub struct IssuedToken {
    pub token: String,
    pub expires_at: DateTime<Utc>,
}

pub fn issue_token(profile_id: &str, secret: &str) -> Result<IssuedToken, ApiError> {
    let expires_at = Utc::now() + Duration::days(7);
    let claims = Claims {
        sub: profile_id.to_string(),
        exp: expires_at.timestamp() as usize,
    };
    let token = encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|_| ApiError::internal("token_encode_failed", "Failed to issue token"))?;

    Ok(IssuedToken { token, expires_at })
}
