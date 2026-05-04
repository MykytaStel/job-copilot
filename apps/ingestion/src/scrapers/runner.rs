use reqwest::Client;
use tracing::warn;

use crate::models::NormalizationResult;

use super::detail_merge::{DetailSnapshot, merge_detail_into_result};
use super::http::{fetch_with_backoff, polite_delay};
use super::company::normalize_company_name;

/// Shared scraping behaviour: HTTP fetch, per-result detail enrichment, polite pacing.
///
/// Implement `SOURCE`, `client`, and `parse_detail`; get `fetch_url`,
/// `enrich_result`, and `enrich_results` for free.
pub trait JobSource {
    const SOURCE: &'static str;

    fn client(&self) -> &Client;

    fn parse_detail(
        &self,
        html: &str,
        context: &NormalizationResult,
        fetched_at: &str,
    ) -> DetailSnapshot;

    async fn fetch_url(&self, url: &str) -> crate::error::Result<String> {
        fetch_with_backoff(self.client(), url).await
    }

    async fn enrich_result(
        &self,
        result: NormalizationResult,
        fetched_at: &str,
    ) -> Option<NormalizationResult> {
        let source_url = result.snapshot.source_url.clone();

        let Some(detail_html) = self.fetch_url(&source_url).await.ok() else {
            if normalize_company_name(&result.job.company_name).is_none() {
                warn!(
                    source = Self::SOURCE,
                    %source_url,
                    "detail fetch failed and company is still missing"
                );
                return None;
            }
            return Some(result);
        };

        let detail = self.parse_detail(&detail_html, &result, fetched_at);
        merge_detail_into_result(result, detail)
    }

    async fn enrich_results(
        &self,
        results: Vec<NormalizationResult>,
        fetched_at: &str,
    ) -> Vec<NormalizationResult> {
        let total = results.len();
        let mut enriched = Vec::with_capacity(total);

        for (index, result) in results.into_iter().enumerate() {
            match self.enrich_result(result, fetched_at).await {
                Some(result) => enriched.push(result),
                None => warn!(source = Self::SOURCE, "skipped job after detail fetch"),
            }

            if index + 1 < total {
                polite_delay(150).await;
            }
        }

        enriched
    }
}
