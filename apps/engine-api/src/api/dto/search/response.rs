use serde::Serialize;

use crate::api::dto::applications::ApplicationResponse;
use crate::api::dto::jobs::JobResponse;
use crate::domain::job::model::Job;
use crate::domain::matching::{JobFit, JobScoreBreakdown, JobScorePenalty};
use crate::domain::search::global::ApplicationSearchHit;
use crate::services::search_ranking::{RankedJob, SearchRunResult};

#[derive(Debug, Serialize)]
pub struct SearchResponse {
    pub jobs: Vec<JobResponse>,
    pub applications: Vec<SearchApplicationResponse>,
}

#[derive(Debug, Serialize)]
pub struct SearchApplicationResponse {
    #[serde(flatten)]
    pub application: ApplicationResponse,
    pub job_title: String,
    pub company_name: String,
}

#[derive(Debug, Serialize)]
pub struct RunSearchResponse {
    pub results: Vec<RankedJobResponse>,
    pub meta: SearchRunMetaResponse,
}

#[derive(Debug, Serialize)]
pub struct RankedJobResponse {
    pub job: JobResponse,
    pub fit: JobFitResponse,
}

#[derive(Debug, Serialize)]
pub struct JobFitResponse {
    pub job_id: String,
    pub score: u8,
    pub score_breakdown: JobScoreBreakdownResponse,
    pub matched_roles: Vec<String>,
    pub matched_skills: Vec<String>,
    pub matched_keywords: Vec<String>,
    pub missing_signals: Vec<String>,
    pub source_match: bool,
    pub work_mode_match: Option<bool>,
    pub region_match: Option<bool>,
    pub description_quality: String,
    pub positive_reasons: Vec<String>,
    pub negative_reasons: Vec<String>,
    pub reasons: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct JobScoreBreakdownResponse {
    pub total_score: u8,
    pub matching_score: i16,
    pub salary_score: i16,
    pub reranker_score: i16,
    pub freshness_score: i16,
    pub penalties: Vec<JobScorePenaltyResponse>,
    pub reranker_mode: String,
}

#[derive(Debug, Serialize)]
pub struct JobScorePenaltyResponse {
    pub kind: String,
    pub score_delta: i16,
    pub reason: String,
}

#[derive(Debug, Serialize)]
pub struct SearchRunMetaResponse {
    pub total_candidates: usize,
    pub filtered_out_by_source: usize,
    pub filtered_out_hidden: usize,
    pub filtered_out_company_blacklist: usize,
    pub scored_jobs: usize,
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
    pub learned_reranker_enabled: bool,
    pub learned_reranker_adjusted_jobs: usize,
    pub trained_reranker_enabled: bool,
    pub trained_reranker_adjusted_jobs: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reranker_comparison: Option<SearchRerankerComparisonResponse>,
}

#[derive(Debug, Serialize)]
pub struct SearchRerankerComparisonResponse {
    pub baseline_mode: String,
    pub active_mode: String,
    pub top_n: usize,
    pub baseline_top: Vec<SearchRerankerComparisonItemResponse>,
    pub learned: SearchRerankerComparisonModeResponse,
    pub trained: SearchRerankerComparisonModeResponse,
}

#[derive(Debug, Serialize)]
pub struct SearchRerankerComparisonModeResponse {
    pub active_mode: String,
    pub would_differ_from_baseline: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fallback_reason: Option<String>,
    pub top: Vec<SearchRerankerComparisonItemResponse>,
}

#[derive(Debug, Serialize)]
pub struct SearchRerankerComparisonItemResponse {
    pub job_id: String,
    pub score: u8,
}

impl SearchResponse {
    pub fn new(jobs: Vec<Job>, applications: Vec<ApplicationSearchHit>) -> Self {
        Self {
            jobs: jobs.into_iter().map(JobResponse::from).collect(),
            applications: applications
                .into_iter()
                .map(SearchApplicationResponse::from)
                .collect(),
        }
    }
}

impl From<ApplicationSearchHit> for SearchApplicationResponse {
    fn from(value: ApplicationSearchHit) -> Self {
        Self {
            application: ApplicationResponse {
                id: value.id,
                job_id: value.job_id,
                resume_id: value.resume_id,
                status: value.status,
                applied_at: value.applied_at,
                due_date: value.due_date,
                updated_at: value.updated_at,
            },
            job_title: value.job_title,
            company_name: value.company_name,
        }
    }
}

#[cfg_attr(not(test), allow(dead_code))]
impl RunSearchResponse {
    pub fn from_result(result: SearchRunResult) -> Self {
        let scored_jobs = result
            .total_candidates
            .saturating_sub(result.filtered_out_by_source)
            .saturating_sub(result.filtered_out_hidden)
            .saturating_sub(result.filtered_out_company_blacklist);
        let results = result
            .ranked_jobs
            .into_iter()
            .map(RankedJobResponse::from)
            .collect::<Vec<_>>();
        let returned_jobs = results.len();

        Self {
            results,
            meta: SearchRunMetaResponse {
                total_candidates: result.total_candidates,
                filtered_out_by_source: result.filtered_out_by_source,
                filtered_out_hidden: result.filtered_out_hidden,
                filtered_out_company_blacklist: result.filtered_out_company_blacklist,
                scored_jobs,
                returned_jobs,
                low_evidence_jobs: 0,
                weak_description_jobs: 0,
                role_mismatch_jobs: 0,
                seniority_mismatch_jobs: 0,
                source_mismatch_jobs: 0,
                top_missing_signals: Vec::new(),
                reranker_mode_requested: "deterministic".to_string(),
                reranker_mode_active: "deterministic".to_string(),
                reranker_fallback_reason: None,
                learned_reranker_enabled: false,
                learned_reranker_adjusted_jobs: 0,
                trained_reranker_enabled: false,
                trained_reranker_adjusted_jobs: 0,
                reranker_comparison: None,
            },
        }
    }
}

impl From<RankedJob> for RankedJobResponse {
    fn from(value: RankedJob) -> Self {
        Self {
            job: JobResponse::from(value.job),
            fit: JobFitResponse::from(value.fit),
        }
    }
}

impl From<JobFit> for JobFitResponse {
    fn from(value: JobFit) -> Self {
        let (positive_reasons, negative_reasons) = split_reasons(&value.reasons);

        Self {
            job_id: value.job_id,
            score: value.score,
            score_breakdown: JobScoreBreakdownResponse::from(value.score_breakdown),
            matched_roles: value
                .matched_roles
                .into_iter()
                .map(|role| role.to_string())
                .collect(),
            matched_skills: value.matched_skills,
            matched_keywords: value.matched_keywords,
            missing_signals: value.missing_signals,
            source_match: value.source_match,
            work_mode_match: value.work_mode_match,
            region_match: value.region_match,
            description_quality: value.description_quality.as_str().to_string(),
            positive_reasons,
            negative_reasons,
            reasons: value.reasons,
        }
    }
}

impl From<JobScoreBreakdown> for JobScoreBreakdownResponse {
    fn from(value: JobScoreBreakdown) -> Self {
        Self {
            total_score: value.total_score,
            matching_score: value.matching_score,
            salary_score: value.salary_score,
            reranker_score: value.reranker_score,
            freshness_score: value.freshness_score,
            penalties: value
                .penalties
                .into_iter()
                .map(JobScorePenaltyResponse::from)
                .collect(),
            reranker_mode: value.reranker_mode.as_str().to_string(),
        }
    }
}

impl From<JobScorePenalty> for JobScorePenaltyResponse {
    fn from(value: JobScorePenalty) -> Self {
        Self {
            kind: value.kind,
            score_delta: value.score_delta,
            reason: value.reason,
        }
    }
}

fn split_reasons(reasons: &[String]) -> (Vec<String>, Vec<String>) {
    let mut positive = Vec::new();
    let mut negative = Vec::new();

    for reason in reasons {
        if is_negative_reason(reason) {
            negative.push(reason.clone());
        } else {
            positive.push(reason.clone());
        }
    }

    (positive, negative)
}

fn is_negative_reason(reason: &str) -> bool {
    let normalized = reason.to_lowercase();

    [
        "penalty",
        "mismatch",
        "did not match",
        "no target role signals",
        "no strong profile evidence",
        "could not be inferred",
        "unavailable",
        "bad fit",
        "weak description",
    ]
    .iter()
    .any(|marker| normalized.contains(marker))
}
