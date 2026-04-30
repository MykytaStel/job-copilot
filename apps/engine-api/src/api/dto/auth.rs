use serde::{Deserialize, Serialize};

use crate::api::error::ApiError;

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub name: String,
    pub email: String,
    pub password: String,
    pub raw_text: String,
}

impl RegisterRequest {
    pub fn validate(self) -> Result<Self, ApiError> {
        let name = self.name.trim().to_string();
        let email = self.email.trim().to_ascii_lowercase();
        let password = self.password.trim().to_string();
        let raw_text = self.raw_text.trim().to_string();

        if name.is_empty() {
            return Err(ApiError::bad_request("invalid_name", "name is required"));
        }
        if !email.contains('@') {
            return Err(ApiError::bad_request("invalid_email", "email is invalid"));
        }
        validate_password(&password)?;
        if raw_text.is_empty() {
            return Err(ApiError::bad_request(
                "invalid_raw_text",
                "raw_text (CV / profile description) is required",
            ));
        }

        Ok(Self {
            name,
            email,
            password,
            raw_text,
        })
    }
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

impl LoginRequest {
    pub fn validate(self) -> Result<Self, ApiError> {
        let email = self.email.trim().to_ascii_lowercase();
        if !email.contains('@') {
            return Err(ApiError::bad_request("invalid_email", "email is invalid"));
        }
        let password = self.password.trim().to_string();
        if password.is_empty() {
            return Err(ApiError::bad_request(
                "invalid_password",
                "password is required",
            ));
        }
        Ok(Self { email, password })
    }
}

#[derive(Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub profile_id: String,
    pub expires_at: String,
}

fn validate_password(password: &str) -> Result<(), ApiError> {
    if password.len() < 8 {
        return Err(ApiError::bad_request(
            "weak_password",
            "password must be at least 8 characters",
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use axum::response::IntoResponse;

    #[test]
    fn register_requires_minimum_password_length() {
        let request = super::RegisterRequest {
            name: "Test User".to_string(),
            email: "test@example.com".to_string(),
            password: "short".to_string(),
            raw_text: "Rust developer".to_string(),
        };

        let Err(error) = request.validate() else {
            panic!("short password should fail");
        };
        let response = error.into_response();

        assert_eq!(response.status(), axum::http::StatusCode::BAD_REQUEST);
    }

    #[test]
    fn login_requires_password() {
        let request = super::LoginRequest {
            email: "test@example.com".to_string(),
            password: " ".to_string(),
        };

        let Err(error) = request.validate() else {
            panic!("empty password should fail");
        };
        let response = error.into_response();

        assert_eq!(response.status(), axum::http::StatusCode::BAD_REQUEST);
    }
}
