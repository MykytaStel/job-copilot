use std::collections::BTreeMap;

use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::job::dedupe::{canonical_job_id, compute_dedupe_key};
use crate::job::normalized::NormalizedJob;

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
    pub dedupe_key: String,
    pub source: String,
    pub source_job_id: String,
    pub source_url: String,
    pub raw_hash: String,
    pub raw_payload: Value,
    pub fetched_at: String,
    pub last_seen_at: String,
    pub is_active: bool,
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

impl JobVariant {
    pub fn from_snapshot(
        job_id: String,
        dedupe_key: String,
        last_seen_at: String,
        is_active: bool,
        snapshot: RawSnapshot,
    ) -> Result<Self, String> {
        Ok(Self {
            id: format!("variant_{}_{}", snapshot.source, snapshot.source_job_id),
            job_id,
            dedupe_key,
            source: snapshot.source,
            source_job_id: snapshot.source_job_id,
            source_url: snapshot.source_url,
            raw_hash: compute_raw_hash(&snapshot.raw_payload)?,
            raw_payload: snapshot.raw_payload,
            fetched_at: snapshot.fetched_at,
            last_seen_at,
            is_active,
        })
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
            let dedupe_key = compute_dedupe_key(&result.job);
            let job_id = canonical_job_id(&dedupe_key);
            let mut job = result.job;
            job.id = job_id.clone();
            job_variants.push(JobVariant::from_snapshot(
                job_id,
                dedupe_key,
                job.last_seen_at.clone(),
                job.is_active,
                result.snapshot,
            )?);
            jobs.push(job);
        }

        Ok(Self { jobs, job_variants })
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.job_variants.is_empty() {
            return Ok(());
        }

        if self.jobs.len() != self.job_variants.len() {
            return Err(format!(
                "adapter-backed batches must contain one job per variant; got {} jobs and {} variants",
                self.jobs.len(),
                self.job_variants.len()
            ));
        }

        let mut seen_variants = BTreeMap::<(&str, &str), usize>::new();

        for (index, (job, variant)) in self.jobs.iter().zip(self.job_variants.iter()).enumerate() {
            let expected_dedupe_key = compute_dedupe_key(job);
            if variant.dedupe_key != expected_dedupe_key {
                return Err(format!(
                    "job variant '{}' at index {} has dedupe_key '{}' but normalized job computes '{}'",
                    variant.id, index, variant.dedupe_key, expected_dedupe_key
                ));
            }

            let expected_job_id = canonical_job_id(&expected_dedupe_key);
            if variant.job_id != expected_job_id {
                return Err(format!(
                    "job variant '{}' at index {} has job_id '{}' but dedupe_key resolves to '{}'",
                    variant.id, index, variant.job_id, expected_job_id
                ));
            }

            if job.last_seen_at != variant.last_seen_at {
                return Err(format!(
                    "job variant '{}' at index {} has last_seen_at '{}' but normalized job has '{}'",
                    variant.id, index, variant.last_seen_at, job.last_seen_at
                ));
            }

            if job.is_active != variant.is_active {
                return Err(format!(
                    "job variant '{}' at index {} has is_active={} but normalized job has {}",
                    variant.id, index, variant.is_active, job.is_active
                ));
            }

            let key = (variant.source.as_str(), variant.source_job_id.as_str());
            if let Some(previous_index) = seen_variants.insert(key, index) {
                return Err(format!(
                    "adapter-backed batch contains duplicate source variant '{}:{}' at indexes {} and {}",
                    variant.source, variant.source_job_id, previous_index, index
                ));
            }
        }

        Ok(())
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

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{IngestionBatch, JobVariant, RawSnapshot};
    use crate::job::dedupe::canonical_job_id;
    use crate::job::normalized::NormalizedJob;

