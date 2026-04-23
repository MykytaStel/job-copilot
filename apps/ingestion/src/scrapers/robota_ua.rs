/// Robota.ua scraper — uses the public JSON REST API.
///
/// Confirmed working endpoint (observed via browser DevTools):
///   GET https://api.robota.ua/vacancy/search?page=0&sort=1&count=20&keyWords=...
///
/// Without a keyword we scrape all recent vacancies sorted by date. The caller
/// can pass `--keyword розробник` (or any IT term) to narrow the results.
use std::time::Duration;

use chrono::Utc;
use reqwest::Client;
use scraper::{Html, Selector};
use serde::Deserialize;
use serde_json::json;
use tracing::{info, warn};

use crate::models::{NormalizationResult, NormalizedJob, RawSnapshot};
use crate::scrapers::{
    DetailSnapshot, ScraperConfig, cleanup_description_text, headers::build_default_headers,
    infer_remote_type, infer_seniority, merge_detail_into_result, normalize_company_name,
    normalized_non_empty, parse_salary_range, polite_delay,
};

const SOURCE: &str = "robota_ua";
const API_BASE: &str = "https://api.robota.ua";
const PAGE_SIZE: u32 = 20;

pub struct RobotaUaScraper {
    client: Client,
}

impl RobotaUaScraper {
    pub fn new() -> Result<Self, String> {
        let headers = build_default_headers(
            "application/json, text/plain, */*",
            "uk-UA,uk;q=0.9,en;q=0.8",
            &[
                ("Origin", "https://robota.ua"),
                ("Referer", "https://robota.ua/"),
            ],
        )?;
        let client = Client::builder()
            .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36")
            .timeout(Duration::from_secs(20))
            .default_headers(headers)
            .build()
            .map_err(|e| format!("failed to build HTTP client: {e}"))?;
        Ok(Self { client })
    }

    pub async fn scrape(&self, config: &ScraperConfig) -> Result<Vec<NormalizationResult>, String> {
        let fetched_at = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
        let mut results: Vec<NormalizationResult> = Vec::new();

        // Robota.ua API uses 0-indexed pages
        for page in 0..config.pages {
            let url = build_api_url(config.keyword.as_deref(), page);
            info!(%url, page = page + 1, source = SOURCE, "fetching page");

            let response: ApiResponse = match self.fetch_json(&url).await {
                Ok(r) => r,
                Err(e) => {
                    warn!(error = %e, %url, "fetch failed, stopping pagination");
                    break;
                }
            };

            let docs = response.documents;
            let count = docs.len();

            if count == 0 {
                info!(page = page + 1, source = SOURCE, "empty page, stopping");
                break;
            }

            let mut page_results = Vec::new();
            for v in docs {
                if let Some(result) = normalize_vacancy(v, &fetched_at) {
                    page_results.push(result);
                }
            }

            let enriched = self.enrich_page_results(page_results, &fetched_at).await;
            results.extend(enriched);

            info!(
                page = page + 1,
                jobs = count,
                total = results.len(),
                source = SOURCE,
                "parsed page"
            );

            if page + 1 < config.pages {
                polite_delay(config.page_delay_ms).await;
            }
        }

        info!(total = results.len(), source = SOURCE, "scrape complete");
        Ok(results)
    }

    async fn fetch_json(&self, url: &str) -> Result<ApiResponse, String> {
        let text = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|e| format!("request failed: {e}"))?
            .error_for_status()
            .map_err(|e| format!("HTTP error: {e}"))?
            .text()
            .await
            .map_err(|e| format!("body read failed: {e}"))?;

