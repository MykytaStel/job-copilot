use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct NormalizedJob {
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
    pub last_seen_at: String,
    #[serde(default = "default_true")]
    pub is_active: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RawSnapshot {
    pub source: String,
    pub source_job_id: String,
    pub source_url: String,
    pub raw_payload: Value,
    pub fetched_at: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct JobVariant {
    pub id: String,
    pub job_id: String,
    pub source: String,
    pub source_job_id: String,
    pub source_url: String,
    pub raw_hash: String,
    pub raw_payload: Value,
    pub fetched_at: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NormalizationResult {
    pub job: NormalizedJob,
    pub snapshot: RawSnapshot,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IngestionBatch {
    pub jobs: Vec<NormalizedJob>,
    pub job_variants: Vec<JobVariant>,
}

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

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct MockSourceInput {
    pub fetched_at: String,
    pub jobs: Vec<MockSourceJob>,
}

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

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct MockCompensation {
    pub min: Option<i32>,
    pub max: Option<i32>,
    pub currency: Option<String>,
}

impl InputDocument {
    pub fn into_jobs(self) -> Vec<NormalizedJob> {
        match self {
            InputDocument::Jobs(jobs) => jobs,
            InputDocument::Wrapped(input) => input.jobs,
        }
    }
}

impl IngestionBatch {
    pub fn from_jobs(jobs: Vec<NormalizedJob>) -> Self {
        Self {
            jobs,
            job_variants: Vec::new(),
        }
    }

    pub fn from_normalization_results(results: Vec<NormalizationResult>) -> Result<Self, String> {
        let mut jobs = Vec::with_capacity(results.len());
        let mut job_variants = Vec::with_capacity(results.len());

        for result in results {
            job_variants.push(JobVariant::from_snapshot(
                result.job.id.clone(),
                result.snapshot,
            )?);
            jobs.push(result.job);
        }

        Ok(Self { jobs, job_variants })
    }
}

impl JobVariant {
    pub fn from_snapshot(job_id: String, snapshot: RawSnapshot) -> Result<Self, String> {
        Ok(Self {
            id: format!("variant_{}_{}", snapshot.source, snapshot.source_job_id),
            job_id,
            source: snapshot.source,
            source_job_id: snapshot.source_job_id,
            source_url: snapshot.source_url,
            raw_hash: compute_raw_hash(&snapshot.raw_payload)?,
            raw_payload: snapshot.raw_payload,
            fetched_at: snapshot.fetched_at,
        })
    }
}

fn compute_raw_hash(value: &Value) -> Result<String, String> {
    let raw_bytes = serde_json::to_vec(value)
        .map_err(|error| format!("failed to serialize raw payload: {error}"))?;
    let digest = Sha256::digest(raw_bytes);
    Ok(digest
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>())
}

pub fn default_true() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{JobVariant, RawSnapshot};

    #[test]
    fn builds_variant_and_hash_from_snapshot() {
        let variant = JobVariant::from_snapshot(
            "job_mock_source_123".to_string(),
            RawSnapshot {
                source: "mock_source".to_string(),
                source_job_id: "123".to_string(),
                source_url: "https://example.com/jobs/123".to_string(),
                raw_payload: json!({
                    "position": "Platform Engineer",
                    "employer": "SignalHire"
                }),
                fetched_at: "2026-04-14T10:00:00Z".to_string(),
            },
        )
        .expect("snapshot should convert into job variant");

        assert_eq!(variant.id, "variant_mock_source_123");
        assert_eq!(variant.job_id, "job_mock_source_123");
        assert_eq!(variant.source, "mock_source");
        assert_eq!(variant.source_job_id, "123");
        assert_eq!(variant.raw_hash.len(), 64);
    }
}
