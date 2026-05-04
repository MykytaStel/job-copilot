use std::time::Duration;

use chrono::Utc;
use reqwest::Client;
use scraper::{ElementRef, Html, Selector};
use serde_json::{Value, json};
use tracing::{info, warn};

use crate::error::Result;
use crate::models::{NormalizationResult, NormalizedJob, RawSnapshot};
use crate::scrapers::{
    DetailSnapshot, ScraperConfig, ScraperRun, cleanup_description_text, collect_text,
    detail_error_summaries, extract_skills, headers::build_default_headers, infer_company_meta,
    infer_remote_type, infer_seniority, infer_seniority_from_title_and_description,
    normalize_company_name, normalized_non_empty, parse_salary_range_with_usd_monthly, polite_delay,
};
use crate::scrapers::runner::JobSource;

const SOURCE: &str = <DjinniScraper as JobSource>::SOURCE;
const BASE_URL: &str = "https://djinni.co";

pub struct DjinniScraper {
    client: Client,
}

impl DjinniScraper {
    pub fn new() -> Result<Self> {
        let headers = build_default_headers(
            "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8",
            "uk-UA,uk;q=0.9,en-US;q=0.8,en;q=0.7",
            &[],
        )?;
        let client = Client::builder()
            .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36")
            .timeout(Duration::from_secs(20))
            .default_headers(headers)
            .build()?;
        Ok(Self { client })
    }

    pub async fn scrape(&self, config: &ScraperConfig) -> Result<ScraperRun> {
        let fetched_at = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
        let mut results: Vec<NormalizationResult> = Vec::new();
        let mut jobs_attempted = 0u32;
        let mut jobs_failed = 0u32;

        for page in 1..=config.pages {
            let url = build_url(config.keyword.as_deref(), page);
            info!(%url, page, source = SOURCE, "fetching page");

            let html = match self.fetch_url(&url).await {
                Ok(html) => html,
                Err(e) => {
                    warn!(error = %e, %url, "fetch failed, stopping pagination");
                    break;
                }
            };

            let page_results = parse_page(&html, &fetched_at);
            let attempted = page_results.len() as u32;
            let page_results = self.enrich_results(page_results, &fetched_at).await;
            let count = page_results.len();
            jobs_attempted += attempted;
            jobs_failed += attempted.saturating_sub(count as u32);

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
            info!(
                page,
                jobs = count,
                total = results.len(),
                source = SOURCE,
                "parsed page"
            );

            if page < config.pages {
                polite_delay(config.page_delay_ms).await;
            }
        }

        info!(total = results.len(), source = SOURCE, "scrape complete");
        Ok(ScraperRun {
            jobs: results,
            jobs_attempted,
            jobs_failed,
            errors: detail_error_summaries(SOURCE, jobs_failed),
        })
    }
}

impl JobSource for DjinniScraper {
    const SOURCE: &'static str = "djinni";

    fn client(&self) -> &Client {
        &self.client
    }

