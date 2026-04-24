use serde::Serialize;

use crate::api::dto::feedback::JobFeedbackStateResponse;
use crate::domain::feedback::model::JobFeedbackState;
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
    pub freshness_label: Option<String>,
    pub lifecycle_primary_label: Option<String>,
    pub lifecycle_secondary_label: Option<String>,
    pub badges: Vec<String>,
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
        let presentation = JobPresentationResponse::from(build_job_presentation(&job));

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
            posted_at: job.posted_at.clone(),
            first_seen_at: job.posted_at.unwrap_or_else(|| job.last_seen_at.clone()),
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
            feedback: JobFeedbackStateResponse::from(feedback),
        }
    }

    pub fn from_view_with_feedback(view: JobView, feedback: JobFeedbackState) -> Self {
        let presentation = JobPresentationResponse::from(build_job_view_presentation(&view));

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
            feedback: JobFeedbackStateResponse::from(feedback),
        }
    }
}

impl From<JobView> for MlJobLifecycleResponse {
    fn from(view: JobView) -> Self {
        let presentation = JobPresentationResponse::from(build_job_view_presentation(&view));

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
            freshness_label: value.freshness_label,
            lifecycle_primary_label: value.lifecycle_primary_label,
            lifecycle_secondary_label: value.lifecycle_secondary_label,
            badges: value.badges,
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