        serde_json::from_str::<ApiResponse>(&text).map_err(|e| {
            format!(
                "JSON parse failed: {e} — body: {}",
                &text[..text.len().min(300)]
            )
        })
    }

    async fn fetch_html(&self, url: &str) -> Result<String, String> {
        self.client
            .get(url)
            .send()
            .await
            .map_err(|e| format!("request failed: {e}"))?
            .error_for_status()
            .map_err(|e| format!("HTTP error: {e}"))?
            .text()
            .await
            .map_err(|e| format!("body read failed: {e}"))
    }

    async fn enrich_page_results(
        &self,
        results: Vec<NormalizationResult>,
        fetched_at: &str,
    ) -> Vec<NormalizationResult> {
        let total = results.len();
        let mut enriched = Vec::with_capacity(total);

        for (index, result) in results.into_iter().enumerate() {
            match self.enrich_result(result, fetched_at).await {
                Some(result) => enriched.push(result),
                None => warn!(source = SOURCE, "skipped job after detail fetch"),
            }

            if index + 1 < total {
                polite_delay(150).await;
            }
        }

        enriched
    }

    async fn enrich_result(
        &self,
        result: NormalizationResult,
        fetched_at: &str,
    ) -> Option<NormalizationResult> {
        let source_url = result.snapshot.source_url.clone();

        let Some(detail_html) = self.fetch_html(&source_url).await.ok() else {
            if normalize_company_name(&result.job.company_name).is_none() {
                warn!(source = SOURCE, %source_url, "detail fetch failed and company is still missing");
                return None;
            }
            return Some(result);
        };

        let detail = parse_detail_page(&detail_html, &result, fetched_at);
        merge_detail_into_result(result, detail)
    }
}

// ── API response types ────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct ApiResponse {
    #[serde(default)]
    documents: Vec<Vacancy>,
}

/// Maps the fields actually returned by `GET /vacancy/search`.
#[derive(Debug, Deserialize)]
struct Vacancy {
    id: Option<serde_json::Value>,
    /// Vacancy title
    #[serde(rename = "name")]
    name: Option<String>,
    #[serde(rename = "companyName")]
    company_name: Option<String>,
    #[serde(rename = "cityName")]
    city_name: Option<String>,
    /// Short description snippet (may contain HTML entities)
    #[serde(rename = "shortDescription")]
    short_description: Option<String>,
    #[serde(rename = "salaryFrom")]
    salary_from: Option<i32>,
    #[serde(rename = "salaryTo")]
    salary_to: Option<i32>,
    /// Published date — ISO-8601 without timezone, e.g. "2026-04-12T12:01:00.2"
    #[serde(rename = "date")]
    date: Option<String>,
}

