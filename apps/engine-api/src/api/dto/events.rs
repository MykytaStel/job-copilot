use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::api::error::ApiError;
use crate::domain::user_event::model::{CreateUserEvent, UserEventRecord, UserEventSummary, UserEventType};

#[derive(Debug, Deserialize)]
pub struct LogUserEventRequest {
    pub event_type: String,
    #[serde(default)]
    pub job_id: Option<String>,
    #[serde(default)]
    pub company_name: Option<String>,
    #[serde(default)]
    pub source: Option<String>,
    #[serde(default)]
    pub role_family: Option<String>,
    #[serde(default)]
    pub payload_json: Option<Value>,
}

#[derive(Debug, Serialize)]
pub struct UserEventResponse {
    pub id: String,
    pub profile_id: String,
    pub event_type: String,
    pub job_id: Option<String>,
    pub company_name: Option<String>,
    pub source: Option<String>,
    pub role_family: Option<String>,
    pub payload_json: Option<Value>,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
pub struct UserEventSummaryResponse {
    pub profile_id: String,
    pub save_count: usize,
    pub hide_count: usize,
    pub bad_fit_count: usize,
    pub search_run_count: usize,
    pub fit_explanation_requested_count: usize,
    pub application_coach_requested_count: usize,
    pub cover_letter_draft_requested_count: usize,
    pub interview_prep_requested_count: usize,
}

impl LogUserEventRequest {
    pub fn validate(self, profile_id: String) -> Result<CreateUserEvent, ApiError> {
        let event_type = UserEventType::parse(&self.event_type).ok_or_else(|| {
            ApiError::bad_request_with_details(
                "invalid_event_type",
                "Unknown event_type value",
                json!({
                    "field": "event_type",
                    "invalid_value": self.event_type,
                    "allowed_values": UserEventType::allowed_values(),
                }),
            )
        })?;

        let payload_json = match self.payload_json {
            Some(Value::Object(map)) => Some(Value::Object(map)),
            Some(_) => {
                return Err(ApiError::bad_request_with_details(
                    "invalid_event_payload",
                    "payload_json must be a JSON object when provided",
                    json!({ "field": "payload_json" }),
                ));
            }
            None => None,
        };

        Ok(CreateUserEvent {
            profile_id,
            event_type,
            job_id: trim_optional(self.job_id),
            company_name: trim_optional(self.company_name),
            source: trim_optional(self.source),
            role_family: trim_optional(self.role_family),
            payload_json,
        })
    }
}

impl From<UserEventRecord> for UserEventResponse {
    fn from(value: UserEventRecord) -> Self {
        Self {
            id: value.id,
            profile_id: value.profile_id,
            event_type: value.event_type.as_str().to_string(),
            job_id: value.job_id,
            company_name: value.company_name,
            source: value.source,
            role_family: value.role_family,
            payload_json: value.payload_json,
            created_at: value.created_at,
        }
    }
}

impl UserEventSummaryResponse {
    pub fn from_summary(profile_id: String, summary: UserEventSummary) -> Self {
        Self {
            profile_id,
            save_count: summary.save_count,
            hide_count: summary.hide_count,
            bad_fit_count: summary.bad_fit_count,
            search_run_count: summary.search_run_count,
            fit_explanation_requested_count: summary.fit_explanation_requested_count,
            application_coach_requested_count: summary.application_coach_requested_count,
            cover_letter_draft_requested_count: summary.cover_letter_draft_requested_count,
            interview_prep_requested_count: summary.interview_prep_requested_count,
        }
    }
}

fn trim_optional(value: Option<String>) -> Option<String> {
    value
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

#[cfg(test)]
mod tests {
    use axum::response::IntoResponse;
    use serde_json::json;

    use super::LogUserEventRequest;

    #[test]
    fn rejects_non_object_payloads() {
        let error = LogUserEventRequest {
            event_type: "job_opened".to_string(),
            job_id: Some("job-1".to_string()),
            company_name: None,
            source: None,
            role_family: None,
            payload_json: Some(json!(["bad"])),
        }
        .validate("profile-1".to_string())
        .expect_err("payload validation should fail");

        let response = error.into_response();
        assert_eq!(response.status(), axum::http::StatusCode::BAD_REQUEST);
    }
}
