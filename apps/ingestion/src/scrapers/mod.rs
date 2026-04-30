pub mod djinni;
pub mod dou_ua;
mod headers;
pub mod robota_ua;
pub mod work_ua;

use std::{sync::OnceLock, time::Duration};

use regex::Regex;
use reqwest::Client;
use serde_json::{Value, json};
use tokio::time::sleep;
use tracing::warn;

use crate::models::{CompanyMeta, NormalizationResult, NormalizedJob};

pub struct ScraperConfig {
    pub pages: u32,
    pub keyword: Option<String>,
    pub page_delay_ms: u64,
}

#[derive(Debug, Clone)]
pub struct ScraperRun {
    pub jobs: Vec<NormalizationResult>,
    pub jobs_attempted: u32,
    pub jobs_failed: u32,
    pub errors: Vec<String>,
}

pub fn detail_error_summaries(source: &str, jobs_failed: u32) -> Vec<String> {
    if jobs_failed == 0 {
        Vec::new()
    } else {
        vec![format!(
            "{source}: {jobs_failed} job detail page(s) failed normalization"
        )]
    }
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

/// Fetch a URL with exponential backoff on 429 / 5xx responses.
///
/// Retries up to 3 times. Respects the `Retry-After` header when present.
/// Jitter is deterministic (avoids the `rand` dependency): each retry adds
/// an extra 200 ms on top of the exponential base so concurrent scrapers
/// don't all retry at the exact same instant.
pub async fn fetch_with_backoff(client: &Client, url: &str) -> Result<String, String> {
    const MAX_RETRIES: u32 = 3;
    const BASE_DELAY_MS: u64 = 1_000;

    let mut attempt = 0u32;
    loop {
        let response = client
            .get(url)
            .send()
            .await
            .map_err(|e| format!("request failed: {e}"))?;

        let status = response.status();

        if status.is_success() {
            return response
                .text()
                .await
                .map_err(|e| format!("body read failed: {e}"));
        }

        if status.as_u16() == 429 || status.is_server_error() {
            if attempt >= MAX_RETRIES {
                warn!(
                    status = status.as_u16(),
                    url,
                    retries = MAX_RETRIES,
                    "giving up after max retries due to rate limiting or server error"
                );
                return Err(format!("HTTP {status} after {MAX_RETRIES} retries: {url}"));
            }

            let retry_after_ms = response
                .headers()
                .get("Retry-After")
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.parse::<u64>().ok())
                .map(|secs| secs * 1_000)
                .unwrap_or(BASE_DELAY_MS << attempt); // 1 s, 2 s, 4 s

            // Deterministic jitter: +200 ms per retry level.
            let delay_ms = retry_after_ms + 200 * (attempt as u64 + 1);

            warn!(
                status = status.as_u16(),
                attempt = attempt + 1,
                delay_ms,
                url,
                "rate limited or server error — retrying with backoff"
            );

            sleep(Duration::from_millis(delay_ms)).await;
            attempt += 1;
            continue;
        }

        return Err(format!("HTTP error: {status}: {url}"));
    }
}

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

pub fn infer_company_meta(description: &str, company_url: Option<&str>) -> Option<CompanyMeta> {
    let size_hint = infer_company_size_hint(description);
    let industry_hint = infer_company_industry_hint(description);
    let url = company_url.and_then(|value| normalized_non_empty(Some(value)));

    if size_hint.is_none() && industry_hint.is_none() && url.is_none() {
        None
    } else {
        Some(CompanyMeta {
            size_hint,
            industry_hint,
            url,
        })
    }
}

pub fn infer_company_size_hint(description: &str) -> Option<String> {
    if let Some(captures) = company_employee_range_re().captures(description) {
        let start = captures.get(1)?.as_str().replace([' ', ','], "");
        let end = captures.get(2)?.as_str().replace([' ', ','], "");
        return Some(format!("{start}-{end} employees"));
    }

    if company_startup_re().is_match(description) {
        Some("startup".to_string())
    } else if company_enterprise_re().is_match(description) {
        Some("enterprise".to_string())
    } else {
        None
    }
}

