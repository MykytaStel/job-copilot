use std::time::Duration;

use chrono::Utc;
use reqwest::Client;
use scraper::{ElementRef, Html, Selector};
use serde_json::json;
use tracing::{info, warn};

use crate::models::{NormalizationResult, NormalizedJob, RawSnapshot};
use crate::scrapers::{ScraperConfig, collect_text, infer_remote_type, infer_seniority, parse_salary_range, polite_delay};

const SOURCE: &str = "work_ua";
const BASE_URL: &str = "https://www.work.ua";

pub struct WorkUaScraper {
    client: Client,
}

impl WorkUaScraper {
    pub fn new() -> Result<Self, String> {
        let client = Client::builder()
            .user_agent("Mozilla/5.0 (compatible; JobCopilot/1.0)")
            .timeout(Duration::from_secs(20))
            .build()
            .map_err(|e| format!("failed to build HTTP client: {e}"))?;
        Ok(Self { client })
    }

    pub async fn scrape(&self, config: &ScraperConfig) -> Result<Vec<NormalizationResult>, String> {
        let fetched_at = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
        let selectors = WorkUaSelectors::new();
        let mut results: Vec<NormalizationResult> = Vec::new();

        for page in 1..=config.pages {
            let url = build_url(config.keyword.as_deref(), page);
            info!(%url, page, source = SOURCE, "fetching page");

            let html = match self.fetch(&url).await {
                Ok(html) => html,
                Err(e) => {
                    warn!(error = %e, %url, "fetch failed, stopping pagination");
                    break;
                }
            };

            let page_results = parse_page(&html, &fetched_at, &selectors);
            let count = page_results.len();

            if count == 0 {
                info!(page, source = SOURCE, "no jobs on page, stopping");
                break;
            }

            results.extend(page_results);
            info!(page, jobs = count, total = results.len(), source = SOURCE, "parsed page");

            if page < config.pages {
                polite_delay(config.page_delay_ms).await;
            }
        }

        info!(total = results.len(), source = SOURCE, "scrape complete");
        Ok(results)
    }

    async fn fetch(&self, url: &str) -> Result<String, String> {
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
}

struct WorkUaSelectors {
    item: Selector,
    title: Selector,
    company: Selector,
    meta: Selector,
    description: Selector,
    salary: Selector,
}

impl WorkUaSelectors {
    fn new() -> Self {
        Self {
            // Work.ua renders each vacancy as a card
            item: Selector::parse("div.card.card-hover").expect("valid selector"),
            // Title + job URL
            title: Selector::parse("h2 a").expect("valid selector"),
            // Employer link — work.ua uses /employer/ paths
            company: Selector::parse("a[href*='/employer/']").expect("valid selector"),
            // Location/meta line that follows the company name
            meta: Selector::parse(".add-top-xs").expect("valid selector"),
            // Short description excerpt
            description: Selector::parse("p.cut-words, .text-default-7").expect("valid selector"),
            // Salary label
            salary: Selector::parse("span.label").expect("valid selector"),
        }
    }
}

/// Build a Work.ua listing URL.
///
/// Without keyword: `https://www.work.ua/jobs-it/?page=N`
/// With keyword:    `https://www.work.ua/jobs-it/?q=keyword&page=N`
fn build_url(keyword: Option<&str>, page: u32) -> String {
    let mut params: Vec<String> = Vec::new();

    if let Some(kw) = keyword {
        let kw = kw.trim();
        if !kw.is_empty() {
            params.push(format!("q={kw}"));
        }
    }

    if page > 1 {
        params.push(format!("page={page}"));
    }

    if params.is_empty() {
        format!("{BASE_URL}/jobs-it/")
    } else {
        format!("{BASE_URL}/jobs-it/?{}", params.join("&"))
    }
}

fn parse_page(html: &str, fetched_at: &str, sel: &WorkUaSelectors) -> Vec<NormalizationResult> {
    let document = Html::parse_document(html);
    let mut results = Vec::new();

    for item in document.select(&sel.item) {
        match parse_item(&item, fetched_at, sel) {
            Some(result) => results.push(result),
            None => warn!(source = SOURCE, "skipped job item: missing required fields"),
        }
    }

    results
}

fn parse_item(item: &ElementRef, fetched_at: &str, sel: &WorkUaSelectors) -> Option<NormalizationResult> {
    let title_el = item.select(&sel.title).next()?;
    let href = title_el.value().attr("href")?;
    let title = collect_text(&title_el);
    if title.is_empty() {
        return None;
    }

    let source_job_id = extract_job_id(href)?;
    let source_url = format!("{BASE_URL}{href}");

    let company_name = item
        .select(&sel.company)
        .next()
        .map(|el| collect_text(&el))
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "Unknown".to_string());

    // Location is often in the same `.add-top-xs` line as the company: "CompanyName · Kyiv"
    // Strip the company name from it to get just the location part.
    let location = item
        .select(&sel.meta)
        .next()
        .map(|el| {
            let full = collect_text(&el);
            // Remove the company portion and clean separators
            full.replace(&company_name, "")
                .trim_matches(|c: char| !c.is_alphanumeric() && c != '(' && c != ')')
                .to_string()
        })
        .filter(|s| !s.is_empty() && s.len() >= 2);

    let description_text = item
        .select(&sel.description)
        .next()
        .map(|el| collect_text(&el))
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| title.clone());

    let salary_text = item
        .select(&sel.salary)
        .next()
        .map(|el| collect_text(&el));
    let (salary_min, salary_max, salary_currency) =
        salary_text.as_deref().map(parse_salary_range).unwrap_or((None, None, None));

    let full_text = collect_text(item);
    let remote_type = infer_remote_type(&full_text);
    let seniority = infer_seniority(&title);

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
        "fetched_at": fetched_at,
    });

    Some(NormalizationResult {
        job: NormalizedJob {
            // id is overridden by IngestionBatch::from_normalization_results
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
            posted_at: None, // work.ua shows relative dates ("3 дні тому"), not absolute
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

/// Extract numeric job ID from a Work.ua job URL.
/// "/job/87654321/" → "87654321"
fn extract_job_id(href: &str) -> Option<String> {
    let id = href.trim_matches('/').split('/').last()?;
    if !id.is_empty() && id.chars().all(|c| c.is_ascii_digit()) {
        Some(id.to_string())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::{build_url, extract_job_id};

    #[test]
    fn builds_url_without_params() {
        assert_eq!(build_url(None, 1), "https://www.work.ua/jobs-it/");
    }

    #[test]
    fn builds_url_with_keyword() {
        assert_eq!(
            build_url(Some("Python"), 1),
            "https://www.work.ua/jobs-it/?q=Python"
        );
    }

    #[test]
    fn builds_url_with_page() {
        assert_eq!(
            build_url(None, 2),
            "https://www.work.ua/jobs-it/?page=2"
        );
    }

    #[test]
    fn extracts_job_id_from_url() {
        assert_eq!(
            extract_job_id("/job/87654321/"),
            Some("87654321".to_string())
        );
    }

    #[test]
    fn rejects_non_numeric_job_id() {
        assert_eq!(extract_job_id("/job/some-slug/"), None);
    }
}
