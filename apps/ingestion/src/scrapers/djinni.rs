use std::time::Duration;

use chrono::Utc;
use reqwest::Client;
use scraper::{ElementRef, Html, Selector};
use serde_json::json;
use tracing::{info, warn};

use crate::models::{NormalizationResult, NormalizedJob, RawSnapshot};
use crate::scrapers::{ScraperConfig, collect_text, infer_remote_type, infer_seniority, parse_salary_range, polite_delay};

const SOURCE: &str = "djinni";
const BASE_URL: &str = "https://djinni.co";

pub struct DjinniScraper {
    client: Client,
}

impl DjinniScraper {
    pub fn new() -> Result<Self, String> {
        let client = Client::builder()
            .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36")
            .timeout(Duration::from_secs(20))
            .default_headers({
                let mut h = reqwest::header::HeaderMap::new();
                h.insert("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8".parse().unwrap());
                h.insert("Accept-Language", "uk-UA,uk;q=0.9,en-US;q=0.8,en;q=0.7".parse().unwrap());
                h
            })
            .build()
            .map_err(|e| format!("failed to build HTTP client: {e}"))?;
        Ok(Self { client })
    }

    pub async fn scrape(&self, config: &ScraperConfig) -> Result<Vec<NormalizationResult>, String> {
        let fetched_at = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
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

            let page_results = parse_page(&html, &fetched_at);
            let count = page_results.len();

            if count == 0 {
                // Dump a snippet so the developer can diagnose selector mismatches
                // or bot-detection pages (Cloudflare, login walls, etc.)
                let snippet = html.get(..1500).unwrap_or(&html);
                warn!(
                    page,
                    source = SOURCE,
                    html_snippet = %snippet,
                    "no jobs found on page — check selectors or bot-detection"
                );
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

/// Compiled selector set for Djinni job cards.
/// Djinni uses BEM-style class names. We try the most likely current names first,
/// with a link-href fallback that is class-name-agnostic.
struct DjinniSelectors {
    // Card containers — try both known variants
    item_primary: Selector,
    item_fallback: Selector,
    // Title link — the href encodes the numeric job id
    title_primary: Selector,
    title_fallback: Selector,
    // Company
    company: Selector,
    // Short description shown in the listing
    description: Selector,
    // Salary badge
    salary: Selector,
    // Location
    location: Selector,
    // Posted date
    time: Selector,
}

impl DjinniSelectors {
    fn new() -> Self {
        Self {
            // Variant 1: original BEM names seen in older Djinni HTML
            item_primary: Selector::parse("li.list-jobs__item").expect("valid"),
            // Variant 2: article-based cards some sites migrate to
            item_fallback: Selector::parse("article.job-list-item, li.job-list-item").expect("valid"),
            // Title — try both class names that have been in use
            title_primary: Selector::parse("a.job-list-item__link, a.job-item__title-link").expect("valid"),
            // Fallback: any anchor whose href matches /jobs/<number>
            title_fallback: Selector::parse("a[href^='/jobs/']").expect("valid"),
            company: Selector::parse("a[href*='/company/'], .job-list-item__pic a, .company-name").expect("valid"),
            description: Selector::parse(".job-list-item__description, .job-list-item__summary, .text-default").expect("valid"),
            salary: Selector::parse(".public-salary-item, .job-list-item__salary").expect("valid"),
            location: Selector::parse(".location-text, .job-list-item__location").expect("valid"),
            time: Selector::parse("time[datetime]").expect("valid"),
        }
    }
}

fn build_url(keyword: Option<&str>, page: u32) -> String {
    let mut params: Vec<String> = Vec::new();

    if let Some(kw) = keyword {
        let kw = kw.trim();
        if !kw.is_empty() {
            params.push(format!("primary_keyword={kw}"));
        }
    }

    if page > 1 {
        params.push(format!("page={page}"));
    }

    if params.is_empty() {
        format!("{BASE_URL}/jobs/")
    } else {
        format!("{BASE_URL}/jobs/?{}", params.join("&"))
    }
}

fn parse_page(html: &str, fetched_at: &str) -> Vec<NormalizationResult> {
    let document = Html::parse_document(html);
    let sel = DjinniSelectors::new();
    let mut results = Vec::new();

    // Determine which item selector matched
    let items: Vec<ElementRef> = {
        let primary: Vec<_> = document.select(&sel.item_primary).collect();
        if !primary.is_empty() {
            primary
        } else {
            document.select(&sel.item_fallback).collect()
        }
    };

    for item in items {
        match parse_item(&item, fetched_at, &sel) {
            Some(result) => results.push(result),
            None => {
                // Only warn if the item had some content (avoid logging empty containers)
                let text = collect_text(&item);
                if text.len() > 20 {
                    warn!(source = SOURCE, "skipped job item: missing required fields");
                }
            }
        }
    }

    // Link-based fallback: if structured selectors found nothing, harvest job links directly
    if results.is_empty() {
        for link in document.select(&sel.title_fallback) {
            let href = match link.value().attr("href") {
                Some(h) => h,
                None => continue,
            };
            // Only /jobs/<number>-<slug>/ — skip /jobs/ index or other pages
            if let Some(result) = build_from_link(href, fetched_at, &link, &document, &sel) {
                results.push(result);
            }
        }
    }

    results
}

fn parse_item(item: &ElementRef, fetched_at: &str, sel: &DjinniSelectors) -> Option<NormalizationResult> {
    // Title: try primary, then fallback within this item
    let title_el = item
        .select(&sel.title_primary)
        .next()
        .or_else(|| item.select(&sel.title_fallback).next())?;

    let href = title_el.value().attr("href")?;
    let title = collect_text(&title_el);
    if title.is_empty() {
        return None;
    }

    let source_job_id = extract_job_id(href)?;
    let source_url = if href.starts_with("http") {
        href.to_string()
    } else {
        format!("{BASE_URL}{href}")
    };

    let company_name = item
        .select(&sel.company)
        .next()
        .map(|el| collect_text(&el))
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "Unknown".to_string());

    let description_text = item
        .select(&sel.description)
        .next()
        .map(|el| collect_text(&el))
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| title.clone());

    let location = item
        .select(&sel.location)
        .next()
        .map(|el| collect_text(&el))
        .filter(|s| !s.is_empty());

    let salary_text = item.select(&sel.salary).next().map(|el| collect_text(&el));
    let (salary_min, salary_max, salary_currency) =
        salary_text.as_deref().map(parse_salary_range).unwrap_or((None, None, None));

    let posted_at = item
        .select(&sel.time)
        .next()
        .and_then(|el| el.value().attr("datetime"))
        .map(|dt| dt.get(..10).unwrap_or(dt).to_string())
        .map(|date| format!("{date}T00:00:00Z"));

    let full_text = collect_text(item);
    let remote_type = infer_remote_type(&full_text);
    let seniority = infer_seniority(&title);

    build_result(
        source_job_id, source_url, title, company_name,
        description_text, location, salary_min, salary_max, salary_currency,
        remote_type, seniority, posted_at, fetched_at,
    )
}

/// Last-resort: build a result from just a job link, scraping parent context.
fn build_from_link(
    href: &str,
    fetched_at: &str,
    link: &ElementRef,
    _document: &Html,
    _sel: &DjinniSelectors,
) -> Option<NormalizationResult> {
    let source_job_id = extract_job_id(href)?;
    let title = collect_text(link);
    if title.is_empty() {
        return None;
    }
    let source_url = format!("{BASE_URL}{href}");

    build_result(
        source_job_id, source_url, title, "Unknown".to_string(),
        String::new(), None, None, None, None, None, None, None, fetched_at,
    )
}

#[allow(clippy::too_many_arguments)]
fn build_result(
    source_job_id: String,
    source_url: String,
    title: String,
    company_name: String,
    description_text: String,
    location: Option<String>,
    salary_min: Option<i32>,
    salary_max: Option<i32>,
    salary_currency: Option<String>,
    remote_type: Option<String>,
    seniority: Option<String>,
    posted_at: Option<String>,
    fetched_at: &str,
) -> Option<NormalizationResult> {
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

/// Extract numeric job ID from a Djinni job slug.
/// "/jobs/12345-senior-rust-engineer/" → "12345"
fn extract_job_id(href: &str) -> Option<String> {
    let slug = href.trim_matches('/').split('/').last()?;
    let id_part = slug.split('-').next()?;
    if !id_part.is_empty() && id_part.chars().all(|c| c.is_ascii_digit()) {
        Some(id_part.to_string())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::{build_url, extract_job_id};

    #[test]
    fn builds_url_without_params() {
        assert_eq!(build_url(None, 1), "https://djinni.co/jobs/");
    }

    #[test]
    fn builds_url_with_keyword_and_page() {
        assert_eq!(
            build_url(Some("Python"), 2),
            "https://djinni.co/jobs/?primary_keyword=Python&page=2"
        );
    }

    #[test]
    fn extracts_job_id_from_slug() {
        assert_eq!(
            extract_job_id("/jobs/12345-senior-rust-engineer/"),
            Some("12345".to_string())
        );
    }

    #[test]
    fn rejects_non_numeric_slug() {
        assert_eq!(extract_job_id("/jobs/senior-engineer/"), None);
    }

    #[test]
    fn link_fallback_parses_basic_card() {
        use crate::scrapers::djinni::parse_page;

        // Minimal HTML with just a job link — tests the link-based fallback
        let html = r#"<html><body>
          <a href="/jobs/99999-rust-developer/">Senior Rust Developer</a>
        </body></html>"#;

        let results = parse_page(html, "2026-04-14T00:00:00Z");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].job.title, "Senior Rust Developer");
        assert_eq!(results[0].snapshot.source_job_id, "99999");
    }
}
