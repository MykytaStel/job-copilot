use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::api::error::ApiError;
use crate::domain::resume::model::{ResumeVersion, UploadResume};

#[derive(Deserialize)]
pub struct UploadResumeRequest {
    pub filename: String,
    pub raw_text: String,
}

#[derive(Debug, Serialize)]
pub struct ResumeVersionResponse {
    pub id: String,
    pub version: i32,
    pub filename: String,
    pub raw_text: String,
    pub is_active: bool,
    pub uploaded_at: String,
}

impl UploadResumeRequest {
    pub fn validate(self) -> Result<UploadResume, ApiError> {
        let filename = self.filename.trim().to_string();
        let raw_text = self.raw_text.trim().to_string();

        if filename.is_empty() {
            return Err(ApiError::bad_request_with_details(
                "invalid_resume_input",
                "Field 'filename' must not be empty",
                json!({ "field": "filename" }),
            ));
        }

        if raw_text.is_empty() {
            return Err(ApiError::bad_request_with_details(
                "invalid_resume_input",
                "Field 'raw_text' must not be empty",
                json!({ "field": "raw_text" }),
            ));
        }

        Ok(UploadResume { filename, raw_text })
    }
}

impl From<ResumeVersion> for ResumeVersionResponse {
    fn from(resume: ResumeVersion) -> Self {
        Self {
            id: resume.id,
            version: resume.version,
            filename: resume.filename,
            raw_text: resume.raw_text,
            is_active: resume.is_active,
            uploaded_at: resume.uploaded_at,
        }
    }
}
