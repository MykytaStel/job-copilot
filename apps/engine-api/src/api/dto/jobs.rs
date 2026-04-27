use crate::domain::matching::JobFit;
use serde::Serialize;

use crate::api::dto::feedback::JobFeedbackStateResponse;
use crate::domain::feedback::model::JobFeedbackState;
use crate::domain::job::age::{is_stale_from_dates, is_stale_job_view};
use crate::domain::job::model::{
    Job, JobFeedSummary, JobLifecycleStage, JobSourceVariant, JobView,
};
use crate::domain::job::presentation::{
    JobPresentation, build_job_presentation, build_job_view_presentation,
};

#[derive(Debug, Serialize)]
pub struct JobSourceVariantResponse {
    pub source: String,
    pub source_url: String,
    pub fetched_at: String,
    pub last_seen_at: String,
    pub is_active: bool,
    pub inactivated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct JobScoreSignalResponse {
    pub label: String,
    pub delta: i32,
}

#[derive(Debug, Serialize)]
pub struct JobPresentationResponse {
    pub title: String,
    pub company: String,
    pub summary: Option<String>,
    pub summary_quality: Option<String>,
    pub summary_fallback: bool,
    pub description_quality: String,
    pub location_label: Option<String>,
    pub work_mode_label: Option<String>,
    pub source_label: Option<String>,
    pub outbound_url: Option<String>,
    pub salary_label: Option<String>,
    pub salary_fit_label: Option<String>,
    pub missing_salary: bool,
    pub freshness_label: Option<String>,
    pub lifecycle_primary_label: Option<String>,
    pub lifecycle_secondary_label: Option<String>,
    pub badges: Vec<String>,
    pub stale: bool,
    pub score_signals: Vec<JobScoreSignalResponse>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum JobLifecycleStageResponse {
    Active,
    Inactive,
    Reactivated,
}

#[derive(Debug, Serialize)]
pub struct JobResponse {
    pub id: String,
    pub title: String,
    pub company_name: String,
    pub location: Option<String>,
    pub remote_type: Option<String>,
    pub seniority: Option<String>,
    pub description_text: String,
    pub salary_min: Option<i32>,
    pub salary_max: Option<i32>,
    pub salary_currency: Option<String>,
    pub posted_at: Option<String>,
    pub first_seen_at: String,
    pub last_seen_at: String,
    pub is_active: bool,
    pub inactivated_at: Option<String>,
    pub reactivated_at: Option<String>,
    pub lifecycle_stage: JobLifecycleStageResponse,
    pub primary_variant: Option<JobSourceVariantResponse>,
    pub presentation: JobPresentationResponse,
    pub stale: bool,
    pub feedback: JobFeedbackStateResponse,
}

#[derive(Debug, Serialize)]
pub struct MlJobLifecycleResponse {
    pub id: String,
    pub title: String,
    pub company_name: String,
    pub location: Option<String>,
    pub remote_type: Option<String>,
    pub seniority: Option<String>,
    pub description_text: String,
    pub salary_min: Option<i32>,
    pub salary_max: Option<i32>,
    pub salary_currency: Option<String>,
    pub posted_at: Option<String>,
    pub first_seen_at: String,
    pub last_seen_at: String,
    pub is_active: bool,
    pub inactivated_at: Option<String>,
    pub reactivated_at: Option<String>,
    pub lifecycle_stage: JobLifecycleStageResponse,
    pub primary_variant: Option<JobSourceVariantResponse>,
    pub presentation: JobPresentationResponse,
    pub stale: bool,
}

#[derive(Debug, Serialize)]
pub struct JobFeedSummaryResponse {
    pub total_jobs: i64,
    pub active_jobs: i64,
    pub inactive_jobs: i64,
    pub reactivated_jobs: i64,
}

#[derive(Debug, Serialize)]
pub struct RecentJobsResponse {
    pub jobs: Vec<JobResponse>,
    pub summary: JobFeedSummaryResponse,
}

impl From<Job> for JobResponse {
    fn from(job: Job) -> Self {
        Self::from_job_with_feedback(job, JobFeedbackState::default())
    }
}

impl From<JobView> for JobResponse {
    fn from(view: JobView) -> Self {
        Self::from_view_with_feedback(view, JobFeedbackState::default())
    }
}

impl JobResponse {
    pub fn from_job_with_feedback(job: Job, feedback: JobFeedbackState) -> Self {
        let first_seen_at = job
            .posted_at
            .clone()
            .unwrap_or_else(|| job.last_seen_at.clone());
        let stale = is_stale_from_dates(Some(&job.last_seen_at), &first_seen_at);

        let mut presentation = JobPresentationResponse::from(build_job_presentation(&job));
        presentation.stale = stale;

        Self {
            id: job.id,
            title: job.title,
            company_name: job.company_name,
            location: job.location,
            remote_type: job.remote_type,
            seniority: job.seniority,
            description_text: job.description_text,
            salary_min: job.salary_min,
            salary_max: job.salary_max,
            salary_currency: job.salary_currency,
            posted_at: job.posted_at,
            first_seen_at,
            last_seen_at: job.last_seen_at,
            is_active: job.is_active,
            inactivated_at: None,
            reactivated_at: None,
            lifecycle_stage: if job.is_active {
                JobLifecycleStageResponse::Active
            } else {
                JobLifecycleStageResponse::Inactive
            },
            primary_variant: None,
            presentation,
            stale,
            feedback: JobFeedbackStateResponse::from(feedback),
        }
    }

