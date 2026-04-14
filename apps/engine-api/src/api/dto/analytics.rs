use serde::Serialize;

use crate::domain::analytics::model::SalaryBucket;

#[derive(Debug, Serialize)]
pub struct SalaryBucketResponse {
    pub seniority: Option<String>,
    pub currency: Option<String>,
    pub salary_min: Option<i32>,
    pub salary_max: Option<i32>,
    pub salary_avg: Option<f64>,
    pub job_count: i64,
}

#[derive(Debug, Serialize)]
pub struct SalaryIntelligenceResponse {
    pub buckets: Vec<SalaryBucketResponse>,
}

impl From<SalaryBucket> for SalaryBucketResponse {
    fn from(bucket: SalaryBucket) -> Self {
        Self {
            seniority: bucket.seniority,
            currency: bucket.currency,
            salary_min: bucket.min,
            salary_max: bucket.max,
            salary_avg: bucket.avg,
            job_count: bucket.job_count,
        }
    }
}

// ─── Analytics Summary ───────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct FeedbackSummarySection {
    pub saved_jobs_count: usize,
    pub hidden_jobs_count: usize,
    pub bad_fit_jobs_count: usize,
    pub whitelisted_companies_count: usize,
    pub blacklisted_companies_count: usize,
}

#[derive(Debug, Serialize)]
pub struct JobsByLifecycleSection {
    pub total: i64,
    pub active: i64,
    pub inactive: i64,
    pub reactivated: i64,
}

#[derive(Debug, Serialize)]
pub struct JobsBySourceEntry {
    pub source: String,
    pub count: i64,
}

#[derive(Debug, Serialize)]
pub struct AnalyticsSummaryResponse {
    pub profile_id: String,
    pub feedback: FeedbackSummarySection,
    pub jobs_by_source: Vec<JobsBySourceEntry>,
    pub jobs_by_lifecycle: JobsByLifecycleSection,
    pub top_matched_roles: Vec<String>,
    pub top_matched_skills: Vec<String>,
    pub top_matched_keywords: Vec<String>,
}

// ─── LLM-readiness context ───────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct LlmContextAnalyzedProfile {
    pub summary: String,
    pub primary_role: String,
    pub seniority: String,
    pub skills: Vec<String>,
    pub keywords: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct LlmContextEvidenceEntry {
    #[serde(rename = "type")]
    pub entry_type: String,
    pub label: String,
}

#[derive(Debug, Serialize)]
pub struct LlmContextResponse {
    pub profile_id: String,
    pub analyzed_profile: Option<LlmContextAnalyzedProfile>,
    pub profile_skills: Vec<String>,
    pub profile_keywords: Vec<String>,
    pub jobs_feed_summary: JobsByLifecycleSection,
    pub feedback_summary: FeedbackSummarySection,
    pub top_positive_evidence: Vec<LlmContextEvidenceEntry>,
    pub top_negative_evidence: Vec<LlmContextEvidenceEntry>,
}