pub fn infer_company_industry_hint(description: &str) -> Option<String> {
    for (label, aliases) in COMPANY_INDUSTRY_DICTIONARY {
        if aliases
            .iter()
            .any(|alias| company_alias_re(alias).is_match(description))
        {
            return Some((*label).to_string());
        }
    }

    None
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

pub fn compute_job_quality_score(job: &NormalizedJob) -> i32 {
    let mut score = 0;

    if normalize_text(&job.description_text).chars().count() >= 200 {
        score += 30;
    }

    if has_salary_info(job) {
        score += 20;
    }

    if job.extracted_skills.len() >= 3 {
        score += 20;
    }

    if normalized_non_empty(job.seniority.as_deref()).is_some() {
        score += 10;
    }

    if normalized_non_empty(job.remote_type.as_deref()).is_some() {
        score += 10;
    }

    if normalize_company_name(&job.company_name).is_some() {
        score += 10;
    }

    score
}

fn has_salary_info(job: &NormalizedJob) -> bool {
    job.salary_min.is_some()
        || job.salary_max.is_some()
        || job.salary_usd_min.is_some()
        || job.salary_usd_max.is_some()
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

const COMPANY_INDUSTRY_DICTIONARY: &[(&str, &[&str])] = &[
    (
        "fintech",
        &["fintech", "financial technology", "payments", "banking"],
    ),
    (
        "edtech",
        &["edtech", "education technology", "e-learning", "elearning"],
    ),
    (
        "e-commerce",
        &[
            "e-commerce",
            "ecommerce",
            "e commerce",
            "marketplace",
            "retail tech",
        ],
    ),
    (
        "outsourcing",
        &[
            "outsourcing",
            "outstaffing",
            "software development company",
            "service company",
        ],
    ),
    (
        "healthtech",
        &["healthtech", "healthcare", "medical technology"],
    ),
    ("gamedev", &["gamedev", "game development", "gaming studio"]),
];

fn company_employee_range_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r"(?iu)\b(\d[\d\s,]{0,8})\s*[-–—]\s*(\d[\d\s,]{0,8})\s*(?:employees|people|specialists|engineers|фахівц(?:ів|і)|співробітник(?:ів|и)|працівник(?:ів|и))\b",
        )
        .expect("valid company employee range regex")
    })
}

fn company_startup_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?iu)(^|[^\p{L}\p{N}])(?:startup|start-up|стартап)($|[^\p{L}\p{N}])")
            .expect("valid startup regex")
    })
}

fn company_enterprise_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?iu)(^|[^\p{L}\p{N}])(?:enterprise|корпорац(?:ія|ії)|міжнародна компанія|large company|global company)($|[^\p{L}\p{N}])")
            .expect("valid enterprise regex")
    })
}

fn company_alias_re(alias: &str) -> Regex {
    let escaped = regex::escape(alias);
    Regex::new(&format!(
        r"(?iu)(^|[^\p{{L}}\p{{N}}]){escaped}($|[^\p{{L}}\p{{N}}])"
    ))
    .expect("valid company alias regex")
}

