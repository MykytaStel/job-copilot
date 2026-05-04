use std::time::Duration;

use reqwest::Client;
use tokio::time::sleep;
use tracing::warn;

use crate::models::NormalizationResult;

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
pub async fn fetch_with_backoff(client: &Client, url: &str) -> crate::error::Result<String> {
    use crate::error::IngestionError;

    const MAX_RETRIES: u32 = 3;
    const BASE_DELAY_MS: u64 = 1_000;

    let mut attempt = 0u32;
    loop {
        let response = client.get(url).send().await?;

        let status = response.status();

        if status.is_success() {
            return Ok(response.text().await?);
        }

        if status.as_u16() == 429 || status.is_server_error() {
            if attempt >= MAX_RETRIES {
                warn!(
                    status = status.as_u16(),
                    url,
                    retries = MAX_RETRIES,
                    "giving up after max retries due to rate limiting or server error"
                );
                return Err(IngestionError::Scraper(format!(
                    "HTTP {status} after {MAX_RETRIES} retries: {url}"
                )));
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

        return Err(IngestionError::Scraper(format!(
            "HTTP error: {status}: {url}"
        )));
    }
}
