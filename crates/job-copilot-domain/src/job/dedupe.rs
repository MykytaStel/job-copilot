use sha2::{Digest, Sha256};

use crate::job::normalized::NormalizedJob;

pub fn compute_dedupe_key(job: &NormalizedJob) -> String {
    let posted_on = job
        .posted_at
        .as_deref()
        .map(|value| value.get(..10).unwrap_or(value))
        .unwrap_or("");

    [
        format!("title={}", dedupe_component(&job.title)),
        format!("company={}", dedupe_component(&job.company_name)),
        format!(
            "location={}",
            job.location
                .as_deref()
                .map(dedupe_component)
                .unwrap_or_default()
        ),
        format!(
            "remote_type={}",
            job.remote_type
                .as_deref()
                .map(dedupe_component)
                .unwrap_or_default()
        ),
        format!(
            "seniority={}",
            job.seniority
                .as_deref()
                .map(dedupe_component)
                .unwrap_or_default()
        ),
        format!("posted_on={posted_on}"),
    ]
    .join("|")
}

pub fn canonical_job_id(dedupe_key: &str) -> String {
    let digest = Sha256::digest(dedupe_key.as_bytes());
    let suffix = digest
        .iter()
        .take(12)
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    format!("job_{suffix}")
}

fn dedupe_component(value: &str) -> String {
    value
        .split_whitespace()
        .filter(|chunk| !chunk.is_empty())
        .map(|chunk| chunk.to_ascii_lowercase())
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::{canonical_job_id, compute_dedupe_key};
    use crate::job::normalized::NormalizedJob;

    #[test]
    fn computes_stable_dedupe_key_from_canonical_fields() {
        let left = NormalizedJob {
            id: "left".to_string(),
            duplicate_of: None,
            title: " Senior Platform Engineer ".to_string(),
            company_name: " SignalHire ".to_string(),
            company_meta: None,
            location: Some(" Kyiv ".to_string()),
            remote_type: Some(" Remote ".to_string()),
            seniority: Some(" Senior ".to_string()),
            description_text: "One".to_string(),
            extracted_skills: Vec::new(),
            salary_min: None,
            salary_max: None,
            salary_currency: None,
            salary_usd_min: None,
            salary_usd_max: None,
            quality_score: None,
            posted_at: Some("2026-04-14T09:15:00Z".to_string()),
            last_seen_at: "2026-04-14T10:00:00Z".to_string(),
            is_active: true,
        };
        let right = NormalizedJob {
            id: "right".to_string(),
            duplicate_of: None,
            title: "Senior   Platform Engineer".to_string(),
            company_name: "signalhire".to_string(),
            company_meta: None,
            location: Some("kyiv".to_string()),
            remote_type: Some("remote".to_string()),
            seniority: Some("senior".to_string()),
            description_text: "Two".to_string(),
            extracted_skills: Vec::new(),
            salary_min: Some(1),
            salary_max: Some(2),
            salary_currency: Some("USD".to_string()),
            salary_usd_min: Some(1),
            salary_usd_max: Some(2),
            quality_score: None,
            posted_at: Some("2026-04-14T18:00:00Z".to_string()),
            last_seen_at: "2026-04-14T19:00:00Z".to_string(),
            is_active: false,
        };

        assert_eq!(compute_dedupe_key(&left), compute_dedupe_key(&right));
    }

    #[test]
    fn canonical_job_id_is_stable_and_prefixed() {
        let key = "title=engineer|company=acme|location=|remote_type=|seniority=|posted_on=";
        let id = canonical_job_id(key);
        assert!(id.starts_with("job_"));
        assert_eq!(canonical_job_id(key), id);
    }
}
