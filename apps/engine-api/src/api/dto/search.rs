use serde::Serialize;

use crate::api::dto::jobs::JobResponse;
use crate::domain::job::model::Job;

#[derive(Debug, Serialize)]
pub struct SearchResponse {
    pub jobs: Vec<JobResponse>,
    pub contacts: Vec<SearchContactResponse>,
}

#[derive(Debug, Serialize)]
pub struct SearchContactResponse {
    pub id: String,
    pub name: String,
    pub role: Option<String>,
    pub email: Option<String>,
}

impl SearchResponse {
    pub fn from_jobs(jobs: Vec<Job>) -> Self {
        Self {
            jobs: jobs.into_iter().map(JobResponse::from).collect(),
            contacts: Vec::new(),
        }
    }
}
