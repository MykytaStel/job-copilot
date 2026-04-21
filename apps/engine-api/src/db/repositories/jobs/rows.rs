use crate::domain::job::model::{Job, JobLifecycleStage, JobSourceVariant, JobView};
use sqlx::FromRow;

#[derive(FromRow)]
pub(super) struct JobRow {
    pub(super) id: String,
    pub(super) title: String,
    pub(super) company_name: String,
    pub(super) location: Option<String>,
    pub(super) remote_type: Option<String>,
    pub(super) seniority: Option<String>,
    pub(super) description_text: String,
    pub(super) salary_min: Option<i32>,
    pub(super) salary_max: Option<i32>,
    pub(super) salary_currency: Option<String>,
    pub(super) posted_at: Option<String>,
    pub(super) last_seen_at: String,
    pub(super) is_active: bool,
}

#[derive(FromRow)]
pub(super) struct JobViewRow {
    pub(super) id: String,
    pub(super) title: String,
    pub(super) company_name: String,
    pub(super) location: Option<String>,
    pub(super) remote_type: Option<String>,
    pub(super) seniority: Option<String>,
    pub(super) description_text: String,
    pub(super) salary_min: Option<i32>,
    pub(super) salary_max: Option<i32>,
    pub(super) salary_currency: Option<String>,
    pub(super) posted_at: Option<String>,
    pub(super) first_seen_at: String,
    pub(super) last_seen_at: String,
    pub(super) is_active: bool,
    pub(super) inactivated_at: Option<String>,
    pub(super) reactivated_at: Option<String>,
    pub(super) variant_source: Option<String>,
    pub(super) variant_source_job_id: Option<String>,
    pub(super) variant_source_url: Option<String>,
    pub(super) variant_raw_payload: Option<serde_json::Value>,
    pub(super) variant_fetched_at: Option<String>,
    pub(super) variant_last_seen_at: Option<String>,
    pub(super) variant_is_active: Option<bool>,
    pub(super) variant_inactivated_at: Option<String>,
}

#[derive(FromRow)]
pub(super) struct JobFeedSummaryRow {
    pub(super) total_jobs: i64,
    pub(super) active_jobs: i64,
    pub(super) inactive_jobs: i64,
    pub(super) reactivated_jobs: i64,
}

impl From<JobRow> for Job {
    fn from(row: JobRow) -> Self {
        Self {
            id: row.id,
            title: row.title,
            company_name: row.company_name,
            location: row.location,
            remote_type: row.remote_type,
            seniority: row.seniority,
            description_text: row.description_text,
            salary_min: row.salary_min,
            salary_max: row.salary_max,
            salary_currency: row.salary_currency,
            posted_at: row.posted_at,
            last_seen_at: row.last_seen_at,
            is_active: row.is_active,
        }
    }
}

impl From<JobViewRow> for JobView {
    fn from(row: JobViewRow) -> Self {
        let lifecycle_stage = if !row.is_active {
            JobLifecycleStage::Inactive
        } else if row
            .reactivated_at
            .as_ref()
            .is_some_and(|value| value == &row.last_seen_at)
        {
            JobLifecycleStage::Reactivated
        } else {
            JobLifecycleStage::Active
        };

        let primary_variant = match (
            row.variant_source,
            row.variant_source_job_id,
            row.variant_source_url,
            row.variant_fetched_at,
            row.variant_last_seen_at,
            row.variant_is_active,
        ) {
            (
                Some(source),
                Some(source_job_id),
                Some(source_url),
                Some(fetched_at),
                Some(last_seen_at),
                Some(is_active),
            ) => Some(JobSourceVariant {
                source,
                source_job_id,
                source_url,
                raw_payload: row.variant_raw_payload,
                fetched_at,
                last_seen_at,
                is_active,
                inactivated_at: row.variant_inactivated_at,
            }),
            _ => None,
        };

        Self {
            job: Job {
                id: row.id,
                title: row.title,
                company_name: row.company_name,
                location: row.location,
                remote_type: row.remote_type,
                seniority: row.seniority,
                description_text: row.description_text,
                salary_min: row.salary_min,
                salary_max: row.salary_max,
                salary_currency: row.salary_currency,
                posted_at: row.posted_at,
                last_seen_at: row.last_seen_at,
                is_active: row.is_active,
            },
            first_seen_at: row.first_seen_at,
            inactivated_at: row.inactivated_at,
            reactivated_at: row.reactivated_at,
            lifecycle_stage,
            primary_variant,
        }
    }
}
