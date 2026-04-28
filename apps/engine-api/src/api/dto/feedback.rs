use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::api::error::ApiError;
use crate::domain::feedback::model::{
    CompanyFeedbackRecord, CompanyFeedbackStatus, JobFeedbackReason, JobFeedbackRecord,
    JobFeedbackState, LegitimacySignal, SalaryFeedbackSignal, WorkModeFeedbackSignal,
};

#[derive(Debug, Deserialize)]
pub struct UpdateCompanyFeedbackRequest {
    pub company_name: String,
}

#[derive(Debug, Deserialize)]
pub struct BulkHideJobsByCompanyRequest {
    pub company_name: String,
}

#[derive(Debug, Deserialize)]
pub struct SetSalaryFeedbackRequest {
    pub signal: String,
}

#[derive(Debug, Deserialize)]
pub struct SetInterestRatingRequest {
    pub rating: i8,
}

#[derive(Debug, Deserialize)]
pub struct SetWorkModeFeedbackRequest {
    pub signal: String,
}

#[derive(Debug, Deserialize)]
pub struct TagJobFeedbackRequest {
    pub tags: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct SetLegitimacySignalRequest {
    pub signal: String,
}

impl SetSalaryFeedbackRequest {
    pub fn validate(self) -> Result<SalaryFeedbackSignal, ApiError> {
        SalaryFeedbackSignal::parse(&self.signal).ok_or_else(|| {
            ApiError::bad_request_with_details(
                "invalid_salary_signal",
                "Unsupported salary feedback signal",
                json!({
                    "field": "signal",
                    "allowed_values": ["above_expectation", "at_expectation", "below_expectation", "not_shown"],
                    "received": self.signal,
                }),
            )
        })
    }
}

impl SetInterestRatingRequest {
    pub fn validate(self) -> Result<i8, ApiError> {
        if !(-2..=2).contains(&self.rating) {
            return Err(ApiError::bad_request_with_details(
                "invalid_interest_rating",
                "Interest rating must be between -2 and 2",
                json!({ "field": "rating", "received": self.rating }),
            ));
        }
        Ok(self.rating)
    }
}

impl SetWorkModeFeedbackRequest {
    pub fn validate(self) -> Result<WorkModeFeedbackSignal, ApiError> {
        WorkModeFeedbackSignal::parse(&self.signal).ok_or_else(|| {
            ApiError::bad_request_with_details(
                "invalid_work_mode_signal",
                "Unsupported work mode feedback signal",
                json!({
                    "field": "signal",
                    "allowed_values": ["matches_preference", "would_accept", "deal_breaker"],
                    "received": self.signal,
                }),
            )
        })
    }
}

impl TagJobFeedbackRequest {
    pub fn validate(self) -> Result<Vec<JobFeedbackReason>, ApiError> {
        self.tags
            .into_iter()
            .map(|tag| {
                JobFeedbackReason::parse(&tag).ok_or_else(|| {
                    ApiError::bad_request_with_details(
                        "invalid_feedback_tag",
                        format!("Unknown feedback tag: '{tag}'"),
                        json!({ "field": "tags", "received": tag }),
                    )
                })
            })
            .collect()
    }
}

impl SetLegitimacySignalRequest {
    pub fn validate(self) -> Result<LegitimacySignal, ApiError> {
        LegitimacySignal::parse(&self.signal).ok_or_else(|| {
            ApiError::bad_request_with_details(
                "invalid_legitimacy_signal",
                "Unsupported legitimacy signal",
                json!({
                    "field": "signal",
                    "allowed_values": ["looks_real", "suspicious", "spam", "duplicate"],
                    "received": self.signal,
                }),
            )
        })
    }
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
    pub salary_signal: Option<String>,
    pub interest_rating: Option<i8>,
    pub work_mode_signal: Option<String>,
    pub legitimacy_signal: Option<String>,
    pub tags: Vec<String>,
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

#[derive(Debug, Serialize)]
pub struct BulkFeedbackActionResponse {
    pub affected_count: u64,
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

impl BulkHideJobsByCompanyRequest {
    pub fn validate_company_name(self) -> Result<String, ApiError> {
        UpdateCompanyFeedbackRequest {
            company_name: self.company_name,
        }
        .validate_company_name()
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
            salary_signal: value.salary_signal.map(|s| s.as_str().to_string()),
            interest_rating: value.interest_rating,
            work_mode_signal: value.work_mode_signal.map(|s| s.as_str().to_string()),
            legitimacy_signal: value.legitimacy_signal.map(|s| s.as_str().to_string()),
            tags: value.tags.iter().map(|t| t.as_str().to_string()).collect(),
        }
    }
}

impl From<CompanyFeedbackStatus> for String {
    fn from(value: CompanyFeedbackStatus) -> Self {
        value.as_str().to_string()
    }
}
