use serde_json::json;

use crate::adapters::SourceAdapter;
use crate::models::{
    MockSourceInput, MockSourceJob, NormalizationResult, NormalizedJob, RawSnapshot,
};

#[derive(Default)]
pub struct MockSourceAdapter;

impl SourceAdapter for MockSourceAdapter {
    type Input = MockSourceInput;

    fn source_name(&self) -> &'static str {
        "mock_source"
    }

    fn normalize(&self, input: Self::Input) -> Result<Vec<NormalizationResult>, String> {
        input
            .jobs
            .into_iter()
            .map(|job| self.normalize_job(&input.fetched_at, job))
            .collect()
    }
}

impl MockSourceAdapter {
    fn normalize_job(
        &self,
        fetched_at: &str,
        job: MockSourceJob,
    ) -> Result<NormalizationResult, String> {
        let title = trim_required("position", job.position)?;
        let company_name = trim_required("employer", job.employer)?;
        let source_job_id = trim_required("source_job_id", job.source_job_id)?;
        let source_url = trim_required("source_url", job.source_url)?;
        let description_text = trim_required("description", job.description)?;
        let last_seen_at = trim_required("last_seen_at", job.last_seen_at)?;

        let snapshot = RawSnapshot {
            source: self.source_name().to_string(),
            source_job_id: source_job_id.clone(),
            source_url: source_url.clone(),
            raw_payload: json!({
                "source_job_id": source_job_id,
                "source_url": source_url,
                "position": title,
                "employer": company_name,
                "city": job.city,
                "work_mode": job.work_mode,
                "seniority": job.seniority,
                "description": description_text,
                "compensation": job.compensation,
                "posted_at": job.posted_at,
                "last_seen_at": last_seen_at,
                "active": job.active,
            }),
            fetched_at: fetched_at.to_string(),
        };

        Ok(NormalizationResult {
            job: NormalizedJob {
                id: format!("job_{}_{}", self.source_name(), snapshot.source_job_id),
                title,
                company_name,
                location: trim_optional(job.city),
                remote_type: trim_optional(job.work_mode),
                seniority: trim_optional(job.seniority),
                description_text,
                salary_min: job.compensation.as_ref().and_then(|value| value.min),
                salary_max: job.compensation.as_ref().and_then(|value| value.max),
                salary_currency: job
                    .compensation
                    .and_then(|value| trim_optional(value.currency)),
                posted_at: trim_optional(job.posted_at),
                last_seen_at,
                is_active: job.active,
            },
            snapshot,
        })
    }
}

fn trim_required(field: &str, value: String) -> Result<String, String> {
    let trimmed = value.trim().to_string();
    if trimmed.is_empty() {
        return Err(format!("field '{field}' must not be empty"));
    }
    Ok(trimmed)
}

fn trim_optional(value: Option<String>) -> Option<String> {
    value.and_then(|value| {
        let trimmed = value.trim().to_string();
        (!trimmed.is_empty()).then_some(trimmed)
    })
}

#[cfg(test)]
mod tests {
    use crate::adapters::SourceAdapter;
    use crate::models::{MockCompensation, MockSourceInput, MockSourceJob};

    use super::MockSourceAdapter;

    #[test]
    fn normalizes_mock_source_jobs_into_canonical_jobs() {
        let adapter = MockSourceAdapter;
        let results = adapter
            .normalize(MockSourceInput {
                fetched_at: "2026-04-14T10:00:00Z".to_string(),
                jobs: vec![MockSourceJob {
                    source_job_id: "123".to_string(),
                    source_url: "https://example.com/jobs/123".to_string(),
                    position: " Senior Backend Engineer ".to_string(),
                    employer: " Nova ".to_string(),
                    city: Some(" Kyiv ".to_string()),
                    work_mode: Some(" remote ".to_string()),
                    seniority: Some(" senior ".to_string()),
                    description: " Rust and Postgres ".to_string(),
                    compensation: Some(MockCompensation {
                        min: Some(4000),
                        max: Some(5500),
                        currency: Some(" USD ".to_string()),
                    }),
                    posted_at: Some("2026-04-14T08:00:00Z".to_string()),
                    last_seen_at: "2026-04-14T09:00:00Z".to_string(),
                    active: true,
                }],
            })
            .expect("mock source input should normalize");

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].job.id, "job_mock_source_123");
        assert_eq!(results[0].job.title, "Senior Backend Engineer");
        assert_eq!(results[0].job.company_name, "Nova");
        assert_eq!(results[0].job.location.as_deref(), Some("Kyiv"));
        assert_eq!(results[0].job.remote_type.as_deref(), Some("remote"));
        assert_eq!(results[0].job.salary_currency.as_deref(), Some("USD"));
        assert_eq!(results[0].snapshot.source, "mock_source");
    }
}
