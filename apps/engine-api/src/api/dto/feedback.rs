use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::api::error::ApiError;
use crate::domain::feedback::model::{
    CompanyFeedbackRecord, CompanyFeedbackStatus, JobFeedbackRecord, JobFeedbackState,
};

#[derive(Debug, Deserialize)]
pub struct UpdateCompanyFeedbackRequest {
    pub company_name: String,
}

#[derive(Debug, Serialize)]
pub struct JobFeedbackResponse {
    pub job_id: String,
    pub saved: bool,
    pub hidden: bool,
    pub bad_fit: bool,
    pub updated_at: String,
}

#[derive(Debug, Serialize)]
pub struct CompanyFeedbackResponse {
    pub company_name: String,
    pub normalized_company_name: String,
    pub status: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Default)]
pub struct JobFeedbackStateResponse {
    pub saved: bool,
    pub hidden: bool,
    pub bad_fit: bool,
    pub company_status: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct FeedbackSummary {
    pub saved_jobs_count: usize,
    pub hidden_jobs_count: usize,
    pub bad_fit_jobs_count: usize,
    pub whitelisted_companies_count: usize,
    pub blacklisted_companies_count: usize,
}

#[derive(Debug, Serialize)]
pub struct FeedbackOverviewResponse {
    pub profile_id: String,
    pub jobs: Vec<JobFeedbackResponse>,
    pub companies: Vec<CompanyFeedbackResponse>,
    pub summary: FeedbackSummary,
}

impl UpdateCompanyFeedbackRequest {
    pub fn validate_company_name(self) -> Result<String, ApiError> {
        let company_name = self.company_name.trim();

        if company_name.is_empty() {
            return Err(ApiError::bad_request_with_details(
                "invalid_company_name",
                "company_name must not be empty",
                json!({ "field": "company_name" }),
            ));
        }

        Ok(company_name.to_string())
    }
}

impl From<JobFeedbackRecord> for JobFeedbackResponse {
    fn from(value: JobFeedbackRecord) -> Self {
        Self {
            job_id: value.job_id,
            saved: value.saved,
            hidden: value.hidden,
            bad_fit: value.bad_fit,
            updated_at: value.updated_at,
        }
    }
}

impl From<CompanyFeedbackRecord> for CompanyFeedbackResponse {
    fn from(value: CompanyFeedbackRecord) -> Self {
        Self {
            company_name: value.company_name,
            normalized_company_name: value.normalized_company_name,
            status: value.status.as_str().to_string(),
            updated_at: value.updated_at,
        }
    }
}

impl From<JobFeedbackState> for JobFeedbackStateResponse {
    fn from(value: JobFeedbackState) -> Self {
        Self {
            saved: value.saved,
            hidden: value.hidden,
            bad_fit: value.bad_fit,
            company_status: value
                .company_status
                .map(|status| status.as_str().to_string()),
        }
    }
}

impl From<CompanyFeedbackStatus> for String {
    fn from(value: CompanyFeedbackStatus) -> Self {
        value.as_str().to_string()
    }
}