    fn parse_detail(
        &self,
        html: &str,
        context: &NormalizationResult,
        fetched_at: &str,
    ) -> DetailSnapshot {
        parse_detail_page(html, context, fetched_at)
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
            item_fallback: Selector::parse(
                "article.job-list-item, li.job-list-item, article[data-job-id]",
            )
            .expect("valid"),
            // Title — try both class names that have been in use
            title_primary: Selector::parse(
                "a.job-list-item__link, a.job-item__title-link, a[data-qa='job-title']",
            )
            .expect("valid"),
            // Fallback: any anchor whose href matches /jobs/<number>
            title_fallback: Selector::parse("a[href^='/jobs/']").expect("valid"),
            company: Selector::parse("a[href*='/company/'], .job-list-item__pic a, .company-name")
                .expect("valid"),
            description: Selector::parse(
                ".job-list-item__description, .job-list-item__summary, [data-qa='job-item-description'], .text-default",
            )
            .expect("valid"),
            salary: Selector::parse(".public-salary-item, .job-list-item__salary").expect("valid"),
            location: Selector::parse(
                ".location-text, .job-list-item__location, [data-qa='job-location']",
            )
            .expect("valid"),
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

    if results.is_empty() {
        results.extend(parse_json_ld_jobs(&document, fetched_at));
    }

    results
}
fn parse_json_ld_jobs(document: &Html, fetched_at: &str) -> Vec<NormalizationResult> {
    let selector =
        Selector::parse(r#"script[type="application/ld+json"]"#).expect("valid selector");

    let mut results = Vec::new();

    for script in document.select(&selector) {
        let raw_json = script.inner_html();

        let Ok(value) = serde_json::from_str::<Value>(&raw_json) else {
            continue;
        };

        let mut postings = Vec::new();
        collect_job_postings(&value, &mut postings);

        for posting in postings {
            if let Some(result) = build_from_json_ld_job(posting, fetched_at) {
                results.push(result);
            }
        }
    }

    results
}

fn collect_job_postings<'a>(value: &'a Value, out: &mut Vec<&'a Value>) {
    match value {
        Value::Array(items) => {
            for item in items {
                collect_job_postings(item, out);
            }
        }
        Value::Object(map) => {
            if is_job_posting(value) {
                out.push(value);
            }

            if let Some(graph) = map.get("@graph") {
                collect_job_postings(graph, out);
            }
        }
        _ => {}
    }
}

fn is_job_posting(value: &Value) -> bool {
    let Some(kind) = value.get("@type") else {
        return false;
    };

    match kind {
        Value::String(value) => value == "JobPosting",
        Value::Array(values) => values
            .iter()
            .any(|value| value.as_str().is_some_and(|value| value == "JobPosting")),
        _ => false,
    }
}

fn build_from_json_ld_job(value: &Value, fetched_at: &str) -> Option<NormalizationResult> {
    let title = json_string(value, &["title"])?;
    let source_url = normalize_source_url(
        json_string(value, &["url"])
            .or_else(|| json_string(value, &["mainEntityOfPage"]))
            .as_deref(),
    )?;

    let source_job_id = extract_job_id(&source_url)
        .or_else(|| json_string(value, &["identifier", "value"]))
        .filter(|value| !value.trim().is_empty())?;

    let company_name = json_string(value, &["hiringOrganization", "name"])
        .and_then(|value| normalize_company_name(&value))?;

    let description_text = json_string(value, &["description"])
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| title.clone());

    let location = extract_json_ld_location(value);

    let posted_at = json_string(value, &["datePosted"])
        .map(|value| value.get(..10).unwrap_or(&value).to_string())
        .map(|date| format!("{date}T00:00:00Z"));

    let signal_text = [
        title.clone(),
        company_name.clone(),
        location.clone().unwrap_or_default(),
        description_text.clone(),
    ]
    .join(" ");

    let remote_type = infer_remote_type(&signal_text);
    let seniority = infer_seniority_from_title_and_description(&title, Some(&description_text));

    build_result(
        source_job_id,
        source_url,
        title,
        Some(company_name),
        description_text,
        location,
        None,
        None,
        None,
        None,
        None,
        remote_type,
        seniority,
        posted_at,
        fetched_at,
    )
}