    pub fn from_view_with_feedback(view: JobView, feedback: JobFeedbackState) -> Self {
        let stale = is_stale_job_view(&view);

        let mut presentation = JobPresentationResponse::from(build_job_view_presentation(&view));
        presentation.stale = stale;

        Self {
            id: view.job.id,
            title: view.job.title,
            company_name: view.job.company_name,
            location: view.job.location,
            remote_type: view.job.remote_type,
            seniority: view.job.seniority,
            description_text: view.job.description_text,
            salary_min: view.job.salary_min,
            salary_max: view.job.salary_max,
            salary_currency: view.job.salary_currency,
            posted_at: view.job.posted_at,
            first_seen_at: view.first_seen_at,
            last_seen_at: view.job.last_seen_at,
            is_active: view.job.is_active,
            inactivated_at: view.inactivated_at,
            reactivated_at: view.reactivated_at,
            lifecycle_stage: view.lifecycle_stage.into(),
            primary_variant: view.primary_variant.map(JobSourceVariantResponse::from),
            presentation,
            stale,
            feedback: JobFeedbackStateResponse::from(feedback),
        }
    }
}

impl From<JobView> for MlJobLifecycleResponse {
    fn from(view: JobView) -> Self {
        let stale = is_stale_job_view(&view);

        let mut presentation = JobPresentationResponse::from(build_job_view_presentation(&view));
        presentation.stale = stale;

        Self {
            id: view.job.id,
            title: view.job.title,
            company_name: view.job.company_name,
            location: view.job.location,
            remote_type: view.job.remote_type,
            seniority: view.job.seniority,
            description_text: view.job.description_text,
            salary_min: view.job.salary_min,
            salary_max: view.job.salary_max,
            salary_currency: view.job.salary_currency,
            posted_at: view.job.posted_at,
            first_seen_at: view.first_seen_at,
            last_seen_at: view.job.last_seen_at,
            is_active: view.job.is_active,
            inactivated_at: view.inactivated_at,
            reactivated_at: view.reactivated_at,
            lifecycle_stage: view.lifecycle_stage.into(),
            primary_variant: view.primary_variant.map(JobSourceVariantResponse::from),
            presentation,
            stale,
        }
    }
}

impl From<JobLifecycleStage> for JobLifecycleStageResponse {
    fn from(value: JobLifecycleStage) -> Self {
        match value {
            JobLifecycleStage::Active => Self::Active,
            JobLifecycleStage::Inactive => Self::Inactive,
            JobLifecycleStage::Reactivated => Self::Reactivated,
        }
    }
}

impl From<JobSourceVariant> for JobSourceVariantResponse {
    fn from(value: JobSourceVariant) -> Self {
        Self {
            source: value.source,
            source_url: value.source_url,
            fetched_at: value.fetched_at,
            last_seen_at: value.last_seen_at,
            is_active: value.is_active,
            inactivated_at: value.inactivated_at,
        }
    }
}

impl From<JobPresentation> for JobPresentationResponse {
    fn from(value: JobPresentation) -> Self {
        Self {
            title: value.title,
            company: value.company,
            summary: value.summary,
            summary_quality: value
                .summary_quality
                .map(|quality| quality.as_str().to_string()),
            summary_fallback: value.summary_fallback,
            description_quality: value.description_quality.as_str().to_string(),
            location_label: value.location_label,
            work_mode_label: value.work_mode_label,
            source_label: value.source_label,
            outbound_url: value.outbound_url,
            salary_label: value.salary_label,
            salary_fit_label: None,
            missing_salary: false,
            freshness_label: value.freshness_label,
            lifecycle_primary_label: value.lifecycle_primary_label,
            lifecycle_secondary_label: value.lifecycle_secondary_label,
            badges: value.badges,
            stale: false,
            score_signals: Vec::new(),
        }
    }
}

impl From<JobFeedSummary> for JobFeedSummaryResponse {
    fn from(value: JobFeedSummary) -> Self {
        Self {
            total_jobs: value.total_jobs,
            active_jobs: value.active_jobs,
            inactive_jobs: value.inactive_jobs,
            reactivated_jobs: value.reactivated_jobs,
        }
    }
}

pub fn score_signals_from_fit(fit: &JobFit) -> Vec<JobScoreSignalResponse> {
    let breakdown = &fit.score_breakdown;
    let mut signals: Vec<JobScoreSignalResponse> = Vec::new();

    if breakdown.matching_score > 0 {
        signals.push(JobScoreSignalResponse {
            label: "Strong profile match".to_string(),
            delta: i32::from(breakdown.matching_score),
        });
    }

    if breakdown.salary_score != 0 {
        signals.push(JobScoreSignalResponse {
            label: if breakdown.salary_score > 0 {
                "Salary matches expectations".to_string()
            } else {
                "Salary below expectations".to_string()
            },
            delta: i32::from(breakdown.salary_score),
        });
    }

    if breakdown.freshness_score != 0 {
        signals.push(JobScoreSignalResponse {
            label: if breakdown.freshness_score > 0 {
                "Fresh job signal".to_string()
            } else {
                "Job age penalty".to_string()
            },
            delta: i32::from(breakdown.freshness_score),
        });
    }

    if breakdown.reranker_score != 0 {
        signals.push(JobScoreSignalResponse {
            label: "Personalized reranker signal".to_string(),
            delta: i32::from(breakdown.reranker_score),
        });
    }

    for penalty in &breakdown.penalties {
        signals.push(JobScoreSignalResponse {
            label: human_score_signal_label(&penalty.kind, &penalty.reason),
            delta: i32::from(penalty.score_delta),
        });
    }

    signals.sort_by(|left, right| {
        right
            .delta
            .abs()
            .cmp(&left.delta.abs())
            .then_with(|| left.label.cmp(&right.label))
    });

    signals.truncate(3);
    signals
}

fn human_score_signal_label(kind: &str, reason: &str) -> String {
    match kind {
        "company_blacklist" => "Company on blacklist".to_string(),
        "bad_fit_feedback" => "Marked as bad fit".to_string(),
        _ if !reason.trim().is_empty() => reason.to_string(),
        _ => "Score penalty".to_string(),
    }
}
