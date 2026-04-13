use serde::Serialize;

use crate::domain::job::model::{
    Job, JobFeedSummary, JobLifecycleStage, JobSourceVariant, JobView,
};

#[derive(Debug, Serialize)]
pub struct JobSourceVariantResponse {
    pub source: String,
    pub source_job_id: String,
    pub source_url: String,
    pub fetched_at: String,
    pub last_seen_at: String,
    pub is_active: bool,
    pub inactivated_at: Option<String>,
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
}

#[derive(Debug, Serialize)]
pub struct MlJobLifecycleResponse {
    pub id: String,
    pub title: String,
    pub company_name: String,
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
        Self {
            id: job.id,
            title: job.title,
            company_name: job.company_name,
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
        }
    }
}

impl From<JobView> for JobResponse {
    fn from(view: JobView) -> Self {
        Self {
            id: view.job.id,
            title: view.job.title,
            company_name: view.job.company_name,
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
        }
    }
}

impl From<JobView> for MlJobLifecycleResponse {
    fn from(view: JobView) -> Self {
        Self {
            id: view.job.id,
            title: view.job.title,
            company_name: view.job.company_name,
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
            source_job_id: value.source_job_id,
            source_url: value.source_url,
            fetched_at: value.fetched_at,
            last_seen_at: value.last_seen_at,
            is_active: value.is_active,
            inactivated_at: value.inactivated_at,
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
