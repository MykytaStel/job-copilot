use serde_json::{Value, json};

use crate::models::NormalizationResult;

use super::company::{compute_job_quality_score, infer_company_meta, normalize_company_name};
use super::skills::extract_skills;
use super::text::{choose_better_description, normalize_text, normalized_non_empty};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DetailSnapshot {
    pub title: Option<String>,
    pub company_name: Option<String>,
    pub company_url: Option<String>,
    pub location: Option<String>,
    pub remote_type: Option<String>,
    pub seniority: Option<String>,
    pub description_text: Option<String>,
    pub salary_min: Option<i32>,
    pub salary_max: Option<i32>,
    pub salary_currency: Option<String>,
    pub salary_usd_min: Option<i32>,
    pub salary_usd_max: Option<i32>,
    pub posted_at: Option<String>,
    pub raw_payload: Value,
}

impl Default for DetailSnapshot {
    fn default() -> Self {
        Self {
            title: None,
            company_name: None,
            company_url: None,
            location: None,
            remote_type: None,
            seniority: None,
            description_text: None,
            salary_min: None,
            salary_max: None,
            salary_currency: None,
            salary_usd_min: None,
            salary_usd_max: None,
            posted_at: None,
            raw_payload: json!({}),
        }
    }
}

pub fn merge_detail_into_result(
    mut result: NormalizationResult,
    detail: DetailSnapshot,
) -> Option<NormalizationResult> {
    let title = normalized_non_empty(detail.title.as_deref())
        .unwrap_or_else(|| normalize_text(&result.job.title));
    let company_name = detail
        .company_name
        .as_deref()
        .and_then(normalize_company_name)
        .or_else(|| normalize_company_name(&result.job.company_name))?;
    let description_text = choose_better_description(
        &result.job.description_text,
        detail.description_text.as_deref(),
        &title,
        &company_name,
    );
    let location = detail
        .location
        .as_deref()
        .and_then(|value| normalized_non_empty(Some(value)))
        .or_else(|| result.job.location.clone());
    let remote_type = detail
        .remote_type
        .as_deref()
        .and_then(|value| normalized_non_empty(Some(value)))
        .or_else(|| result.job.remote_type.clone());
    let seniority = detail
        .seniority
        .as_deref()
        .and_then(|value| normalized_non_empty(Some(value)))
        .or_else(|| result.job.seniority.clone());
    let posted_at = detail.posted_at.or(result.job.posted_at.clone());
    let company_url = detail.company_url.or_else(|| {
        result
            .job
            .company_meta
            .as_ref()
            .and_then(|meta| meta.url.clone())
    });
    let company_meta = infer_company_meta(&description_text, company_url.as_deref());

    result.job.title = title.clone();
    result.job.company_name = company_name.clone();
    result.job.company_meta = company_meta.clone();
    result.job.location = location.clone();
    result.job.remote_type = remote_type.clone();
    result.job.seniority = seniority.clone();
    result.job.description_text = description_text.clone();
    result.job.extracted_skills = extract_skills(&description_text);
    result.job.salary_min = detail.salary_min.or(result.job.salary_min);
    result.job.salary_max = detail.salary_max.or(result.job.salary_max);
    result.job.salary_currency = detail
        .salary_currency
        .or(result.job.salary_currency.clone());
    result.job.salary_usd_min = detail.salary_usd_min.or(result.job.salary_usd_min);
    result.job.salary_usd_max = detail.salary_usd_max.or(result.job.salary_usd_max);
    result.job.quality_score = Some(compute_job_quality_score(&result.job));
    result.job.posted_at = posted_at.clone();

    result.snapshot.raw_payload = json!({
        "source_job_id": result.snapshot.source_job_id,
        "source_url": result.snapshot.source_url,
        "title": title,
        "company_name": company_name,
        "company_meta": company_meta,
        "location": location,
        "remote_type": remote_type,
        "seniority": seniority,
        "description_text": description_text,
        "extracted_skills": result.job.extracted_skills.clone(),
        "salary_min": result.job.salary_min,
        "salary_max": result.job.salary_max,
        "salary_currency": result.job.salary_currency,
        "salary_usd_min": result.job.salary_usd_min,
        "salary_usd_max": result.job.salary_usd_max,
        "quality_score": result.job.quality_score,
        "posted_at": posted_at,
        "fetched_at": result.snapshot.fetched_at,
        "listing_snapshot": result.snapshot.raw_payload,
        "detail_snapshot": detail.raw_payload,
    });

    Some(result)
}
