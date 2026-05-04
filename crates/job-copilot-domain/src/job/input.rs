use serde::Deserialize;

use crate::job::normalized::NormalizedJob;

#[derive(Debug, Deserialize)]
pub struct IngestionInput {
    pub jobs: Vec<NormalizedJob>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum InputDocument {
    Jobs(Vec<NormalizedJob>),
    Wrapped(IngestionInput),
}

impl InputDocument {
    pub fn into_jobs(self) -> Vec<NormalizedJob> {
        match self {
            InputDocument::Jobs(jobs) => jobs,
            InputDocument::Wrapped(input) => input.jobs,
        }
    }
}

#[cfg(any(feature = "mock", test))]
use serde::Serialize;

#[cfg(any(feature = "mock", test))]
use crate::job::normalized::default_true;

#[cfg(any(feature = "mock", test))]
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct MockSourceInput {
    pub fetched_at: String,
    pub jobs: Vec<MockSourceJob>,
}

#[cfg(any(feature = "mock", test))]
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct MockSourceJob {
    pub source_job_id: String,
    pub source_url: String,
    pub position: String,
    pub employer: String,
    pub city: Option<String>,
    pub work_mode: Option<String>,
    pub seniority: Option<String>,
    pub description: String,
    pub compensation: Option<MockCompensation>,
    pub posted_at: Option<String>,
    pub last_seen_at: String,
    #[serde(default = "default_true")]
    pub active: bool,
}

#[cfg(any(feature = "mock", test))]
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct MockCompensation {
    pub min: Option<i32>,
    pub max: Option<i32>,
    pub currency: Option<String>,
}