pub const SKILL_DICTIONARY: &[(&str, &[&str])] = &[
    ("React", &["react", "react.js", "reactjs"]),
    ("Vue", &["vue", "vue.js", "vuejs"]),
    ("TypeScript", &["typescript", "type script"]),
    ("Node.js", &["node.js", "nodejs", "node js"]),
    ("Python", &["python"]),
    ("Rust", &["rust"]),
    ("Go", &["go", "golang"]),
    ("Java", &["java"]),
    ("Kotlin", &["kotlin"]),
    ("PostgreSQL", &["postgresql", "postgres"]),
    ("MongoDB", &["mongodb", "mongo db"]),
    ("Redis", &["redis"]),
    ("Docker", &["docker"]),
    ("Kubernetes", &["kubernetes", "k8s"]),
    ("AWS", &["aws", "amazon web services"]),
    ("GCP", &["gcp", "google cloud platform"]),
    ("Azure", &["azure"]),
    ("Git", &["git"]),
    ("CI/CD", &["ci/cd", "cicd", "ci cd"]),
    ("GraphQL", &["graphql", "graph ql"]),
    ("REST", &["rest", "restful"]),
    ("FastAPI", &["fastapi", "fast api"]),
    ("Django", &["django"]),
    ("Spring Boot", &["spring boot"]),
    ("Terraform", &["terraform"]),
    ("Linux", &["linux"]),
];

pub fn extract_skills(description: &str) -> Vec<String> {
    let searchable = skill_searchable_text(description);
    let mut extracted = Vec::new();

    for (skill, aliases) in SKILL_DICTIONARY {
        if aliases
            .iter()
            .any(|alias| skill_alias_re(alias).is_match(&searchable))
        {
            extracted.push((*skill).to_string());
        }
    }

    extracted
}

fn skill_searchable_text(description: &str) -> String {
    let mut parts = vec![description.to_string()];
    parts.extend(explicit_skill_segments(description));
    normalize_text(&parts.join(" "))
}

fn explicit_skill_segments(description: &str) -> Vec<String> {
    description
        .lines()
        .map(str::trim)
        .filter_map(|line| {
            let (_, value) = line.split_once(':')?;
            if explicit_skill_prefix_re().is_match(line) {
                Some(value.trim().to_string())
            } else {
                None
            }
        })
        .collect()
}

fn explicit_skill_prefix_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?iu)^\s*(?:required|skills|вимоги|технології)\s*:")
            .expect("valid explicit skill prefix regex")
    })
}

fn skill_alias_re(alias: &str) -> Regex {
    let escaped = regex::escape(alias);
    Regex::new(&format!(
        r"(?iu)(^|[^\p{{L}}\p{{N}}]){escaped}($|[^\p{{L}}\p{{N}}])"
    ))
    .expect("valid skill alias regex")
}

pub const EUR_TO_USD_RATE: f64 = 1.10;
pub const UAH_TO_USD_RATE: f64 = 0.024;
pub const HOURLY_TO_MONTHLY_HOURS: f64 = 160.0;
pub const ANNUAL_TO_MONTHLY_DIVISOR: f64 = 12.0;

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

pub fn normalize_salary_to_usd_monthly(
    salary_min: Option<i32>,
    salary_max: Option<i32>,
    salary_currency: Option<&str>,
    source_text: &str,
) -> (Option<i32>, Option<i32>) {
    let Some(currency) = salary_currency else {
        return (None, None);
    };

    let Some(exchange_rate) = usd_exchange_rate(currency) else {
        return (None, None);
    };
    let period_multiplier = salary_period_multiplier(source_text);

    (
        normalize_salary_amount(salary_min, exchange_rate, period_multiplier),
        normalize_salary_amount(salary_max, exchange_rate, period_multiplier),
    )
}

pub fn parse_salary_range_with_usd_monthly(
    text: &str,
) -> (
    Option<i32>,
    Option<i32>,
    Option<String>,
    Option<i32>,
    Option<i32>,
) {
    let (salary_min, salary_max, salary_currency) = parse_salary_range(text);
    let (salary_usd_min, salary_usd_max) =
        normalize_salary_to_usd_monthly(salary_min, salary_max, salary_currency.as_deref(), text);

    (
        salary_min,
        salary_max,
        salary_currency,
        salary_usd_min,
        salary_usd_max,
    )
}

fn normalize_salary_amount(
    amount: Option<i32>,
    exchange_rate: f64,
    period_multiplier: f64,
) -> Option<i32> {
    amount.map(|value| (value as f64 * exchange_rate * period_multiplier).round() as i32)
}

