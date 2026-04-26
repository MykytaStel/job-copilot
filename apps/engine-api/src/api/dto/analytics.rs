use serde::Serialize;

use crate::domain::analytics::model::SalaryBucket;
use crate::services::funnel::{FunnelSourceCount, ProfileFunnelSummary};

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
    pub search_quality: SearchQualitySummaryResponse,
}

#[derive(Debug, Serialize)]
pub struct SearchQualitySummaryResponse {
    pub low_evidence_jobs: usize,
    pub weak_description_jobs: usize,
    pub role_mismatch_jobs: usize,
    pub seniority_mismatch_jobs: usize,
    pub source_mismatch_jobs: usize,
    pub top_missing_signals: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct FunnelSourceCountEntry {
    pub source: String,
    pub count: usize,
}

#[derive(Debug, Serialize)]
pub struct FunnelConversionRatesResponse {
    pub open_rate_from_impressions: f64,
    pub save_rate_from_opens: f64,
    pub application_rate_from_saves: f64,
}

#[derive(Debug, Serialize)]
pub struct FunnelSummaryResponse {
    pub profile_id: String,
    pub impression_count: usize,
    pub open_count: usize,
    pub save_count: usize,
    pub hide_count: usize,
    pub bad_fit_count: usize,
    pub application_created_count: usize,
    pub fit_explanation_requested_count: usize,
    pub application_coach_requested_count: usize,
    pub cover_letter_draft_requested_count: usize,
    pub interview_prep_requested_count: usize,
    pub conversion_rates: FunnelConversionRatesResponse,
    pub impressions_by_source: Vec<FunnelSourceCountEntry>,
    pub opens_by_source: Vec<FunnelSourceCountEntry>,
    pub saves_by_source: Vec<FunnelSourceCountEntry>,
    pub applications_by_source: Vec<FunnelSourceCountEntry>,
}

impl FunnelSummaryResponse {
    pub fn from_summary(profile_id: String, summary: ProfileFunnelSummary) -> Self {
        Self {
            profile_id,
            impression_count: summary.impression_count,
            open_count: summary.open_count,
            save_count: summary.save_count,
            hide_count: summary.hide_count,
            bad_fit_count: summary.bad_fit_count,
            application_created_count: summary.application_created_count,
            fit_explanation_requested_count: summary.fit_explanation_requested_count,
            application_coach_requested_count: summary.application_coach_requested_count,
            cover_letter_draft_requested_count: summary.cover_letter_draft_requested_count,
            interview_prep_requested_count: summary.interview_prep_requested_count,
            conversion_rates: FunnelConversionRatesResponse {
                open_rate_from_impressions: summary.conversion_rates.open_rate_from_impressions,
                save_rate_from_opens: summary.conversion_rates.save_rate_from_opens,
                application_rate_from_saves: summary.conversion_rates.application_rate_from_saves,
            },
            impressions_by_source: summary
                .impressions_by_source
                .into_iter()
                .map(FunnelSourceCountEntry::from)
                .collect(),
            opens_by_source: summary
                .opens_by_source
                .into_iter()
                .map(FunnelSourceCountEntry::from)
                .collect(),
            saves_by_source: summary
                .saves_by_source
                .into_iter()
                .map(FunnelSourceCountEntry::from)
                .collect(),
            applications_by_source: summary
                .applications_by_source
                .into_iter()
                .map(FunnelSourceCountEntry::from)
                .collect(),
        }
    }
}

impl From<FunnelSourceCount> for FunnelSourceCountEntry {
    fn from(value: FunnelSourceCount) -> Self {
        Self {
            source: value.source,
            count: value.count,
        }
    }
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

// ─── Ingestion Stats ─────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct IngestionSourceEntry {
    pub source: String,
    pub count: u32,
    pub last_seen: String,
}

#[derive(Debug, Serialize)]
pub struct IngestionStatsResponse {
    pub last_ingested_at: Option<String>,
    pub total_jobs: u32,
    pub active_jobs: u32,
    pub inactive_jobs: u32,
    pub sources: Vec<IngestionSourceEntry>,
}