    #[test]
    fn builds_variant_and_hash_from_snapshot() {
        let dedupe_key = "title=platform engineer|company=signalhire|location=kyiv|remote_type=remote|seniority=senior|posted_on=2026-04-14".to_string();
        let variant = JobVariant::from_snapshot(
            canonical_job_id(&dedupe_key),
            dedupe_key.clone(),
            "2026-04-14T09:00:00Z".to_string(),
            true,
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
        assert_eq!(variant.job_id, canonical_job_id(&dedupe_key));
        assert_eq!(variant.dedupe_key, dedupe_key);
        assert_eq!(variant.source, "mock_source");
        assert_eq!(variant.source_job_id, "123");
        assert_eq!(variant.raw_hash.len(), 64);
        assert_eq!(variant.last_seen_at, "2026-04-14T09:00:00Z");
        assert!(variant.is_active);
    }

    #[test]
    fn rejects_duplicate_source_variants_in_adapter_backed_batches() {
        let dedupe_key = "title=platform engineer|company=signalhire|location=kyiv|remote_type=remote|seniority=senior|posted_on=2026-04-14".to_string();
        let batch = IngestionBatch {
            jobs: vec![
                NormalizedJob {
                    id: canonical_job_id(&dedupe_key),
                    duplicate_of: None,
                    title: "Platform Engineer".to_string(),
                    company_name: "SignalHire".to_string(),
                    company_meta: None,
                    location: Some("Kyiv".to_string()),
                    remote_type: Some("remote".to_string()),
                    seniority: Some("senior".to_string()),
                    description_text: "One".to_string(),
                    extracted_skills: Vec::new(),
                    salary_min: None,
                    salary_max: None,
                    salary_currency: None,
                    salary_usd_min: None,
                    salary_usd_max: None,
                    quality_score: None,
                    posted_at: Some("2026-04-14T09:00:00Z".to_string()),
                    last_seen_at: "2026-04-14T10:00:00Z".to_string(),
                    is_active: true,
                },
                NormalizedJob {
                    id: canonical_job_id(&dedupe_key),
                    duplicate_of: None,
                    title: "Platform Engineer".to_string(),
                    company_name: "SignalHire".to_string(),
                    company_meta: None,
                    location: Some("Kyiv".to_string()),
                    remote_type: Some("remote".to_string()),
                    seniority: Some("senior".to_string()),
                    description_text: "Two".to_string(),
                    extracted_skills: Vec::new(),
                    salary_min: None,
                    salary_max: None,
                    salary_currency: None,
                    salary_usd_min: None,
                    salary_usd_max: None,
                    quality_score: None,
                    posted_at: Some("2026-04-14T09:00:00Z".to_string()),
                    last_seen_at: "2026-04-14T10:00:00Z".to_string(),
                    is_active: true,
                },
            ],
            job_variants: vec![
                JobVariant::from_snapshot(
                    canonical_job_id(&dedupe_key),
                    dedupe_key.clone(),
                    "2026-04-14T10:00:00Z".to_string(),
                    true,
                    RawSnapshot {
                        source: "mock_source".to_string(),
                        source_job_id: "123".to_string(),
                        source_url: "https://example.com/jobs/123".to_string(),
                        raw_payload: json!({ "position": "Platform Engineer" }),
                        fetched_at: "2026-04-14T10:00:00Z".to_string(),
                    },
                )
                .expect("first variant should build"),
                JobVariant::from_snapshot(
                    canonical_job_id(&dedupe_key),
                    dedupe_key,
                    "2026-04-14T10:00:00Z".to_string(),
                    true,
                    RawSnapshot {
                        source: "mock_source".to_string(),
                        source_job_id: "123".to_string(),
                        source_url: "https://example.com/jobs/123?dup=1".to_string(),
                        raw_payload: json!({ "position": "Platform Engineer v2" }),
                        fetched_at: "2026-04-14T10:00:00Z".to_string(),
                    },
                )
                .expect("second variant should build"),
            ],
        };

        let error = batch
            .validate()
            .expect_err("duplicate source variants must be rejected");
        assert!(error.contains("duplicate source variant"));
    }

    #[test]
    fn rejects_dedupe_mismatch_between_job_and_variant() {
        let batch = IngestionBatch {
            jobs: vec![NormalizedJob {
                id: "job_1".to_string(),
                duplicate_of: None,
                title: "Platform Engineer".to_string(),
                company_name: "SignalHire".to_string(),
                company_meta: None,
                location: Some("Kyiv".to_string()),
                remote_type: Some("remote".to_string()),
                seniority: Some("senior".to_string()),
                description_text: "One".to_string(),
                extracted_skills: Vec::new(),
                salary_min: None,
                salary_max: None,
                salary_currency: None,
                salary_usd_min: None,
                salary_usd_max: None,
                quality_score: None,
                posted_at: Some("2026-04-14T09:00:00Z".to_string()),
                last_seen_at: "2026-04-14T10:00:00Z".to_string(),
                is_active: true,
            }],
            job_variants: vec![JobVariant {
                id: "variant_mock_source_123".to_string(),
                job_id: canonical_job_id(
                    "title=another role|company=signalhire|location=kyiv|remote_type=remote|seniority=senior|posted_on=2026-04-14",
                ),
                dedupe_key: "title=another role|company=signalhire|location=kyiv|remote_type=remote|seniority=senior|posted_on=2026-04-14".to_string(),
                source: "mock_source".to_string(),
                source_job_id: "123".to_string(),
                source_url: "https://example.com/jobs/123".to_string(),
                raw_hash: "abc".repeat(21) + "a",
                raw_payload: json!({ "position": "Platform Engineer" }),
                fetched_at: "2026-04-14T10:00:00Z".to_string(),
                last_seen_at: "2026-04-14T10:00:00Z".to_string(),
                is_active: true,
            }],
        };

        let error = batch
            .validate()
            .expect_err("mismatched dedupe fingerprints must be rejected");
        assert!(error.contains("computes"));
    }
}
