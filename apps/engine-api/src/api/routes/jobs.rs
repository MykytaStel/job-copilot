#[path = "jobs/handlers.rs"]
mod handlers;
#[path = "jobs/helpers.rs"]
mod helpers;

use serde::Deserialize;

use crate::services::matching::RankedJob;
use crate::services::ranking::runtime::ResolvedRerankerRuntime;

pub use handlers::{
    bulk_profile_job_match, get_job_by_id, get_job_fit, get_job_match, get_ml_job_lifecycle,
    get_profile_job_match, get_recent_jobs, score_job_match,
};
pub(crate) use helpers::load_feedback_state;

#[derive(Debug, Deserialize)]
pub struct RecentJobsQuery {
    pub limit: Option<i64>,
    /// Filter by lifecycle stage: "active" | "inactive" | "reactivated"
    pub lifecycle: Option<String>,
    /// Filter by source name: "djinni" | "work_ua" | "robota_ua"
    pub source: Option<crate::domain::source::SourceId>,
    pub profile_id: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
pub struct JobContextQuery {
    pub profile_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct BulkProfileJobMatchRequest {
    pub job_ids: Vec<String>,
}

#[derive(Debug)]
struct BulkProfileJobMatchInput {
    job_ids: Vec<String>,
}

#[derive(Debug, serde::Serialize)]
pub struct BulkProfileJobMatchResponse {
    pub profile_id: String,
    pub results: Vec<crate::api::dto::search::JobFitResponse>,
    pub meta: BulkProfileJobMatchMeta,
}

#[derive(Debug, serde::Serialize)]
pub struct BulkProfileJobMatchMeta {
    pub returned_jobs: usize,
    pub low_evidence_jobs: usize,
    pub weak_description_jobs: usize,
    pub role_mismatch_jobs: usize,
    pub seniority_mismatch_jobs: usize,
    pub source_mismatch_jobs: usize,
    pub top_missing_signals: Vec<String>,
    pub reranker_mode_requested: String,
    pub reranker_mode_active: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reranker_fallback_reason: Option<String>,
}

#[derive(Debug)]
struct ProfileRankedJobsResult {
    ranked_jobs: Vec<RankedJob>,
    reranker_runtime: ResolvedRerankerRuntime,
}

impl BulkProfileJobMatchRequest {
    fn validate(self) -> Result<BulkProfileJobMatchInput, crate::api::error::ApiError> {
        let mut job_ids = Vec::new();

        for job_id in self.job_ids {
            let normalized = job_id.trim();
            if normalized.is_empty() || job_ids.iter().any(|existing| existing == normalized) {
                continue;
            }

            job_ids.push(normalized.to_string());
        }

        if job_ids.is_empty() {
            return Err(crate::api::error::ApiError::bad_request(
                "invalid_job_ids",
                "job_ids must contain at least one non-empty id",
            ));
        }

        if job_ids.len() > 200 {
            return Err(crate::api::error::ApiError::bad_request(
                "invalid_job_ids",
                "job_ids must contain at most 200 ids",
            ));
        }

        Ok(BulkProfileJobMatchInput { job_ids })
    }
}

#[cfg(test)]
#[path = "jobs/tests.rs"]
mod tests;
