use std::time::Duration;

use tracing::{error, info, warn};

const MAX_SCRAPE_ATTEMPTS: u8 = 3;
const SCRAPE_BACKOFF_SECS: [u64; 2] = [5, 15];

use crate::cli::{DaemonMode, ScrapeMode, run_scraper};
use crate::db;

/// Runs all configured sources on repeat, sleeping `interval_minutes` between rounds.
/// Never returns unless a fatal error occurs.
pub(crate) async fn run_daemon(mode: &DaemonMode, pool: &sqlx::PgPool) -> Result<(), String> {
    let interval = Duration::from_secs(mode.interval_minutes * 60);
    info!(
        sources = ?mode.sources.iter().map(|s| s.name()).collect::<Vec<_>>(),
        pages = mode.pages,
        interval_minutes = mode.interval_minutes,
        "daemon started"
    );

    loop {
        for &source in &mode.sources {
            let scrape_mode = ScrapeMode {
                source,
                pages: mode.pages,
                keyword: mode.keyword.clone(),
            };

            let scrape_result = {
                let mut result;
                let mut attempt = 0u8;
                loop {
                    result = run_scraper(&scrape_mode).await;
                    attempt += 1;
                    match &result {
                        Ok(_) => break,
                        Err(error) if attempt < MAX_SCRAPE_ATTEMPTS => {
                            let delay = SCRAPE_BACKOFF_SECS[usize::from(attempt - 1)];
                            warn!(
                                source = source.name(),
                                attempt,
                                delay_secs = delay,
                                error = %error,
                                "scrape failed, retrying"
                            );
                            tokio::time::sleep(Duration::from_secs(delay)).await;
                        }
                        Err(_) => break,
                    }
                }
                result
            };

            match scrape_result {
                Ok(batch) => match db::upsert_batch(pool, &batch).await {
                    Ok(summary) => {
                        let market_snapshot_summary = match db::refresh_market_snapshots(pool).await
                        {
                            Ok(summary) => Some(summary),
                            Err(error) => {
                                warn!(
                                    source = source.name(),
                                    error = %error,
                                    "market snapshot refresh failed after daemon upsert"
                                );
                                None
                            }
                        };

                        info!(
                            source = source.name(),
                            jobs_written = summary.jobs_written,
                            variants_created = summary.variants_created,
                            variants_updated = summary.variants_updated,
                            variants_unchanged = summary.variants_unchanged,
                            variants_inactivated = summary.variants_inactivated,
                            market_snapshots_written = market_snapshot_summary
                                .as_ref()
                                .map(|value| value.snapshots_written)
                                .unwrap_or(0),
                            "daemon round complete"
                        )
                    }
                    Err(error) => {
                        error!(source = source.name(), error = %error, "db upsert failed")
                    }
                },
                Err(error) => {
                    error!(
                        source = source.name(),
                        attempts = MAX_SCRAPE_ATTEMPTS,
                        error = %error,
                        "scrape failed after all retries, skipping source"
                    )
                }
            }
        }

        info!(
            next_in_minutes = mode.interval_minutes,
            "sleeping until next round"
        );
        tokio::time::sleep(interval).await;
    }
}