fn usd_exchange_rate(currency: &str) -> Option<f64> {
    match currency.trim().to_uppercase().as_str() {
        "USD" => Some(1.0),
        "EUR" => Some(EUR_TO_USD_RATE),
        "UAH" => Some(UAH_TO_USD_RATE),
        _ => None,
    }
}

fn salary_period_multiplier(text: &str) -> f64 {
    let normalized = text.to_lowercase();

    if hourly_salary_re().is_match(&normalized) {
        HOURLY_TO_MONTHLY_HOURS
    } else if annual_salary_re().is_match(&normalized) {
        1.0 / ANNUAL_TO_MONTHLY_DIVISOR
    } else {
        1.0
    }
}

fn hourly_salary_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r"(?iu)(?:/|\bper\s+|\bза\s+|\bна\s+)(?:hour|hr|годину|год\b)|\b(?:hourly|погодинно)\b",
        )
        .expect("valid hourly salary regex")
    })
}

fn annual_salary_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?iu)(?:/|\bper\s+|\bза\s+|\bна\s+|\bin\s+|\bв\s+)(?:year|yr|annum|рік)|\b(?:annual|annually|yearly|річна)\b")
            .expect("valid annual salary regex")
    })
}

pub fn infer_seniority(title: &str) -> Option<String> {
    infer_seniority_from_title_and_description(title, None)
}

pub fn infer_seniority_from_title_and_description(
    title: &str,
    description: Option<&str>,
) -> Option<String> {
    infer_seniority_from_title(title)
        .or_else(|| description.and_then(infer_seniority_from_description))
}

fn infer_seniority_from_title(title: &str) -> Option<String> {
    if title_lead_re().is_match(title) {
        Some("lead".to_string())
    } else if title_junior_re().is_match(title) {
        Some("junior".to_string())
    } else if title_middle_re().is_match(title) {
        Some("middle".to_string())
    } else if title_senior_re().is_match(title) {
        Some("senior".to_string())
    } else {
        infer_seniority_from_years(title)
    }
}

fn infer_seniority_from_description(description: &str) -> Option<String> {
    if description_junior_re().is_match(description) {
        Some("junior".to_string())
    } else if description_middle_re().is_match(description) {
        Some("middle".to_string())
    } else if description_lead_re().is_match(description) {
        Some("lead".to_string())
    } else if description_senior_re().is_match(description) {
        Some("senior".to_string())
    } else {
        infer_seniority_from_years(description)
    }
}

fn infer_seniority_from_years(text: &str) -> Option<String> {
    if let Some(captures) = years_range_re().captures(text) {
        let start = captures.get(1)?.as_str().parse::<u8>().ok()?;
        let end = captures.get(2)?.as_str().parse::<u8>().ok()?;
        return seniority_from_year_range(start, end);
    }

    let captures = years_plus_re().captures(text)?;
    let years = captures.get(1)?.as_str().parse::<u8>().ok()?;
    seniority_from_years(years)
}

fn seniority_from_year_range(start: u8, end: u8) -> Option<String> {
    if start == 2 && end == 4 {
        Some("middle".to_string())
    } else {
        seniority_from_years(end)
    }
}

fn seniority_from_years(years: u8) -> Option<String> {
    match years {
        0..=1 => Some("junior".to_string()),
        2..=3 => Some("middle".to_string()),
        4..=6 => Some("senior".to_string()),
        _ => Some("lead".to_string()),
    }
}

fn title_junior_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?iu)(?:\bjunior\b|\bjr(?:\.|\b)|\bentry[-\s]?level\b|\bintern\b|\btrainee\b|\bпочатківець\b|\bмолодший\b)")
            .expect("valid junior seniority regex")
    })
}

fn title_middle_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?iu)\b(?:middle|mid[-\s]?level|mid|regular|intermediate|середній)\b")
            .expect("valid middle seniority regex")
    })
}

