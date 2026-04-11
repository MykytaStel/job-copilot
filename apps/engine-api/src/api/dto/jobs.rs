use serde::Serialize;

use crate::domain::job::model::Job;

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
    pub last_seen_at: String,
    pub is_active: bool,
}

#[derive(Debug, Serialize)]
pub struct RecentJobsResponse {
    pub jobs: Vec<JobResponse>,
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
            posted_at: job.posted_at,
            last_seen_at: job.last_seen_at,
            is_active: job.is_active,
        }
    }
}