impl Vacancy {
    fn numeric_id(&self) -> Option<String> {
        match &self.id {
            Some(serde_json::Value::Number(n)) => Some(n.to_string()),
            Some(serde_json::Value::String(s)) => {
                let part = s.split('-').next().unwrap_or(s);
                if part.chars().all(|c| c.is_ascii_digit()) && !part.is_empty() {
                    Some(part.to_string())
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

// ── URL builder ───────────────────────────────────────────────────────────────

/// Builds a robota.ua API URL.
///
/// Without keyword: sorts all vacancies by date (most recent first).
/// With keyword:    full-text search filtered by the keyword.
fn build_api_url(keyword: Option<&str>, page: u32) -> String {
    let mut params = vec![
        format!("page={page}"),
        "sort=1".to_string(),
        format!("count={PAGE_SIZE}"),
    ];

    match keyword {
        Some(kw) if !kw.trim().is_empty() => {
            params.push(format!("keyWords={}", urlencoded(kw.trim())));
        }
        // Default IT-focused keyword when none is supplied
        _ => params.push("keyWords=developer+engineer+програміст+розробник".to_string()),
    }

    format!("{API_BASE}/vacancy/search?{}", params.join("&"))
}

fn urlencoded(s: &str) -> String {
    s.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' || c == '+' {
                c.to_string()
            } else {
                format!("%{:02X}", c as u32)
            }
        })
        .collect()
}

// ── Normalization ─────────────────────────────────────────────────────────────

fn normalize_vacancy(v: Vacancy, fetched_at: &str) -> Option<NormalizationResult> {
    let source_job_id = v.numeric_id()?;
    let title = v.name.filter(|s| !s.is_empty())?;

    let source_url = format!("https://robota.ua/vacancy/{source_job_id}");
    let company_name = v.company_name.as_deref().and_then(normalize_company_name)?;

    let description_text = v
        .short_description
        .map(|s| {
            // Strip HTML entities and excess whitespace from the snippet
            s.replace("&nbsp;", " ")
                .replace("&amp;", "&")
                .replace("&lt;", "<")
                .replace("&gt;", ">")
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ")
        })
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| title.clone());

    let location = v.city_name.filter(|s| !s.is_empty());

    let (salary_min, salary_max, salary_currency) = match (v.salary_from, v.salary_to) {
        (Some(from), Some(to)) if from > 0 || to > 0 => (
            Some(from).filter(|&n| n > 0),
            Some(to).filter(|&n| n > 0),
            Some("UAH".into()),
        ),
        (Some(from), None) if from > 0 => (Some(from), None, Some("UAH".into())),
        _ => parse_salary_range(&description_text),
    };

    let remote_type = infer_remote_type(&description_text);
    let seniority = infer_seniority(&title);

    // Keep only the date portion of the ISO timestamp
    let posted_at = v
        .date
        .as_deref()
        .and_then(|s| s.get(..10))
        .map(|d| format!("{d}T00:00:00Z"));

    let raw_payload = json!({
        "source_job_id": source_job_id,
        "source_url": source_url,
        "title": title,
        "company_name": company_name,
        "location": location,
        "remote_type": remote_type,
        "seniority": seniority,
        "description_text": description_text,
        "salary_min": salary_min,
        "salary_max": salary_max,
        "salary_currency": salary_currency,
        "posted_at": posted_at,
        "fetched_at": fetched_at,
    });

    Some(NormalizationResult {
        job: NormalizedJob {
            id: format!("job_{SOURCE}_{source_job_id}"),
            title,
            company_name,
            location,
            remote_type,
            seniority,
            description_text,
            salary_min,
            salary_max,
            salary_currency,
            posted_at,
            last_seen_at: fetched_at.to_string(),
            is_active: true,
        },
        snapshot: RawSnapshot {
            source: SOURCE.to_string(),
            source_job_id: source_job_id.to_string(),
            source_url,
            raw_payload,
            fetched_at: fetched_at.to_string(),
        },
    })
}

fn parse_detail_page(
    html: &str,
    fallback: &NormalizationResult,
    fetched_at: &str,
) -> DetailSnapshot {
    let document = Html::parse_document(html);
    let title = extract_first_text(&document, &["h1"]);
    let company_name = extract_first_text(&document, &["a[href*='/company']", "main"])
        .and_then(|value| normalize_company_name(&value));
    let location = extract_first_text(&document, &["main"]).and_then(|value| {
        extract_metadata_value(&value, &[&fallback.job.company_name, &fallback.job.title])
    });
    let description_text = extract_rich_text(&document, &["main", "article"]).map(|value| {
        cleanup_description_text(
            &value,
            title.as_deref().unwrap_or(&fallback.job.title),
            company_name
                .as_deref()
                .unwrap_or(&fallback.job.company_name),
            &[
                "Гарячі вакансії",
                "Схожі вакансії",
                "Відгукнутись",
                "Відгукнутися",
                "Підписатися",
            ],
        )
    });
    let salary_source = extract_first_text(&document, &["main"]);
    let (salary_min, salary_max, salary_currency) = salary_source
        .as_deref()
        .map(parse_salary_range)
        .unwrap_or((None, None, None));

    let signal_text = [
        title.clone().unwrap_or_default(),
        company_name.clone().unwrap_or_default(),
        location.clone().unwrap_or_default(),
        description_text.clone().unwrap_or_default(),
    ]
    .join(" ");

    DetailSnapshot {
        title,
        company_name,
        location,
        remote_type: infer_remote_type(&signal_text),
        seniority: infer_seniority(&signal_text),
        description_text,
        salary_min,
        salary_max,
        salary_currency,
        posted_at: None,
        raw_payload: json!({
            "detail_title": extract_first_text(&document, &["h1"]),
            "detail_company_name": extract_first_text(&document, &["a[href*='/company']", "main"]),
            "detail_description_text": extract_rich_text(&document, &["main", "article"]),
            "detail_salary_text": salary_source,
            "detail_fetched_at": fetched_at,
        }),
    }
}

fn extract_first_text(document: &Html, selectors: &[&str]) -> Option<String> {
    for raw_selector in selectors {
        let Ok(selector) = Selector::parse(raw_selector) else {
            continue;
        };
        if let Some(value) = document
            .select(&selector)
            .map(|element| element.text().collect::<Vec<_>>().join(" "))
            .map(|value| value.split_whitespace().collect::<Vec<_>>().join(" "))
            .find(|value| !value.trim().is_empty())
        {
            return Some(value);
        }
    }

    None
}

fn extract_rich_text(document: &Html, selectors: &[&str]) -> Option<String> {
    let list_selector = Selector::parse("p, li").expect("valid selector");
    let mut best: Option<String> = None;

    for raw_selector in selectors {
        let Ok(selector) = Selector::parse(raw_selector) else {
            continue;
        };

        for container in document.select(&selector) {
            let parts = container
                .select(&list_selector)
                .map(|element| element.text().collect::<Vec<_>>().join(" "))
                .map(|value| value.split_whitespace().collect::<Vec<_>>().join(" "))
                .filter(|value| value.len() > 20)
                .collect::<Vec<_>>();
            let candidate = if parts.is_empty() {
                container
                    .text()
                    .collect::<Vec<_>>()
                    .join(" ")
                    .split_whitespace()
                    .collect::<Vec<_>>()
                    .join(" ")
            } else {
                parts.join(" ")
            };
            if candidate.len() > best.as_ref().map_or(0, String::len) {
                best = Some(candidate);
            }
        }
    }

    best
}

fn extract_metadata_value(value: &str, rejected: &[&str]) -> Option<String> {
    value
        .split("  ")
        .map(str::trim)
        .find(|part| {
            let normalized = normalized_non_empty(Some(part));
            normalized.is_some()
                && rejected.iter().all(|rejected_value| {
                    !part.eq_ignore_ascii_case(rejected_value)
                        && !part.contains("Відгукнути")
                        && !part.contains("Гарячі вакансії")
                })
        })
        .and_then(|part| normalized_non_empty(Some(part)))
}

#[cfg(test)]
mod tests {
    use super::{Vacancy, build_api_url, parse_detail_page};
    use crate::models::{NormalizationResult, NormalizedJob, RawSnapshot};
    use serde_json::json;