fn title_senior_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?iu)(?:\bsenior\b|\bsr(?:\.|\b)|\bдосвідчений\b)")
            .expect("valid senior seniority regex")
    })
}

fn title_lead_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?iu)\b(?:tech\s+lead|team\s+lead|head\s+of|principal|staff|lead)\b")
            .expect("valid lead seniority regex")
    })
}

fn description_junior_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| title_junior_re().clone())
}

fn description_middle_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| title_middle_re().clone())
}

fn description_senior_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?iu)(?:\bsenior\b|\bsr(?:\.|\b)|\bдосвідчений\b|\blead\b)")
            .expect("valid description seniority regex")
    })
}

fn description_lead_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?iu)\b(?:tech\s+lead|team\s+lead|head\s+of|principal|staff)\b")
            .expect("valid lead description seniority regex")
    })
}

fn years_plus_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?iu)\b(\d{1,2})\s*\+\s*(?:years?|yrs?|рок(?:и|ів)?)\b")
            .expect("valid plus years regex")
    })
}

fn years_range_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?iu)\b(\d{1,2})\s*[-–]\s*(\d{1,2})\s*(?:years?|yrs?|рок(?:и|ів)?)\b")
            .expect("valid range years regex")
    })
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

    use super::{
        DetailSnapshot, cleanup_description_text, compute_job_quality_score, extract_skills,
        infer_company_industry_hint, infer_company_meta, infer_company_size_hint, infer_seniority,
        infer_seniority_from_title_and_description, merge_detail_into_result,
        normalize_salary_to_usd_monthly,
    };

    #[test]
    fn computes_job_quality_score_from_ingestion_signals() {
        let job = NormalizedJob {
            id: "job-1".to_string(),
            duplicate_of: None,
            title: "Senior Rust Engineer".to_string(),
            company_name: "SignalHire".to_string(),
            company_meta: None,
            location: Some("Kyiv".to_string()),
            remote_type: Some("remote".to_string()),
            seniority: Some("senior".to_string()),
            description_text: "Build reliable Rust APIs with PostgreSQL, Docker, and Kubernetes. "
                .repeat(4),
            extracted_skills: vec![
                "Rust".to_string(),
                "PostgreSQL".to_string(),
                "Docker".to_string(),
            ],
            salary_min: Some(4000),
            salary_max: Some(5500),
            salary_currency: Some("USD".to_string()),
            salary_usd_min: Some(4000),
            salary_usd_max: Some(5500),
            quality_score: None,
            posted_at: None,
            last_seen_at: "2026-04-18T10:00:00Z".to_string(),
            is_active: true,
        };

        assert_eq!(compute_job_quality_score(&job), 100);
    }

    #[test]
    fn quality_score_rejects_generic_company_and_missing_signals() {
        let job = NormalizedJob {
            id: "job-2".to_string(),
            duplicate_of: None,
            title: "Engineer".to_string(),
            company_name: "Unknown".to_string(),
            company_meta: None,
            location: None,
            remote_type: None,
            seniority: None,
            description_text: "Short snippet".to_string(),
            extracted_skills: vec!["Rust".to_string(), "Docker".to_string()],
            salary_min: None,
            salary_max: None,
            salary_currency: None,
            salary_usd_min: None,
            salary_usd_max: None,
            quality_score: None,
            posted_at: None,
            last_seen_at: "2026-04-18T10:00:00Z".to_string(),
            is_active: true,
        };

        assert_eq!(compute_job_quality_score(&job), 0);
    }

    #[test]
    fn infers_seniority_from_english_title_patterns() {
        let cases = [
            ("Junior Rust Developer", Some("junior")),
            ("Backend Jr. Engineer", Some("junior")),
            ("Entry level QA Engineer", Some("junior")),
            ("Frontend Intern", Some("junior")),
            ("Data Trainee", Some("junior")),
            ("Middle Rust Developer", Some("middle")),
            ("Mid-level Frontend Engineer", Some("middle")),
            ("Regular PHP Developer", Some("middle")),
            ("Intermediate QA Engineer", Some("middle")),
            ("Senior Backend Engineer", Some("senior")),
            ("Sr. Data Engineer", Some("senior")),
            ("Lead Software Engineer", Some("lead")),
            ("Tech Lead Rust", Some("lead")),
            ("Team Lead Backend", Some("lead")),
            ("Principal Engineer", Some("lead")),
            ("Staff Software Engineer", Some("lead")),
            ("Head of Engineering", Some("lead")),
        ];

        for (title, expected) in cases {
            assert_eq!(infer_seniority(title).as_deref(), expected, "{title}");
        }
    }

    #[test]
    fn infers_seniority_from_ukrainian_title_patterns() {
        let cases = [
            ("Початківець Python Developer", Some("junior")),
            ("Молодший QA Engineer", Some("junior")),
            ("Середній Java Developer", Some("middle")),
            ("Досвідчений Rust Developer", Some("senior")),
        ];

        for (title, expected) in cases {
            assert_eq!(infer_seniority(title).as_deref(), expected, "{title}");
        }
    }

    #[test]
    fn infers_seniority_from_year_patterns() {
        let cases = [
            (
                "Developer",
                "Requires 0+ years of commercial experience",
                Some("junior"),
            ),
            (
                "Developer",
                "Requires 1+ year of commercial experience",
                Some("junior"),
            ),
            (
                "Developer",
                "Requires 2+ years of commercial experience",
                Some("middle"),
            ),
            (
                "Developer",
                "Requires 3+ years of commercial experience",
                Some("middle"),
            ),
            (
                "Developer",
                "Requires 4+ years of commercial experience",
                Some("senior"),
            ),
            (
                "Developer",
                "Requires 6+ years of commercial experience",
                Some("senior"),
            ),
            (
                "Developer",
                "Requires 7+ years of commercial experience",
                Some("lead"),
            ),
            (
                "Developer",
                "Experience: 2-4 years with React",
                Some("middle"),
            ),
            (
                "Developer",
                "Experience: 5+ років with Rust",
                Some("senior"),
            ),
            (
                "Developer",
                "You will lead backend delivery for the product team",
                Some("senior"),
            ),
            (
                "Developer",
                "Work as a Tech Lead for a distributed engineering team",
                Some("lead"),
            ),
        ];

        for (title, description, expected) in cases {
            assert_eq!(
                infer_seniority_from_title_and_description(title, Some(description)).as_deref(),
                expected,
                "{description}"
            );
        }
    }

    #[test]
    fn title_seniority_takes_precedence_over_description() {
        assert_eq!(
            infer_seniority_from_title_and_description(
                "Junior Backend Engineer",
                Some("Requirements: 7+ years of production experience")
            )
            .as_deref(),
            Some("junior")
        );
    }

    #[test]
    fn infers_company_size_hints_from_description_patterns() {
        assert_eq!(
            infer_company_size_hint("We are a product startup building developer tooling.")
                .as_deref(),
            Some("startup")
        );
        assert_eq!(
            infer_company_size_hint("Join an enterprise platform team serving global customers.")
                .as_deref(),
            Some("enterprise")
        );
        assert_eq!(
            infer_company_size_hint("Our team has 50-200 employees across Europe.").as_deref(),
            Some("50-200 employees")
        );
    }

    #[test]
    fn infers_company_industry_hints_from_common_sectors() {
        assert_eq!(
            infer_company_industry_hint("FinTech platform for card payments").as_deref(),
            Some("fintech")
        );
        assert_eq!(
            infer_company_industry_hint("We build edtech products for online learning").as_deref(),
            Some("edtech")
        );
        assert_eq!(
            infer_company_industry_hint("E-commerce marketplace and retail tech").as_deref(),
            Some("e-commerce")
        );
        assert_eq!(
            infer_company_industry_hint("Outsourcing software development company").as_deref(),
            Some("outsourcing")
        );
    }

    #[test]
    fn builds_nullable_company_meta_with_optional_url() {
        let meta = infer_company_meta(
            "Startup in fintech hiring backend engineers",
            Some("https://example.com/company/acme"),
        )
        .expect("company meta should be inferred");

        assert_eq!(meta.size_hint.as_deref(), Some("startup"));
        assert_eq!(meta.industry_hint.as_deref(), Some("fintech"));
        assert_eq!(
            meta.url.as_deref(),
            Some("https://example.com/company/acme")
        );
        assert!(infer_company_meta("No company hints here", None).is_none());
    }

    #[test]
    fn extracts_skills_case_insensitively_without_duplicates() {
        let skills = extract_skills(
            "We use React, react.js, TypeScript, nodejs, PostgreSQL, Docker and AWS.",
        );

        assert_eq!(
            skills,
            vec![
                "React".to_string(),
                "TypeScript".to_string(),
                "Node.js".to_string(),
                "PostgreSQL".to_string(),
                "Docker".to_string(),
                "AWS".to_string(),
            ]
        );
    }

    #[test]
    fn extracts_skills_from_explicit_english_and_ukrainian_lists() {
        let skills = extract_skills(
            "Required: Python, FastAPI, Redis\nТехнології: Kubernetes, GCP, CI/CD\nВимоги: Git, Linux",
        );

        assert_eq!(
            skills,
            vec![
                "Python".to_string(),
                "Redis".to_string(),
                "Kubernetes".to_string(),
                "GCP".to_string(),
                "Git".to_string(),
                "CI/CD".to_string(),
                "FastAPI".to_string(),
                "Linux".to_string(),
            ]
        );
    }

    #[test]
    fn skill_extraction_avoids_common_substring_false_positives() {
        let skills = extract_skills("Interest in JavaScript and ongoing collaboration is useful.");

        assert!(skills.is_empty());
    }

    #[test]
    fn rejects_results_with_placeholder_company_after_detail_merge() {
        let result = NormalizationResult {
            job: NormalizedJob {
                id: "job-1".to_string(),
                duplicate_of: None,
                title: "Rust Engineer".to_string(),
                company_name: "Unknown".to_string(),
                company_meta: None,
                location: None,
                remote_type: None,
                seniority: None,
                description_text: "Short snippet".to_string(),
                extracted_skills: Vec::new(),
                salary_min: None,
                salary_max: None,
                salary_currency: None,
                salary_usd_min: None,
                salary_usd_max: None,
                quality_score: None,
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
                duplicate_of: None,
                title: "Front-end React Developer".to_string(),
                company_name: "SignalHire".to_string(),
                company_meta: None,
                location: None,
                remote_type: Some("remote".to_string()),
                seniority: Some("senior".to_string()),
                description_text: "React product team".to_string(),
                extracted_skills: vec!["React".to_string()],
                salary_min: None,
                salary_max: None,
                salary_currency: None,
                salary_usd_min: None,
                salary_usd_max: None,
                quality_score: None,
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

    #[test]
    fn normalizes_monthly_eur_salary_to_usd() {
        assert_eq!(
            normalize_salary_to_usd_monthly(Some(2000), Some(3000), Some("EUR"), "€2000-3000"),
            (Some(2200), Some(3300))
        );
    }

    #[test]
    fn normalizes_hourly_uah_salary_to_usd_monthly() {
        assert_eq!(
            normalize_salary_to_usd_monthly(Some(500), Some(700), Some("UAH"), "500-700 грн/год"),
            (Some(1920), Some(2688))
        );
    }

    #[test]
    fn normalizes_annual_usd_salary_to_monthly() {
        assert_eq!(
            normalize_salary_to_usd_monthly(
                Some(120000),
                Some(150000),
                Some("USD"),
                "$120000-$150000 per year"
            ),
            (Some(10000), Some(12500))
        );
    }
}
