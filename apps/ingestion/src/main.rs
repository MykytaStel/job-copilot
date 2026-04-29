mod adapters;
mod cli;
mod daemon;
mod db;
mod db_runtime;
mod models;
mod scrapers;

use std::env;
use std::time::Instant;

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

    if let RunMode::Scrape(ref scrape_mode) = config.run_mode {
        let pool = db_runtime::connect(&config.database_url).await?;
        run_migrations_if_requested(&pool).await?;
        let started = Instant::now();

        let batch = match run_scraper(scrape_mode).await {
            Ok(batch) => batch,
            Err(error) => {
                db::record_ingestion_run(
                    &pool,
                    &db::IngestionRunMetrics {
                        source: scrape_mode.source.name(),
                        jobs_fetched: 0,
                        jobs_upserted: 0,
                        errors: 1,
                        duration_ms: started.elapsed().as_millis() as u64,
                        status: db::IngestionRunStatus::Failed,
                    },
                )
                .await?;
                return Err(error);
            }
        };

        let summary = match db::upsert_batch(&pool, &batch).await {
            Ok(summary) => summary,
            Err(error) => {
                db::record_ingestion_run(
                    &pool,
                    &db::IngestionRunMetrics {
                        source: scrape_mode.source.name(),
                        jobs_fetched: batch.jobs.len() as u32,
                        jobs_upserted: 0,
                        errors: 1,
                        duration_ms: started.elapsed().as_millis() as u64,
                        status: db::IngestionRunStatus::Failed,
                    },
                )
                .await?;
                return Err(error);
            }
        };

        let market_snapshot_summary = match db::refresh_market_snapshots(&pool).await {
            Ok(summary) => Some(summary),
            Err(error) => {
                warn!(error = %error, "market snapshot refresh failed after ingestion");
                None
            }
        };
        let run_errors = if market_snapshot_summary.is_none() {
            1
        } else {
            0
        };
        db::record_ingestion_run(
            &pool,
            &db::IngestionRunMetrics {
                source: scrape_mode.source.name(),
                jobs_fetched: batch.jobs.len() as u32,
                jobs_upserted: summary.jobs_written as u32,
                errors: run_errors,
                duration_ms: started.elapsed().as_millis() as u64,
                status: if run_errors > 0 {
                    db::IngestionRunStatus::Partial
                } else {
                    db::IngestionRunStatus::Ok
                },
            },
        )
        .await?;

        print_summary(&summary, market_snapshot_summary.as_ref());
        return Ok(());
    }

    let batch = match config.run_mode {
        RunMode::File(ref file_mode) => load_batch(file_mode)?,
        RunMode::Scrape(_) => unreachable!(),
        RunMode::Daemon(_) => unreachable!(),
    };

    if batch.jobs.is_empty() {
        return Err("no jobs to ingest".to_string());
    }

    let pool = db_runtime::connect(&config.database_url).await?;

    run_migrations_if_requested(&pool).await?;

    let summary = db::upsert_batch(&pool, &batch).await?;
    let market_snapshot_summary = match db::refresh_market_snapshots(&pool).await {
        Ok(summary) => Some(summary),
        Err(error) => {
            warn!(error = %error, "market snapshot refresh failed after ingestion");
            None
        }
    };

    print_summary(&summary, market_snapshot_summary.as_ref());
    Ok(())
}

async fn run_migrations_if_requested(pool: &sqlx::PgPool) -> Result<(), String> {
    // Run migrations if RUN_DB_MIGRATIONS=true — useful when running ingestion
    // standalone without engine-api having started first.
    if env::var("RUN_DB_MIGRATIONS")
        .map(|v| v.trim().to_lowercase())
        .as_deref()
        == Ok("true")
    {
        db_runtime::run_migrations(pool).await?;
        info!("migrations applied");
    }

    Ok(())
}

fn print_summary(
    summary: &db::UpsertSummary,
    market_snapshot_summary: Option<&db::MarketSnapshotSummary>,
) {
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
            .map(|value| value.snapshots_written)
            .unwrap_or(0)
    );
}
