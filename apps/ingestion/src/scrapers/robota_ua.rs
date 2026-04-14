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
use serde::Deserialize;
use serde_json::json;
use tracing::{info, warn};

use crate::models::{NormalizationResult, NormalizedJob, RawSnapshot};
use crate::scrapers::{ScraperConfig, infer_remote_type, infer_seniority, parse_salary_range, polite_delay};

const SOURCE: &str = "robota_ua";
const API_BASE: &str = "https://api.robota.ua";
const PAGE_SIZE: u32 = 20;

pub struct RobotaUaScraper {
    client: Client,
}

impl RobotaUaScraper {
    pub fn new() -> Result<Self, String> {
        let client = Client::builder()
            .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36")
            .timeout(Duration::from_secs(20))
            .default_headers({
                let mut h = reqwest::header::HeaderMap::new();
                h.insert("Accept", "application/json, text/plain, */*".parse().unwrap());
                h.insert("Accept-Language", "uk-UA,uk;q=0.9,en;q=0.8".parse().unwrap());
                h.insert("Origin", "https://robota.ua".parse().unwrap());
                h.insert("Referer", "https://robota.ua/".parse().unwrap());
                h
            })
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

            for v in docs {
                if let Some(result) = normalize_vacancy(v, &fetched_at) {
                    results.push(result);
                }
            }

            info!(page = page + 1, jobs = count, total = results.len(), source = SOURCE, "parsed page");

            if page + 1 < config.pages {
                polite_delay(config.page_delay_ms).await;
            }
        }

        info!(total = results.len(), source = SOURCE, "scrape complete");
        Ok(results)
    }

    async fn fetch_json(&self, url: &str) -> Result<ApiResponse, String> {
        let text = self.client
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
            format!("JSON parse failed: {e} — body: {}", &text[..text.len().min(300)])
        })
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
    let company_name = v.company_name.filter(|s| !s.is_empty()).unwrap_or_else(|| "Unknown".into());

    let description_text = v.short_description
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

    let (salary_min, salary_max, salary_currency) =
        match (v.salary_from, v.salary_to) {
            (Some(from), Some(to)) if from > 0 || to > 0 =>
                (Some(from).filter(|&n| n > 0), Some(to).filter(|&n| n > 0), Some("UAH".into())),
            (Some(from), None) if from > 0 =>
                (Some(from), None, Some("UAH".into())),
            _ => parse_salary_range(&description_text),
        };

    let remote_type = infer_remote_type(&description_text);
    let seniority = infer_seniority(&title);

    // Keep only the date portion of the ISO timestamp
    let posted_at = v.date
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

#[cfg(test)]
mod tests {
    use super::{build_api_url, Vacancy};

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
}
