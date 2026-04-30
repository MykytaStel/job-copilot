use std::time::Duration;

use chrono::Utc;
use reqwest::Client;
use scraper::{ElementRef, Html, Selector};
use serde_json::json;
use tracing::{info, warn};

use crate::models::{NormalizationResult, NormalizedJob, RawSnapshot};
use crate::scrapers::{
    DetailSnapshot, ScraperConfig, ScraperRun, cleanup_description_text, collect_text,
    detail_error_summaries, extract_skills, fetch_with_backoff, infer_company_meta,
    infer_remote_type, infer_seniority_from_title_and_description, merge_detail_into_result,
    normalize_company_name, normalized_non_empty, parse_salary_range_with_usd_monthly,
    polite_delay,
};

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

    pub async fn scrape(&self, config: &ScraperConfig) -> Result<ScraperRun, String> {
        let fetched_at = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
        let selectors = WorkUaSelectors::new();
        let mut results: Vec<NormalizationResult> = Vec::new();
        let mut jobs_attempted = 0u32;
        let mut jobs_failed = 0u32;

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
            let attempted = page_results.len() as u32;
            let page_results = self.enrich_page_results(page_results, &fetched_at).await;
            let count = page_results.len();
            jobs_attempted += attempted;
            jobs_failed += attempted.saturating_sub(count as u32);

            if count == 0 {
                info!(page, source = SOURCE, "no jobs on page, stopping");
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

    async fn fetch(&self, url: &str) -> Result<String, String> {
        fetch_with_backoff(&self.client, url).await
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

        let Some(detail_html) = self.fetch(&source_url).await.ok() else {
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

fn parse_item(
    item: &ElementRef,
    fetched_at: &str,
    sel: &WorkUaSelectors,
) -> Option<NormalizationResult> {
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
        .and_then(|value| normalize_company_name(&value));

    // Location is often in the same `.add-top-xs` line as the company: "CompanyName · Kyiv"
    // Strip the company name from it to get just the location part.
    let location = item
        .select(&sel.meta)
        .next()
        .map(|el| {
            let full = collect_text(&el);
            // Remove the company portion and clean separators
            full.replace(company_name.as_deref().unwrap_or(""), "")
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

    let salary_text = item.select(&sel.salary).next().map(|el| collect_text(&el));
    let (salary_min, salary_max, salary_currency, salary_usd_min, salary_usd_max) = salary_text
        .as_deref()
        .map(parse_salary_range_with_usd_monthly)
        .unwrap_or((None, None, None, None, None));

    let full_text = collect_text(item);
    let remote_type = infer_remote_type(&full_text);
    let seniority = infer_seniority_from_title_and_description(&title, Some(&description_text));
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
        "fetched_at": fetched_at,
    });

    Some(NormalizationResult {
        job: NormalizedJob {
            // id is overridden by IngestionBatch::from_normalization_results
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
            "a[href*='/employer/']",
            ".breadcrumbs a",
        ],
    )
    .and_then(|value| normalize_company_name(&value));
    let company_url = extract_first_href(
        &document,
        &["a[href*='/employer/']", ".breadcrumbs a[href]"],
    )
    .and_then(|value| normalize_absolute_url(&value));
    let description_text = extract_rich_text(
        &document,
        &[
            "div#job-description",
            "[data-qa='vacancy-description']",
            "section.wordwrap",
            "div.wordwrap",
            ".overflow.wordwrap",
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
                "Схожі вакансії",
                "Вакансії за категоріями",
                "Відгукнутися",
                "Відгукнутись",
            ],
        )
    })
    .filter(|value| !value.is_empty());
    let location = extract_first_text(
        &document,
        &[
            ".text-muted",
            ".breadcrumbs li:last-child",
            "[data-qa='vacancy-city']",
        ],
    )
    .and_then(|value| normalized_non_empty(Some(&value)));

    let salary_source = extract_first_text(
        &document,
        &[
            ".salary",
            ".text-default strong",
            "span[class*='salary']",
            "main",
        ],
    );
    let (salary_min, salary_max, salary_currency, salary_usd_min, salary_usd_max) = salary_source
        .as_deref()
        .map(parse_salary_range_with_usd_monthly)
        .unwrap_or((None, None, None, None, None));

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
        posted_at: None,
        raw_payload: json!({
            "detail_title": extract_first_text(&document, &["h1"]),
            "detail_company_name": extract_first_text(
                &document,
                &["a[href*='/company/']", "a[href*='/employer/']", ".breadcrumbs a"]
            ),
            "detail_company_url": company_url,
            "detail_location": extract_first_text(
                &document,
                &[".text-muted", ".breadcrumbs li:last-child", "[data-qa='vacancy-city']"]
            ),
            "detail_description_text": extract_rich_text(
                &document,
                &[
                    "div#job-description",
                    "[data-qa='vacancy-description']",
                    "section.wordwrap",
                    "div.wordwrap",
                    ".overflow.wordwrap",
                    "article",
                    "main",
                ]
            ),
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
                .filter(|value| value.len() > 10)
                .filter(|value| {
                    !matches!(
                        value.as_str(),
                        "Схожі вакансії"
                            | "Вакансії за категоріями"
                            | "Відгукнутися"
                            | "Відгукнутись"
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
    use super::{build_url, extract_job_id, parse_detail_page};
    use crate::models::{NormalizationResult, NormalizedJob, RawSnapshot};
    use serde_json::json;

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
        assert_eq!(build_url(None, 2), "https://www.work.ua/jobs-it/?page=2");
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

    #[test]
    fn parses_work_ua_detail_fixture() {
        let html = include_str!("../../tests/fixtures/work_ua_detail.html");
        let fallback = NormalizationResult {
            job: NormalizedJob {
                id: "job_work_ua_123".to_string(),
                duplicate_of: None,
                title: "Senior Rust Engineer".to_string(),
                company_name: "SignalHire".to_string(),
                company_meta: None,
                location: Some("Kyiv".to_string()),
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
                source: "work_ua".to_string(),
                source_job_id: "123".to_string(),
                source_url: "https://www.work.ua/jobs/123/".to_string(),
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
                .is_some_and(|value| value.contains("Build reliable Rust APIs"))
        );
    }

    #[test]
    fn keeps_rich_detail_description_over_short_listing_snippet() {
        let html = include_str!("../../tests/fixtures/work_ua_detail_regression.html");
        let fallback = NormalizationResult {
            job: NormalizedJob {
                id: "job_work_ua_456".to_string(),
                duplicate_of: None,
                title: "Front-end React Developer".to_string(),
                company_name: "SignalHire".to_string(),
                company_meta: None,
                location: Some("Kyiv".to_string()),
                remote_type: Some("remote".to_string()),
                seniority: Some("senior".to_string()),
                description_text: "React team".to_string(),
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
                source: "work_ua".to_string(),
                source_job_id: "456".to_string(),
                source_url: "https://www.work.ua/jobs/456/".to_string(),
                raw_payload: json!({}),
                fetched_at: "2026-04-18T10:00:00Z".to_string(),
            },
        };

        let detail = parse_detail_page(html, &fallback, "2026-04-18T10:00:00Z");
        let description = detail
            .description_text
            .expect("detail description should be present");

        assert!(description.contains("Ship accessible frontend features"));
        assert!(description.contains("Partner with product and design"));
        assert!(!description.contains("Схожі вакансії"));
    }
}
