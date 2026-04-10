use serde::Serialize;

use crate::domain::job::model::Job;

#[derive(Serialize)]
pub struct JobResponse {
    pub id: String,
    pub title: String,
    pub company: String,
    pub location: String,
    pub matched_keywords: Vec<String>,
}

impl From<Job> for JobResponse {
    fn from(job: Job) -> Self {
        Self {
            id: job.id,
            title: job.title,
            company: job.company,
            location: job.location,
            matched_keywords: job.matched_keywords,
        }
    }
}
