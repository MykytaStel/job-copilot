pub mod djinni;
pub mod dou_ua;
mod headers;
pub mod robota_ua;
pub mod work_ua;

use std::time::Duration;

use serde_json::{Value, json};
use tokio::time::sleep;

use crate::models::NormalizationResult;

pub struct ScraperConfig {
    pub pages: u32,
    pub keyword: Option<String>,
    pub page_delay_ms: u64,
}

impl Default for ScraperConfig {
    fn default() -> Self {
        Self {
            pages: 3,
            keyword: None,
            page_delay_ms: 600,
        }
    }
}

pub async fn polite_delay(ms: u64) {
    sleep(Duration::from_millis(ms)).await;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DetailSnapshot {
    pub title: Option<String>,
    pub company_name: Option<String>,
    pub location: Option<String>,
    pub remote_type: Option<String>,
    pub seniority: Option<String>,
    pub description_text: Option<String>,
    pub salary_min: Option<i32>,
    pub salary_max: Option<i32>,
    pub salary_currency: Option<String>,
    pub posted_at: Option<String>,
    pub raw_payload: Value,
}

impl Default for DetailSnapshot {
    fn default() -> Self {
        Self {
            title: None,
            company_name: None,
            location: None,
            remote_type: None,
            seniority: None,
            description_text: None,
            salary_min: None,
            salary_max: None,
            salary_currency: None,
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

    result.job.title = title.clone();
    result.job.company_name = company_name.clone();
    result.job.location = location.clone();
    result.job.remote_type = remote_type.clone();
    result.job.seniority = seniority.clone();
    result.job.description_text = description_text.clone();
    result.job.salary_min = detail.salary_min.or(result.job.salary_min);
    result.job.salary_max = detail.salary_max.or(result.job.salary_max);
    result.job.salary_currency = detail
        .salary_currency
        .or(result.job.salary_currency.clone());
    result.job.posted_at = posted_at.clone();

    result.snapshot.raw_payload = json!({
        "source_job_id": result.snapshot.source_job_id,
        "source_url": result.snapshot.source_url,
        "title": title,
        "company_name": company_name,
        "location": location,
        "remote_type": remote_type,
        "seniority": seniority,
        "description_text": description_text,
        "salary_min": result.job.salary_min,
        "salary_max": result.job.salary_max,
        "salary_currency": result.job.salary_currency,
        "posted_at": posted_at,
        "fetched_at": result.snapshot.fetched_at,
        "listing_snapshot": result.snapshot.raw_payload,
        "detail_snapshot": detail.raw_payload,
    });

    Some(result)
}

pub fn normalized_non_empty(value: Option<&str>) -> Option<String> {
    let cleaned = normalize_text(value?);
    if cleaned.is_empty() {
        None
    } else {
        Some(cleaned)
    }
}

pub fn normalize_company_name(value: &str) -> Option<String> {
    let cleaned = normalize_text(value);
    if cleaned.is_empty() {
        return None;
    }

    let lowered = cleaned.to_lowercase();
    if matches!(
        lowered.as_str(),
        "unknown" | "n/a" | "na" | "company not specified" | "компанія не вказана"
    ) {
        return None;
    }

    Some(cleaned)
}

pub fn normalize_text(value: &str) -> String {
    value
        .replace("&nbsp;", " ")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .trim()
        .to_string()
}

pub fn cleanup_description_text(
    value: &str,
    title: &str,
    company_name: &str,
    cut_markers: &[&str],
) -> String {
    let mut cleaned = normalize_text(value);

    for marker in cut_markers.iter().chain(DESCRIPTION_CUT_MARKERS.iter()) {
        if let Some(head) = truncate_at_marker(&cleaned, marker) {
            cleaned = head.trim().to_string();
        }
    }

    for prefix in [title, company_name] {
        let normalized_prefix = normalize_text(prefix);
        if !normalized_prefix.is_empty() && cleaned.starts_with(&normalized_prefix) {
            cleaned = cleaned[normalized_prefix.len()..].trim().to_string();
        }
    }

    normalize_text(&cleaned)
}

fn truncate_at_marker(value: &str, marker: &str) -> Option<String> {
    let value_lower = value.to_lowercase();
    let marker_lower = marker.to_lowercase();
    let index = value_lower.find(&marker_lower)?;

    Some(value[..index].to_string())
}

fn choose_better_description(
    current: &str,
    candidate: Option<&str>,
    title: &str,
    company_name: &str,
) -> String {
    let current_clean = cleanup_description_text(current, title, company_name, &[]);
    let current_quality = score_description_quality(current, &current_clean, title, company_name);

    let Some(candidate_value) = candidate else {
        return if current_clean.is_empty() {
            title.to_string()
        } else {
            current_clean
        };
    };

    let candidate_clean = cleanup_description_text(candidate_value, title, company_name, &[]);
    if candidate_clean.is_empty() {
        return if current_clean.is_empty() {
            title.to_string()
        } else {
            current_clean
        };
    }

    let candidate_quality =
        score_description_quality(candidate_value, &candidate_clean, title, company_name);

    if current_clean.is_empty() || current_clean.eq_ignore_ascii_case(title) {
        return candidate_clean;
    }

    if candidate_quality >= current_quality + 12 {
        return candidate_clean;
    }

    if !candidate_clean.eq_ignore_ascii_case(&current_clean)
        && candidate_clean.contains(&current_clean)
    {
        return candidate_clean;
    }

    current_clean
}

fn score_description_quality(raw: &str, cleaned: &str, title: &str, company_name: &str) -> usize {
    if cleaned.is_empty() {
        return 0;
    }

    let normalized_title = normalize_text(title).to_lowercase();
    let normalized_company = normalize_text(company_name).to_lowercase();
    let useful_length = cleaned.chars().count();
    let block_count = raw
        .lines()
        .map(str::trim)
        .filter(|line| line.len() > 24)
        .count()
        .max(1);
    let sentence_count = cleaned
        .split(['.', '!', '?', ';', ':'])
        .map(str::trim)
        .filter(|segment| segment.len() > 24)
        .count();
    let unique_terms = cleaned
        .split_whitespace()
        .map(|term| {
            term.trim_matches(|ch: char| !ch.is_alphanumeric())
                .to_lowercase()
        })
        .filter(|term| term.len() > 3)
        .filter(|term| term != &normalized_title && term != &normalized_company)
        .collect::<std::collections::BTreeSet<_>>()
        .len();
    let cleaned_lower = cleaned.to_lowercase();
    let noise_hits = DESCRIPTION_CUT_MARKERS
        .iter()
        .filter(|marker| cleaned_lower.contains(&marker.to_lowercase()))
        .count();

    useful_length / 20 + block_count * 4 + sentence_count * 3 + unique_terms - noise_hits * 8
}

const DESCRIPTION_CUT_MARKERS: &[&str] = &[
    "how to apply",
    "apply now",
    "apply on company website",
    "similar vacancies",
    "related jobs",
    "схожі вакансії",
    "відгукнутися",
    "відгукнутись",
    "правила відгуків",
];

/// Parse a salary string into (min, max, currency).
/// Handles Ukrainian notation ("40 000 грн"), USD ("$3000-5000"), EUR ("€2000").
pub fn parse_salary_range(text: &str) -> (Option<i32>, Option<i32>, Option<String>) {
    let currency = if text.contains('$') || text.to_lowercase().contains("usd") {
        Some("USD".to_string())
    } else if text.contains('€') || text.to_lowercase().contains("eur") {
        Some("EUR".to_string())
    } else if text.contains("грн") || text.to_lowercase().contains("uah") {
        Some("UAH".to_string())
    } else {
        None
    };

    // Strip all whitespace so "40 000" becomes "40000", then extract digit runs.
    let stripped: String = text.chars().filter(|c| !c.is_whitespace()).collect();
    let numbers: Vec<i32> = stripped
        .split(|c: char| !c.is_ascii_digit())
        .filter(|s| s.len() >= 3) // at least 3 digits → ≥100
        .filter_map(|s| s.parse::<i32>().ok())
        .collect();

    match numbers.as_slice() {
        [min, max, ..] => (Some(*min), Some(*max), currency),
        [single] => (Some(*single), None, currency),
        [] => (None, None, currency),
    }
}

pub fn infer_seniority(title: &str) -> Option<String> {
    let t = title.to_lowercase();
    if t.contains("junior") || t.contains("jr.") || t.contains("intern") || t.contains("trainee") {
        Some("junior".to_string())
    } else if t.contains("middle") || t.contains(" mid ") || t.contains("mid-level") {
        Some("middle".to_string())
    } else if t.contains("senior") || t.contains("sr.") {
        Some("senior".to_string())
    } else if t.contains("staff") || t.contains("principal") {
        Some("senior".to_string())
    } else if t.contains(" lead") || t.starts_with("lead ") || t.contains("tech lead") {
        Some("lead".to_string())
    } else {
        None
    }
}

pub fn infer_remote_type(text: &str) -> Option<String> {
    let t = text.to_lowercase();
    if t.contains("remote")
        || t.contains("remotely")
        || t.contains("дистанційно")
        || t.contains("віддален")
    {
        Some("remote".to_string())
    } else if t.contains("hybrid") || t.contains("гібрид") || t.contains("частково") {
        Some("hybrid".to_string())
    } else if t.contains(" office") || t.contains("в офіс") || t.contains("на місці") {
        Some("office".to_string())
    } else {
        None
    }
}

pub fn collect_text(el: &scraper::ElementRef) -> String {
    el.text()
        .collect::<Vec<_>>()
        .join(" ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::models::{NormalizationResult, NormalizedJob, RawSnapshot};

    use super::{DetailSnapshot, cleanup_description_text, merge_detail_into_result};

    #[test]
    fn rejects_results_with_placeholder_company_after_detail_merge() {
        let result = NormalizationResult {
            job: NormalizedJob {
                id: "job-1".to_string(),
                title: "Rust Engineer".to_string(),
                company_name: "Unknown".to_string(),
                location: None,
                remote_type: None,
                seniority: None,
                description_text: "Short snippet".to_string(),
                salary_min: None,
                salary_max: None,
                salary_currency: None,
                posted_at: None,
                last_seen_at: "2026-04-18T10:00:00Z".to_string(),
                is_active: true,
            },
            snapshot: RawSnapshot {
                source: "djinni".to_string(),
                source_job_id: "1".to_string(),
                source_url: "https://example.com/jobs/1".to_string(),
                raw_payload: json!({}),
                fetched_at: "2026-04-18T10:00:00Z".to_string(),
            },
        };

        let merged = merge_detail_into_result(
            result,
            DetailSnapshot {
                description_text: Some("Longer description".to_string()),
                ..DetailSnapshot::default()
            },
        );

        assert!(merged.is_none());
    }

    #[test]
    fn prefers_detail_description_when_it_has_substantially_richer_content() {
        let result = NormalizationResult {
            job: NormalizedJob {
                id: "job-2".to_string(),
                title: "Front-end React Developer".to_string(),
                company_name: "SignalHire".to_string(),
                location: None,
                remote_type: Some("remote".to_string()),
                seniority: Some("senior".to_string()),
                description_text: "React product team".to_string(),
                salary_min: None,
                salary_max: None,
                salary_currency: None,
                posted_at: None,
                last_seen_at: "2026-04-18T10:00:00Z".to_string(),
                is_active: true,
            },
            snapshot: RawSnapshot {
                source: "djinni".to_string(),
                source_job_id: "2".to_string(),
                source_url: "https://example.com/jobs/2".to_string(),
                raw_payload: json!({}),
                fetched_at: "2026-04-18T10:00:00Z".to_string(),
            },
        };

        let merged = merge_detail_into_result(
            result,
            DetailSnapshot {
                description_text: Some(
                    "About the role\nBuild a frontend platform with React and TypeScript.\nPartner with product and design on experiments.".to_string(),
                ),
                ..DetailSnapshot::default()
            },
        )
        .expect("detail merge should succeed");

        assert!(
            merged
                .job
                .description_text
                .contains("Build a frontend platform")
        );
        assert!(
            merged
                .job
                .description_text
                .contains("Partner with product and design")
        );
    }

    #[test]
    fn cleanup_description_text_truncates_apply_blocks_and_related_jobs_noise() {
        let cleaned = cleanup_description_text(
            "Ship accessible frontend features with React and TypeScript. How to apply: send CV. Similar vacancies below.",
            "Senior Front-end React Developer",
            "SignalHire",
            &[],
        );

        assert_eq!(
            cleaned,
            "Ship accessible frontend features with React and TypeScript."
        );
    }
}