    #[test]
    fn builds_url_default_keyword() {
        let url = build_api_url(None, 0);
        assert!(url.contains("/vacancy/search"));
        assert!(url.contains("page=0"));
        assert!(url.contains("keyWords="));
    }

    #[test]
    fn builds_url_with_page() {
        let url = build_api_url(None, 2);
        assert!(url.contains("page=2"));
    }

    #[test]
    fn builds_url_with_custom_keyword() {
        let url = build_api_url(Some("Rust"), 0);
        assert!(url.contains("keyWords=Rust"));
    }

    #[test]
    fn extracts_id_from_json_number() {
        let v = Vacancy {
            id: Some(serde_json::Value::Number(12345678_u64.into())),
            name: Some("Dev".into()),
            company_name: None,
            city_name: None,
            short_description: None,
            salary_from: None,
            salary_to: None,
            date: None,
        };
        assert_eq!(v.numeric_id(), Some("12345678".to_string()));
    }

    #[test]
    fn strips_html_entities_from_description() {
        let v = Vacancy {
            id: Some(serde_json::Value::Number(1_u64.into())),
            name: Some("Dev".into()),
            company_name: Some("Acme".into()),
            city_name: None,
            short_description: Some("Join&nbsp;us &amp; build great things".into()),
            salary_from: None,
            salary_to: None,
            date: None,
        };
        let result = super::normalize_vacancy(v, "2026-04-14T00:00:00Z").unwrap();
        assert_eq!(result.job.description_text, "Join us & build great things");
    }

    #[test]
    fn parses_robota_detail_fixture() {
        let html = include_str!("../../tests/fixtures/robota_ua_detail.html");
        let fallback = NormalizationResult {
            job: NormalizedJob {
                id: "job_robota_ua_123".to_string(),
                title: "Senior Rust Engineer".to_string(),
                company_name: "SignalHire".to_string(),
                location: Some("Kyiv".to_string()),
                remote_type: Some("hybrid".to_string()),
                seniority: Some("senior".to_string()),
                description_text: "Short snippet".to_string(),
                salary_min: None,
                salary_max: None,
                salary_currency: None,
                posted_at: None,
                last_seen_at: "2026-04-18T10:00:00Z".to_string(),
                is_active: true,
            },
            snapshot: RawSnapshot {
                source: "robota_ua".to_string(),
                source_job_id: "123".to_string(),
                source_url: "https://robota.ua/company1/vacancy123".to_string(),
                raw_payload: json!({}),
                fetched_at: "2026-04-18T10:00:00Z".to_string(),
            },
        };

        let detail = parse_detail_page(html, &fallback, "2026-04-18T10:00:00Z");

        assert_eq!(detail.company_name.as_deref(), Some("SignalHire"));
        assert!(
            detail
                .description_text
                .as_deref()
                .is_some_and(|value| value.contains("Build and operate backend services"))
        );
    }
}
