use serde::{Deserialize, Serialize};

use crate::api::error::ApiError;

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub name: String,
    pub email: String,
    pub raw_text: String,
}

impl RegisterRequest {
    pub fn validate(self) -> Result<Self, ApiError> {
        let name = self.name.trim().to_string();
        let email = self.email.trim().to_ascii_lowercase();
        let raw_text = self.raw_text.trim().to_string();

        if name.is_empty() {
            return Err(ApiError::bad_request("invalid_name", "name is required"));
        }
        if !email.contains('@') {
            return Err(ApiError::bad_request("invalid_email", "email is invalid"));
        }
        if raw_text.is_empty() {
            return Err(ApiError::bad_request(
                "invalid_raw_text",
                "raw_text (CV / profile description) is required",
            ));
        }

        Ok(Self {
            name,
            email,
            raw_text,
        })
    }
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
}

impl LoginRequest {
    pub fn validate(self) -> Result<Self, ApiError> {
        let email = self.email.trim().to_ascii_lowercase();
        if !email.contains('@') {
            return Err(ApiError::bad_request("invalid_email", "email is invalid"));
        }
        Ok(Self { email })
    }
}

#[derive(Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub profile_id: String,
    pub expires_at: String,
}
