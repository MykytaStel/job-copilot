use std::time::Duration;

use chrono::{DateTime, Utc};
use reqwest::Client;
use scraper::{Html, Selector};
use serde_json::json;
use tracing::{info, warn};

use crate::models::{NormalizationResult, NormalizedJob, RawSnapshot};
use crate::scrapers::{
    DetailSnapshot, ScraperConfig, cleanup_description_text, collect_text, infer_remote_type,
    infer_seniority, merge_detail_into_result, normalize_company_name, normalized_non_empty,
    parse_salary_range, polite_delay,
};

const SOURCE: &str = "dou_ua";
const FEED_URL: &str = "https://jobs.dou.ua/vacancies/feeds/";
const PAGE_SIZE: usize = 20;

pub struct DouUaScraper {
    client: Client,
}

impl DouUaScraper {
    pub fn new() -> Result<Self, String> {
        let client = Client::builder()
            .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36")
            .timeout(Duration::from_secs(20))
            .default_headers({
                let mut h = reqwest::header::HeaderMap::new();
                h.insert(
                    "Accept",
                    "application/rss+xml,application/xml,text/xml;q=0.9,text/html;q=0.8,*/*;q=0.7"
                        .parse()
                        .unwrap(),
                );
                h.insert("Accept-Language", "uk-UA,uk;q=0.9,en-US;q=0.8,en;q=0.7".parse().unwrap());
                h
            })
            .build()
            .map_err(|e| format!("failed to build HTTP client: {e}"))?;
        Ok(Self { client })
    }

