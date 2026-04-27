#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Job {
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
    pub language: Option<String>,
    pub posted_at: Option<String>,
    pub last_seen_at: String,
    pub is_active: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JobSourceVariant {
    pub source: String,
    pub source_job_id: String,
    pub source_url: String,
    pub raw_payload: Option<serde_json::Value>,
    pub fetched_at: String,
    pub last_seen_at: String,
    pub is_active: bool,
    pub inactivated_at: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JobView {
    pub job: Job,
    pub first_seen_at: String,
    pub inactivated_at: Option<String>,
    pub reactivated_at: Option<String>,
    pub lifecycle_stage: JobLifecycleStage,
    pub primary_variant: Option<JobSourceVariant>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum JobLifecycleStage {
    Active,
    Inactive,
    Reactivated,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JobFeedSummary {
    pub total_jobs: i64,
    pub active_jobs: i64,
    pub inactive_jobs: i64,
    pub reactivated_jobs: i64,
    pub last_ingested_at: Option<String>,
}