fn json_string(value: &Value, path: &[&str]) -> Option<String> {
    let mut current = value;

    for key in path {
        current = current.get(*key)?;
    }

    current
        .as_str()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

fn normalize_source_url(value: Option<&str>) -> Option<String> {
    let value = value?.trim();

    if value.is_empty() {
        return None;
    }

    if value.starts_with("http://") || value.starts_with("https://") {
        Some(value.to_string())
    } else if value.starts_with('/') {
        Some(format!("{BASE_URL}{value}"))
    } else {
        Some(format!("{BASE_URL}/{value}"))
    }
}

fn extract_json_ld_location(value: &Value) -> Option<String> {
    json_string(value, &["jobLocation", "address", "addressLocality"])
        .or_else(|| json_string(value, &["jobLocation", "address", "addressRegion"]))
        .or_else(|| json_string(value, &["jobLocation", "address", "addressCountry"]))
        .or_else(|| {
            json_string(
                value,
                &["applicantLocationRequirements", "address", "addressCountry"],
            )
        })
}

fn parse_item(
    item: &ElementRef,
    fetched_at: &str,
    sel: &DjinniSelectors,
) -> Option<NormalizationResult> {
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
        .and_then(|value| normalize_company_name(&value));

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
    let (salary_min, salary_max, salary_currency, salary_usd_min, salary_usd_max) = salary_text
        .as_deref()
        .map(parse_salary_range_with_usd_monthly)
        .unwrap_or((None, None, None, None, None));

    let posted_at = item
        .select(&sel.time)
        .next()
        .and_then(|el| el.value().attr("datetime"))
        .map(|dt| dt.get(..10).unwrap_or(dt).to_string())
        .map(|date| format!("{date}T00:00:00Z"));

    let full_text = collect_text(item);
    let remote_type = infer_remote_type(&full_text);
    let seniority = infer_seniority_from_title_and_description(&title, Some(&description_text));

    build_result(
        source_job_id,
        source_url,
        title,
        company_name,
        description_text,
        location,
        salary_min,
        salary_max,
        salary_currency,
        salary_usd_min,
        salary_usd_max,
        remote_type,
        seniority,
        posted_at,
        fetched_at,
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
    let seniority = infer_seniority(&title);

    build_result(
        source_job_id,
        source_url,
        title,
        None,
        String::new(),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        seniority,
        None,
        fetched_at,
    )
}

#[allow(clippy::too_many_arguments)]
fn build_result(
    source_job_id: String,
    source_url: String,
    title: String,
    company_name: Option<String>,
    description_text: String,
    location: Option<String>,
    salary_min: Option<i32>,
    salary_max: Option<i32>,
    salary_currency: Option<String>,
    salary_usd_min: Option<i32>,
    salary_usd_max: Option<i32>,
    remote_type: Option<String>,
    seniority: Option<String>,
    posted_at: Option<String>,
    fetched_at: &str,
) -> Option<NormalizationResult> {
    let extracted_skills = extract_skills(&description_text);
    let company_meta = infer_company_meta(&description_text, None);

    let raw_payload = json!({
        "source_job_id": source_job_id,
        "source_url": source_url,
        "title": title,
        "company_name": company_name,
        "company_meta": company_meta,
        "location": location,
        "remote_type": remote_type,
        "seniority": seniority,
        "description_text": description_text,
        "extracted_skills": extracted_skills.clone(),
        "salary_min": salary_min,
        "salary_max": salary_max,
        "salary_currency": salary_currency,
        "salary_usd_min": salary_usd_min,
        "salary_usd_max": salary_usd_max,
        "posted_at": posted_at,
        "fetched_at": fetched_at,
    });

    Some(NormalizationResult {
        job: NormalizedJob {
            id: format!("job_{SOURCE}_{source_job_id}"),
            duplicate_of: None,
            title,
            company_name: company_name?,
            company_meta,
            location,
            remote_type,
            seniority,
            description_text,
            extracted_skills,
            salary_min,
            salary_max,
            salary_currency,
            salary_usd_min,
            salary_usd_max,
            quality_score: None,
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
    let company_name = extract_first_text(
        &document,
        &[
            "a[href*='/company/']",
            ".job-details--title a",
            ".profile-page__title a",
        ],
    )
    .and_then(|value| normalize_company_name(&value));
    let company_url = extract_first_href(
        &document,
        &[
            "a[href*='/company/']",
            ".job-details--title a",
            ".profile-page__title a",
        ],
    )
    .and_then(|value| normalize_absolute_url(&value));
    let description_text = extract_rich_text(
        &document,
        &[
            "#job-description",
            "[data-testid='job-description']",
            "[data-qa='job-description']",
            ".job-post__description",
            ".job-post__content",
            ".profile-page-section .mb-4",
            ".profile-page-section",
            "article",
            "main",
        ],
    )
    .map(|value| {
        cleanup_description_text(
            &value,
            title.as_deref().unwrap_or(&fallback.job.title),
            company_name
                .as_deref()
                .unwrap_or(&fallback.job.company_name),
            &[
                "How to apply",
                "Send CV",
                "View vacancy",
                "Jobs feed in RSS",
            ],
        )
    })
    .filter(|value| !value.is_empty());
    let location = extract_first_text(
        &document,
        &[".location-text", ".job-details--location", "header"],
    )
    .and_then(|value| normalized_non_empty(Some(&value)));
    let salary_source = extract_first_text(
        &document,
        &[".public-salary-item", ".job-details--salary", "main"],
    );
    let (salary_min, salary_max, salary_currency, salary_usd_min, salary_usd_max) = salary_source
        .as_deref()
        .map(parse_salary_range_with_usd_monthly)
        .unwrap_or((None, None, None, None, None));
    let posted_at = extract_datetime(&document);

    let signal_text = [
        title.clone().unwrap_or_default(),
        company_name.clone().unwrap_or_default(),
        location.clone().unwrap_or_default(),
        description_text.clone().unwrap_or_default(),
    ]
    .join(" ");
    let seniority = infer_seniority_from_title_and_description(
        title.as_deref().unwrap_or(&fallback.job.title),
        description_text.as_deref(),
    );

    DetailSnapshot {
        title,
        company_name,
        company_url: company_url.clone(),
        location,
        remote_type: infer_remote_type(&signal_text),
        seniority,
        description_text,
        salary_min,
        salary_max,
        salary_currency,
        salary_usd_min,
        salary_usd_max,
        posted_at,
        raw_payload: json!({
            "detail_title": extract_first_text(&document, &["h1"]),
            "detail_company_name": extract_first_text(
                &document,
                &["a[href*='/company/']", ".job-details--title a", ".profile-page__title a"]
            ),
            "detail_company_url": company_url,
            "detail_location": extract_first_text(
                &document,
                &[".location-text", ".job-details--location", "header"]
            ),
            "detail_description_text": extract_rich_text(
                &document,
                &[
                    "#job-description",
                    "[data-testid='job-description']",
                    "[data-qa='job-description']",
                    ".job-post__description",
                    ".job-post__content",
                    ".profile-page-section .mb-4",
                    ".profile-page-section",
                    "article",
                    "main",
                ]
            ),
            "detail_salary_text": salary_source,
            "detail_posted_at": extract_datetime(&document),
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
            .map(|element| collect_text(&element))
            .find(|value| !value.trim().is_empty())
        {
            return Some(value);
        }
    }

    None
}

fn extract_first_href(document: &Html, selectors: &[&str]) -> Option<String> {
    for raw_selector in selectors {
        let Ok(selector) = Selector::parse(raw_selector) else {
            continue;
        };
        if let Some(value) = document
            .select(&selector)
            .filter_map(|element| element.value().attr("href"))
            .map(str::trim)
            .find(|value| !value.is_empty())
        {
            return Some(value.to_string());
        }
    }

    None
}

fn normalize_absolute_url(value: &str) -> Option<String> {
    let value = value.trim();
    if value.is_empty() {
        None
    } else if value.starts_with("http://") || value.starts_with("https://") {
        Some(value.to_string())
    } else if value.starts_with('/') {
        Some(format!("{BASE_URL}{value}"))
    } else {
        Some(format!("{BASE_URL}/{value}"))
    }
}

fn extract_rich_text(document: &Html, selectors: &[&str]) -> Option<String> {
    let list_selector = Selector::parse("p, li, h2, h3, h4, ul, ol").expect("valid selector");
    let mut best: Option<String> = None;

    for raw_selector in selectors {
        let Ok(selector) = Selector::parse(raw_selector) else {
            continue;
        };

        for container in document.select(&selector) {
            let parts = container
                .select(&list_selector)
                .map(|element| collect_text(&element))
                .filter(|value| value.len() > 12)
                .filter(|value| {
                    !matches!(
                        value.as_str(),
                        "How to apply" | "Send CV" | "View vacancy" | "Jobs feed in RSS"
                    )
                })
                .collect::<Vec<_>>();
            let candidate = if parts.is_empty() {
                collect_text(&container)
            } else {
                parts.join("\n")
            };
            if candidate.len() > best.as_ref().map_or(0, String::len) {
                best = Some(candidate);
            }
        }
    }

    best
}

fn extract_datetime(document: &Html) -> Option<String> {
    let selector = Selector::parse("time[datetime]").expect("valid selector");
    document
        .select(&selector)
        .filter_map(|element| element.value().attr("datetime"))
        .map(|value| value.get(..10).unwrap_or(value).to_string())
        .next()
        .map(|date| format!("{date}T00:00:00Z"))
}

/// Extract numeric job ID from a Djinni job slug.
/// "/jobs/12345-senior-rust-engineer/" → "12345"
fn extract_job_id(href: &str) -> Option<String> {
    let slug = href.trim_matches('/').split('/').next_back()?;
    let id_part = slug.split('-').next()?;
    if !id_part.is_empty() && id_part.chars().all(|c| c.is_ascii_digit()) {
        Some(id_part.to_string())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::{build_url, extract_job_id, parse_detail_page};
    use crate::models::{NormalizationResult, NormalizedJob, RawSnapshot};
    use serde_json::json;

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

        // Link-only fallbacks are no longer accepted because they cannot
        // produce a canonical company_name without the detail fetch succeeding.
        let html = r#"<html><body>
          <a href="/jobs/99999-rust-developer/">Senior Rust Developer</a>
        </body></html>"#;

        let results = parse_page(html, "2026-04-14T00:00:00Z");
        assert!(results.is_empty());
    }

    #[test]
    fn parses_djinni_detail_fixture() {
        let html = include_str!("../../tests/fixtures/djinni_detail.html");
        let fallback = NormalizationResult {
            job: NormalizedJob {
                id: "job_djinni_123".to_string(),
                duplicate_of: None,
                title: "Senior Rust Engineer".to_string(),
                company_name: "SignalHire".to_string(),
                company_meta: None,
                location: Some("Remote".to_string()),
                remote_type: Some("remote".to_string()),
                seniority: Some("senior".to_string()),
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
                source_job_id: "123".to_string(),
                source_url: "https://djinni.co/jobs/123-senior-rust-engineer/".to_string(),
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
                .is_some_and(|value| value.contains("Own backend services"))
        );
        assert_eq!(detail.posted_at.as_deref(), Some("2026-04-16T00:00:00Z"));
    }

    #[test]
    fn prefers_detail_body_when_listing_snippet_is_short() {
        let html = include_str!("../../tests/fixtures/djinni_detail_regression.html");
        let fallback = NormalizationResult {
            job: NormalizedJob {
                id: "job_djinni_321".to_string(),
                duplicate_of: None,
                title: "Front-end React Developer".to_string(),
                company_name: "SignalHire".to_string(),
                company_meta: None,
                location: Some("Remote, Europe".to_string()),
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
                source_job_id: "321".to_string(),
                source_url: "https://djinni.co/jobs/321-front-end-react-developer/".to_string(),
                raw_payload: json!({}),
                fetched_at: "2026-04-18T10:00:00Z".to_string(),
            },
        };

        let detail = parse_detail_page(html, &fallback, "2026-04-18T10:00:00Z");
        let description = detail
            .description_text
            .expect("detail description should be present");

        assert!(description.contains("Own the frontend architecture"));
        assert!(description.contains("Improve performance budgets"));
        assert!(!description.contains("How to apply"));
    }
    #[test]
    fn parses_djinni_json_ld_listing_fallback() {
        use crate::scrapers::djinni::parse_page;

        let html = r#"
        <!doctype html>
        <html>
          <head>
            <script type="application/ld+json">
              [{
                "@context": "https://schema.org/",
                "@type": "JobPosting",
                "title": "Senior Software Engineer",
                "description": "React, TypeScript, Node.js product role",
                "datePosted": "2026-04-25T23:12:41.156144",
                "url": "https://djinni.co/jobs/12345-senior-software-engineer/",
                "hiringOrganization": {
                  "@type": "Organization",
                  "name": "Test Company"
                },
                "jobLocation": {
                  "@type": "Place",
                  "address": {
                    "@type": "PostalAddress",
                    "addressLocality": "Kyiv",
                    "addressCountry": "UA"
                  }
                }
              }]
            </script>
          </head>
          <body></body>
        </html>
    "#;

        let results = parse_page(html, "2026-04-25T00:00:00Z");

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].job.id, "job_djinni_12345");
        assert_eq!(results[0].job.title, "Senior Software Engineer");
        assert_eq!(results[0].job.company_name, "Test Company");
        assert_eq!(results[0].job.location.as_deref(), Some("Kyiv"));
        assert_eq!(
            results[0].job.posted_at.as_deref(),
            Some("2026-04-25T00:00:00Z")
        );
    }
}
