mod adapters;
mod cli;
mod daemon;
mod db;
mod db_runtime;
mod models;
mod scrapers;

use std::env;

use tracing::{info, warn};

use crate::cli::{Config, RunMode, load_batch, run_scraper};

// ── Entry point ─────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() -> Result<(), String> {
    tracing_subscriber::fmt().with_env_filter("info").init();

    let config = Config::from_env()?;

    if let RunMode::Daemon(ref daemon_mode) = config.run_mode {
        let pool = db_runtime::connect(&config.database_url).await?;
        db_runtime::run_migrations(&pool).await?;
        info!("migrations applied");
        return daemon::run_daemon(daemon_mode, &pool).await;
    }

    let batch = match config.run_mode {
        RunMode::File(ref file_mode) => load_batch(file_mode)?,
        RunMode::Scrape(ref scrape_mode) => run_scraper(scrape_mode).await?,
        RunMode::Daemon(_) => unreachable!(),
    };

    if batch.jobs.is_empty() {
        return Err("no jobs to ingest".to_string());
    }

    let pool = db_runtime::connect(&config.database_url).await?;

    // Run migrations if RUN_DB_MIGRATIONS=true — useful when running ingestion
    // standalone without engine-api having started first.
    if env::var("RUN_DB_MIGRATIONS")
        .map(|v| v.trim().to_lowercase())
        .as_deref()
        == Ok("true")
    {
        db_runtime::run_migrations(&pool).await?;
        info!("migrations applied");
    }

    let summary = db::upsert_batch(&pool, &batch).await?;
    let market_snapshot_summary = match db::refresh_market_snapshots(&pool).await {
        Ok(summary) => Some(summary),
        Err(error) => {
            warn!(error = %error, "market snapshot refresh failed after ingestion");
            None
        }
    };

    info!(
        jobs_written = summary.jobs_written,
        variants_created = summary.variants_created,
        variants_updated = summary.variants_updated,
        variants_unchanged = summary.variants_unchanged,
        variants_inactivated = summary.variants_inactivated,
        jobs_inactivated = summary.jobs_inactivated,
        jobs_reactivated = summary.jobs_reactivated,
        sources_refreshed = summary.sources_refreshed,
        market_snapshots_written = market_snapshot_summary
            .as_ref()
            .map(|value| value.snapshots_written)
            .unwrap_or(0),
        "ingestion completed"
    );

    println!(
        "Wrote {} jobs; variants created: {}, updated: {}, unchanged: {}, inactivated: {}; jobs inactivated: {}, reactivated: {}; sources refreshed: {}; market snapshots refreshed: {}",
        summary.jobs_written,
        summary.variants_created,
        summary.variants_updated,
        summary.variants_unchanged,
        summary.variants_inactivated,
        summary.jobs_inactivated,
        summary.jobs_reactivated,
        summary.sources_refreshed,
        market_snapshot_summary
            .as_ref()
            .map(|value| value.snapshots_written)
            .unwrap_or(0)
    );
    Ok(())
}
