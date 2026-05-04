use std::env;
use std::time::Instant;

use tracing::{info, warn};

use crate::cli::{FileMode, ScrapeMode, load_batch, run_scraper};
use crate::db;
use crate::db_runtime;
use crate::error::{IngestionError, Result};

pub(crate) async fn run_scrape(mode: &ScrapeMode, pool: &sqlx::PgPool) -> Result<()> {
    run_migrations_if_requested(pool).await?;
    let started = Instant::now();

    let scrape_output = match run_scraper(mode).await {
        Ok(output) => output,
        Err(error) => {
            db::record_ingestion_run(
                pool,
                &db::IngestionRunMetrics {
                    source: mode.source.name(),
                    jobs_fetched: 0,
                    jobs_attempted: 0,
                    jobs_upserted: 0,
                    jobs_failed: 0,
                    errors: 1,
                    errors_json: vec!["scrape_failed".to_string()],
                    duration_ms: started.elapsed().as_millis() as u64,
                    status: db::IngestionRunStatus::Failed,
                },
            )
            .await?;
            return Err(error);
        }
    };
    let batch = scrape_output.batch;

    let summary = match db::upsert_batch(pool, &batch).await {
        Ok(summary) => summary,
        Err(error) => {
            db::record_ingestion_run(
                pool,
                &db::IngestionRunMetrics {
                    source: mode.source.name(),
                    jobs_fetched: batch.jobs.len() as u32,
                    jobs_attempted: scrape_output.jobs_attempted,
                    jobs_upserted: 0,
                    jobs_failed: scrape_output.jobs_failed,
                    errors: scrape_output.jobs_failed + 1,
                    errors_json: append_error(scrape_output.errors.clone(), "db_upsert_failed"),
                    duration_ms: started.elapsed().as_millis() as u64,
                    status: db::IngestionRunStatus::Failed,
                },
            )
            .await?;
            return Err(error);
        }
    };

    let market_snapshot_summary = match db::refresh_market_snapshots(pool).await {
        Ok(summary) => Some(summary),
        Err(error) => {
            warn!(error = %error, "market snapshot refresh failed after ingestion");
            None
        }
    };
    let run_errors = (if market_snapshot_summary.is_none() {
        1
    } else {
        0
    }) + scrape_output.jobs_failed;
    let errors_json = if market_snapshot_summary.is_none() {
        append_error(
            scrape_output.errors.clone(),
            "market_snapshot_refresh_failed",
        )
    } else {
        scrape_output.errors.clone()
    };
    db::record_ingestion_run(
        pool,
        &db::IngestionRunMetrics {
            source: mode.source.name(),
            jobs_fetched: batch.jobs.len() as u32,
            jobs_attempted: scrape_output.jobs_attempted,
            jobs_upserted: summary.jobs_written as u32,
            jobs_failed: scrape_output.jobs_failed,
            errors: run_errors,
            errors_json,
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
    Ok(())
}

pub(crate) async fn run_file(mode: &FileMode, database_url: &str) -> Result<()> {
    let batch = load_batch(mode)?;

    if batch.jobs.is_empty() {
        return Err(IngestionError::Input("no jobs to ingest".to_string()));
    }

    let pool = db_runtime::connect(database_url).await?;
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

async fn run_migrations_if_requested(pool: &sqlx::PgPool) -> Result<()> {
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

fn append_error(mut errors: Vec<String>, error: &str) -> Vec<String> {
    errors.push(error.to_string());
    errors
}