    pub async fn scrape(&self, config: &ScraperConfig) -> Result<Vec<NormalizationResult>, String> {
        let fetched_at = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
        info!(source = SOURCE, url = FEED_URL, "fetching RSS feed");

        let xml = self.fetch(FEED_URL).await?;
        let items = parse_feed(&xml);
        let filtered = items
            .into_iter()
            .filter(|item| matches_keyword(item, config.keyword.as_deref()))
            .take((config.pages.max(1) as usize) * PAGE_SIZE)
            .collect::<Vec<_>>();

        if filtered.is_empty() {
            warn!(source = SOURCE, "RSS feed returned no matching jobs");
            return Ok(Vec::new());
        }

        let mut results = Vec::new();
        for item in filtered {
            if let Some(result) = normalize_feed_item(item, &fetched_at) {
                results.push(result);
            }
        }

        let results = self.enrich_page_results(results, &fetched_at).await;
        info!(source = SOURCE, jobs = results.len(), "parsed RSS feed");
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

#[derive(Debug, Clone, PartialEq, Eq)]
struct FeedItem {
    title: String,
    link: String,
    description: String,
    pub_date: Option<String>,
}

fn parse_feed(xml: &str) -> Vec<FeedItem> {
    extract_blocks(xml, "item")
        .into_iter()
        .filter_map(|block| {
            let title = extract_tag_text(&block, "title")?;
            let link = extract_tag_text(&block, "link")?;
            let description = extract_tag_text(&block, "description").unwrap_or_default();
            let pub_date = extract_tag_text(&block, "pubDate");

            Some(FeedItem {
                title: decode_html_entities(&title),
                link: canonicalize_source_url(&link),
                description: description.trim().to_string(),
                pub_date,
            })
        })
        .collect()
}

fn normalize_feed_item(item: FeedItem, fetched_at: &str) -> Option<NormalizationResult> {
    let source_job_id = extract_job_id(&item.link)?;
    let (title, company_name, meta) = parse_feed_title(&item.title)?;
    let company_name = normalize_company_name(&company_name)?;
    let description_text = html_fragment_to_text(&item.description)
        .map(|value| {
            cleanup_description_text(
                &value,
                &title,
                &company_name,
                &["Відгукнутись", "Відгукнутися", "Правила відгуків"],
            )
        })
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| title.clone());
    let location = extract_location_from_meta(&meta);
    let salary_source = normalized_non_empty(Some(&meta));
    let (salary_min, salary_max, salary_currency) = salary_source
        .as_deref()
        .map(parse_salary_range)
        .unwrap_or((None, None, None));
    let posted_at = item.pub_date.as_deref().and_then(parse_pub_date);

    let signal_text = [
        title.clone(),
        company_name.clone(),
        meta.clone(),
        description_text.clone(),
    ]
    .join(" ");

    let remote_type = infer_remote_type(&signal_text);
    let seniority = infer_seniority(&signal_text);

    let raw_payload = json!({
        "rss_title": item.title,
        "source_job_id": source_job_id,
        "source_url": item.link,
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
        "rss_pub_date": item.pub_date,
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
            source_job_id,
            source_url: item.link,
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
    let title = extract_first_text(&document, &["h1.g-h2", "h1"]);
    let company_name = extract_first_text(
        &document,
        &[
            ".b-compinfo .l-n a[href*='/companies/']",
            ".b-compinfo a[href*='/companies/']",
        ],
    )
    .and_then(|value| normalize_company_name(&value));
    let location = extract_first_text(
        &document,
        &[".l-vacancy .sh-info .place", ".sh-info .place"],
    )
    .and_then(|value| normalized_non_empty(Some(&value)));
    let salary_source = extract_first_text(
        &document,
        &[".l-vacancy .sh-info .salary", ".sh-info .salary"],
    );
    let (salary_min, salary_max, salary_currency) = salary_source
        .as_deref()
        .map(parse_salary_range)
        .unwrap_or((None, None, None));
    let description_text = extract_rich_text(
        &document,
        &[
            ".b-typo.vacancy-section",
            ".l-vacancy [itemprop='description']",
            ".l-vacancy[itemprop='description']",
            ".l-vacancy .b-typo",
            ".b-vacancy .b-typo",
            ".l-vacancy",
            "article",
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
                "Відгукнутися",
                "Відгукнутись",
                "Правила відгуків",
                "Схожі вакансії",
            ],
        )
    })
    .filter(|value| !value.is_empty());

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
        posted_at: fallback.job.posted_at.clone(),
        raw_payload: json!({
            "detail_title": extract_first_text(&document, &["h1.g-h2", "h1"]),
            "detail_company_name": extract_first_text(
                &document,
                &[".b-compinfo .l-n a[href*='/companies/']", ".b-compinfo a[href*='/companies/']"]
            ),
            "detail_location": extract_first_text(
                &document,
                &[".l-vacancy .sh-info .place", ".sh-info .place"]
            ),
            "detail_description_text": extract_rich_text(
                &document,
                &[
                    ".b-typo.vacancy-section",
                    ".l-vacancy [itemprop='description']",
                    ".l-vacancy[itemprop='description']",
                    ".l-vacancy .b-typo",
                    ".b-vacancy .b-typo",
                    ".l-vacancy",
                    "article",
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
                .filter(|value| value.len() > 6)
                .filter(|value| {
                    !matches!(
                        value.as_str(),
                        "Відгукнутися на вакансію"
                            | "Відгукнутися"
                            | "Відгукнутись"
                            | "Правила відгуків"
                            | "Схожі вакансії"
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

fn matches_keyword(item: &FeedItem, keyword: Option<&str>) -> bool {
    let Some(keyword) = keyword.map(str::trim).filter(|value| !value.is_empty()) else {
        return true;
    };

    let haystack = normalize_for_match(
        &[
            item.title.as_str(),
            item.description.as_str(),
            item.link.as_str(),
        ]
        .join(" "),
    );
    let needle = normalize_for_match(keyword);

    haystack.contains(&needle)
}

fn normalize_for_match(value: &str) -> String {
    decode_html_entities(value).to_lowercase()
}

fn parse_feed_title(value: &str) -> Option<(String, String, String)> {
    let cleaned = decode_html_entities(value);
    let (title, rest) = cleaned.rsplit_once(" в ")?;
    let (company_name, meta) = match rest.split_once(',') {
        Some((company, remainder)) => (company.trim(), remainder.trim()),
        None => (rest.trim(), ""),
    };

    Some((
        title.trim().to_string(),
        company_name.trim().to_string(),
        meta.trim().to_string(),
    ))
}

fn extract_location_from_meta(value: &str) -> Option<String> {
    let segments = value
        .split(',')
        .map(decode_html_entities)
        .map(|part| part.trim().to_string())
        .filter(|part| !part.is_empty())
        .filter(|part| !looks_like_salary(part))
        .collect::<Vec<_>>();

    if segments.is_empty() {
        None
    } else {
        Some(segments.join(", "))
    }
}

fn looks_like_salary(value: &str) -> bool {
    let normalized = value.to_lowercase();
    normalized.contains('$')
        || normalized.contains('€')
        || normalized.contains("usd")
        || normalized.contains("eur")
        || normalized.contains("грн")
        || normalized.contains("uah")
        || normalized.starts_with("від ")
        || normalized.starts_with("до ")
        || normalized
            .chars()
            .any(|ch| ch.is_ascii_digit() && normalized.contains('$'))
}

fn canonicalize_source_url(value: &str) -> String {
    value.trim().split('?').next().unwrap_or(value).to_string()
}

fn extract_job_id(url: &str) -> Option<String> {
    let trimmed = url.trim_end_matches('/');
    let id = trimmed.split('/').next_back()?;
    if !id.is_empty() && id.chars().all(|ch| ch.is_ascii_digit()) {
        Some(id.to_string())
    } else {
        None
    }
}

fn parse_pub_date(value: &str) -> Option<String> {
    DateTime::parse_from_rfc2822(value).ok().map(|date| {
        date.with_timezone(&Utc)
            .format("%Y-%m-%dT%H:%M:%SZ")
            .to_string()
    })
}

fn html_fragment_to_text(value: &str) -> Option<String> {
    let decoded = decode_html_entities(value);
    let document = Html::parse_fragment(&decoded);
    let root = document.root_element();
    let text = collect_text(&root);
    if text.is_empty() { None } else { Some(text) }
}

fn decode_html_entities(value: &str) -> String {
    value
        .replace("&amp;nbsp;", " ")
        .replace("&nbsp;", " ")
        .replace("&#160;", " ")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&amp;", "&")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn extract_blocks(value: &str, tag: &str) -> Vec<String> {
    let start_tag = format!("<{tag}>");
    let end_tag = format!("</{tag}>");
    let mut remainder = value;
    let mut blocks = Vec::new();

    while let Some(start_index) = remainder.find(&start_tag) {
        remainder = &remainder[start_index + start_tag.len()..];
        let Some(end_index) = remainder.find(&end_tag) else {
            break;
        };
        blocks.push(remainder[..end_index].to_string());
        remainder = &remainder[end_index + end_tag.len()..];
    }

    blocks
}

fn extract_tag_text(value: &str, tag: &str) -> Option<String> {
    let start_tag = format!("<{tag}>");
    let end_tag = format!("</{tag}>");
    let start_index = value.find(&start_tag)?;
    let remainder = &value[start_index + start_tag.len()..];
    let end_index = remainder.find(&end_tag)?;
    Some(remainder[..end_index].trim().to_string())
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{
        extract_job_id, matches_keyword, normalize_feed_item, parse_detail_page, parse_feed,
    };
    use crate::models::{NormalizationResult, NormalizedJob, RawSnapshot};

    #[test]
    fn extracts_job_id_from_dou_url() {
        assert_eq!(
            extract_job_id("https://jobs.dou.ua/companies/getcode/vacancies/354587/"),
            Some("354587".to_string())
        );
    }

    #[test]
    fn parses_rss_feed_item() {
        let xml = r#"
            <rss>
                <channel>
                    <item>
                        <title>Backend Developer (Laravel) в GetCode, від&amp;nbsp;$2500, віддалено</title>
                        <link>https://jobs.dou.ua/companies/getcode/vacancies/354587/?utm_source=jobsrss</link>
                        <description>&lt;p&gt;Працюємо над CRM/ERP-продуктом.&lt;/p&gt;</description>
                        <pubDate>Sat, 18 Apr 2026 10:00:00 +0300</pubDate>
                    </item>
                </channel>
            </rss>
        "#;

        let items = parse_feed(xml);
        assert_eq!(items.len(), 1);
        assert!(matches_keyword(&items[0], Some("laravel")));

        let result = normalize_feed_item(items[0].clone(), "2026-04-18T10:00:00Z")
            .expect("RSS item should normalize");

        assert_eq!(result.job.title, "Backend Developer (Laravel)");
        assert_eq!(result.job.company_name, "GetCode");
        assert_eq!(result.job.location.as_deref(), Some("віддалено"));
        assert_eq!(result.job.salary_min, Some(2500));
        assert_eq!(result.job.salary_currency.as_deref(), Some("USD"));
        assert_eq!(
            result.job.posted_at.as_deref(),
            Some("2026-04-18T07:00:00Z")
        );
    }

    #[test]
    fn parses_dou_detail_fixture() {
        let html = include_str!("../../tests/fixtures/dou_ua_detail.html");
        let fallback = NormalizationResult {
            job: NormalizedJob {
                id: "job_dou_ua_354587".to_string(),
                title: "Backend Developer (Laravel)".to_string(),
                company_name: "GetCode".to_string(),
                location: Some("віддалено".to_string()),
                remote_type: Some("remote".to_string()),
                seniority: Some("middle".to_string()),
                description_text: "Short snippet".to_string(),
                salary_min: Some(2500),
                salary_max: None,
                salary_currency: Some("USD".to_string()),
                posted_at: Some("2026-04-18T07:00:00Z".to_string()),
                last_seen_at: "2026-04-18T10:00:00Z".to_string(),
                is_active: true,
            },
            snapshot: RawSnapshot {
                source: "dou_ua".to_string(),
                source_job_id: "354587".to_string(),
                source_url: "https://jobs.dou.ua/companies/getcode/vacancies/354587/".to_string(),
                raw_payload: json!({}),
                fetched_at: "2026-04-18T10:00:00Z".to_string(),
            },
        };

        let detail = parse_detail_page(html, &fallback, "2026-04-18T10:00:00Z");

        assert_eq!(detail.company_name.as_deref(), Some("GetCode"));
        assert_eq!(detail.location.as_deref(), Some("віддалено"));
        assert_eq!(detail.salary_min, Some(2500));
        assert_eq!(detail.salary_currency.as_deref(), Some("USD"));
        assert!(
            detail
                .description_text
                .as_deref()
                .expect("description should be present")
                .contains("CRM/ERP-продуктом")
        );
    }

    #[test]
    fn prefers_real_vacancy_body_over_sidebar_noise() {
        let html = include_str!("../../tests/fixtures/dou_ua_detail_regression.html");
        let fallback = NormalizationResult {
            job: NormalizedJob {
                id: "job_dou_ua_999".to_string(),
                title: "Front-end React Developer".to_string(),
                company_name: "SignalHire".to_string(),
                location: Some("віддалено".to_string()),
                remote_type: Some("remote".to_string()),
                seniority: Some("senior".to_string()),
                description_text: "Short listing snippet".to_string(),
                salary_min: None,
                salary_max: None,
                salary_currency: None,
                posted_at: Some("2026-04-18T07:00:00Z".to_string()),
                last_seen_at: "2026-04-18T10:00:00Z".to_string(),
                is_active: true,
            },
            snapshot: RawSnapshot {
                source: "dou_ua".to_string(),
                source_job_id: "999".to_string(),
                source_url: "https://jobs.dou.ua/companies/signalhire/vacancies/999/".to_string(),
                raw_payload: json!({}),
                fetched_at: "2026-04-18T10:00:00Z".to_string(),
            },
        };

        let detail = parse_detail_page(html, &fallback, "2026-04-18T10:00:00Z");
        let description = detail
            .description_text
            .expect("detail description should be present");

        assert!(description.contains("Build and evolve a frontend platform"));
        assert!(description.contains("Collaborate closely with product"));
        assert!(!description.contains("Схожі вакансії"));
        assert!(!description.contains("Відгукнутися"));
    }
}
