mod adapters;
mod db;
mod models;
mod scrapers;

use std::env;
use std::fs;
use std::path::PathBuf;
use std::time::Duration;

use tracing::{error, info, warn};

#[cfg(any(feature = "mock", test))]
use crate::adapters::SourceAdapter;
#[cfg(any(feature = "mock", test))]
use crate::adapters::mock_source::MockSourceAdapter;
use crate::models::{IngestionBatch, InputDocument};
#[cfg(any(feature = "mock", test))]
use crate::models::MockSourceInput;
use crate::scrapers::ScraperConfig;
use crate::scrapers::djinni::DjinniScraper;
use crate::scrapers::dou_ua::DouUaScraper;
use crate::scrapers::robota_ua::RobotaUaScraper;
use crate::scrapers::work_ua::WorkUaScraper;

// ── Config ─────────────────────────────────────────────────────────────────

struct Config {
    database_url: String,
    run_mode: RunMode,
}

enum RunMode {
    File(FileMode),
    Scrape(ScrapeMode),
    Daemon(DaemonMode),
}

struct FileMode {
    input_path: PathBuf,
    input_format: InputFormat,
}

struct ScrapeMode {
    source: ScrapeSource,
    pages: u32,
    keyword: Option<String>,
}

/// Runs all configured sources in a loop with a fixed interval.
struct DaemonMode {
    sources: Vec<ScrapeSource>,
    pages: u32,
    interval_minutes: u64,
    keyword: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum InputFormat {
    Normalized,
    #[cfg(any(feature = "mock", test))]
    MockSource,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ScrapeSource {
    Djinni,
    DouUa,
    WorkUa,
    RobotaUa,
}

// ── Entry point ─────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() -> Result<(), String> {
    tracing_subscriber::fmt().with_env_filter("info").init();

    let config = Config::from_env()?;

    if let RunMode::Daemon(ref daemon_mode) = config.run_mode {
        let pool = db::connect(&config.database_url).await?;
        db::run_migrations(&pool).await?;
        info!("migrations applied");
        return run_daemon(daemon_mode, &pool).await;
    }

    let batch = match config.run_mode {
        RunMode::File(ref file_mode) => load_batch(file_mode)?,
        RunMode::Scrape(ref scrape_mode) => run_scraper(scrape_mode).await?,
        RunMode::Daemon(_) => unreachable!(),
    };

    if batch.jobs.is_empty() {
        return Err("no jobs to ingest".to_string());
    }

    let pool = db::connect(&config.database_url).await?;

    // Run migrations if RUN_DB_MIGRATIONS=true — useful when running ingestion
    // standalone without engine-api having started first.
    if env::var("RUN_DB_MIGRATIONS")
        .map(|v| v.trim().to_lowercase())
        .as_deref()
        == Ok("true")
    {
        db::run_migrations(&pool).await?;
        info!("migrations applied");
    }

    let summary = db::upsert_batch(&pool, &batch).await?;

    info!(
        jobs_written = summary.jobs_written,
        variants_created = summary.variants_created,
        variants_updated = summary.variants_updated,
        variants_unchanged = summary.variants_unchanged,
        variants_inactivated = summary.variants_inactivated,
        jobs_inactivated = summary.jobs_inactivated,
        jobs_reactivated = summary.jobs_reactivated,
        sources_refreshed = summary.sources_refreshed,
        "ingestion completed"
    );

    println!(
        "Wrote {} jobs; variants created: {}, updated: {}, unchanged: {}, inactivated: {}; jobs inactivated: {}, reactivated: {}; sources refreshed: {}",
        summary.jobs_written,
        summary.variants_created,
        summary.variants_updated,
        summary.variants_unchanged,
        summary.variants_inactivated,
        summary.jobs_inactivated,
        summary.jobs_reactivated,
        summary.sources_refreshed
    );
    Ok(())
}

// ── Config parsing ──────────────────────────────────────────────────────────

impl Config {
    fn from_env() -> Result<Self, String> {
        let database_url = env::var("DATABASE_URL")
            .ok()
            .map(|v| v.trim().to_string())
            .filter(|v| !v.is_empty())
            .ok_or_else(|| "DATABASE_URL is required".to_string())?;

        let mut args = env::args().skip(1);
        let flag = args.next().ok_or_else(|| usage_error())?;

        let run_mode = match flag.as_str() {
            "--input" => {
                let path = args.next().ok_or("missing value for --input")?;
                let mut input_format = InputFormat::Normalized;
                while let Some(f) = args.next() {
                    if f != "--input-format" {
                        return Err(format!(
                            "unsupported argument '{f}', expected '--input-format'"
                        ));
                    }
                    let val = args.next().ok_or("missing value for --input-format")?;
                    input_format = InputFormat::from_cli(&val)?;
                }
                RunMode::File(FileMode {
                    input_path: PathBuf::from(path),
                    input_format,
                })
            }
            "--source" => {
                let source_str = args.next().ok_or("missing value for --source")?;
                let source = ScrapeSource::from_cli(&source_str)?;
                let mut pages: u32 = 3;
                let mut keyword: Option<String> = None;

                while let Some(f) = args.next() {
                    match f.as_str() {
                        "--pages" => {
                            let val = args.next().ok_or("missing value for --pages")?;
                            pages = val.parse::<u32>().map_err(|_| {
                                format!("--pages must be a positive integer, got '{val}'")
                            })?;
                            if pages == 0 {
                                return Err("--pages must be at least 1".to_string());
                            }
                        }
                        "--keyword" => {
                            keyword = Some(args.next().ok_or("missing value for --keyword")?);
                        }
                        other => return Err(format!("unsupported argument '{other}'")),
                    }
                }

                RunMode::Scrape(ScrapeMode {
                    source,
                    pages,
                    keyword,
                })
            }
            "--daemon" => {
                let mut sources: Vec<ScrapeSource> = Vec::new();
                let mut pages: u32 = 3;
                let mut interval_minutes: u64 = 60;
                let mut keyword: Option<String> = None;

                while let Some(f) = args.next() {
                    match f.as_str() {
                        "--sources" => {
                            let val = args.next().ok_or("missing value for --sources")?;
                            for s in val.split(',') {
                                let s = s.trim();
                                if s == "all" {
                                    sources.push(ScrapeSource::Djinni);
                                    sources.push(ScrapeSource::DouUa);
                                    sources.push(ScrapeSource::WorkUa);
                                    sources.push(ScrapeSource::RobotaUa);
                                } else {
                                    sources.push(ScrapeSource::from_cli(s)?);
                                }
                            }
                        }
                        "--pages" => {
                            let val = args.next().ok_or("missing value for --pages")?;
                            pages = val.parse::<u32>().map_err(|_| {
                                format!("--pages must be a positive integer, got '{val}'")
                            })?;
                            if pages == 0 {
                                return Err("--pages must be at least 1".to_string());
                            }
                        }
                        "--interval-minutes" => {
                            let val = args.next().ok_or("missing value for --interval-minutes")?;
                            interval_minutes = val.parse::<u64>().map_err(|_| {
                                format!(
                                    "--interval-minutes must be a positive integer, got '{val}'"
                                )
                            })?;
                            if interval_minutes == 0 {
                                return Err("--interval-minutes must be at least 1".to_string());
                            }
                        }
                        "--keyword" => {
                            keyword = Some(args.next().ok_or("missing value for --keyword")?);
                        }
                        other => return Err(format!("unsupported daemon argument '{other}'")),
                    }
                }

                // Default: scrape all sources
                if sources.is_empty() {
                    sources.push(ScrapeSource::Djinni);
                    sources.push(ScrapeSource::DouUa);
                    sources.push(ScrapeSource::WorkUa);
                    sources.push(ScrapeSource::RobotaUa);
                }
                sources.dedup();

                RunMode::Daemon(DaemonMode {
                    sources,
                    pages,
                    interval_minutes,
                    keyword,
                })
            }
            other => return Err(format!("unsupported flag '{other}'\n{}", usage_error())),
        };

        Ok(Self {
            database_url,
            run_mode,
        })
    }
}

fn usage_error() -> String {
    #[cfg(any(feature = "mock", test))]
    let fmt = "normalized|mock-source";
    #[cfg(not(any(feature = "mock", test)))]
    let fmt = "normalized";
    format!(
        "usage:\n  cargo run -- --input <path> [--input-format {fmt}]\n  cargo run -- --source <djinni|douua|workua|robotaua> [--pages <n>] [--keyword <kw>]\n  cargo run -- --daemon [--sources all|djinni|douua|workua|robotaua] [--pages <n>] [--interval-minutes <m>] [--keyword <kw>]"
    )
}

impl InputFormat {
    fn from_cli(value: &str) -> Result<Self, String> {
        match value.trim() {
            "normalized" => Ok(Self::Normalized),
            #[cfg(any(feature = "mock", test))]
            "mock-source" => Ok(Self::MockSource),
            other => Err(format!(
                "unsupported input format '{other}', expected 'normalized'"
            )),
        }
    }
}

impl ScrapeSource {
    fn from_cli(value: &str) -> Result<Self, String> {
        match value.trim().to_lowercase().as_str() {
            "djinni" => Ok(Self::Djinni),
            "dou" | "douua" | "dou_ua" | "dou.ua" => Ok(Self::DouUa),
            "workua" | "work_ua" | "work.ua" => Ok(Self::WorkUa),
            "robotaua" | "robota_ua" | "robota.ua" => Ok(Self::RobotaUa),
            other => Err(format!(
                "unsupported source '{other}', expected 'djinni', 'douua', 'workua', or 'robotaua'"
            )),
        }
    }

    fn name(self) -> &'static str {
        match self {
            Self::Djinni => "djinni",
            Self::DouUa => "dou_ua",
            Self::WorkUa => "work_ua",
            Self::RobotaUa => "robota_ua",
        }
    }
}

// ── Batch loaders ────────────────────────────────────────────────────────────

fn load_batch(mode: &FileMode) -> Result<IngestionBatch, String> {
    info!(
        input_path = %mode.input_path.display(),
        input_format = match mode.input_format {
            InputFormat::Normalized => "normalized",
            #[cfg(any(feature = "mock", test))]
            InputFormat::MockSource => "mock-source",
        },
        "loading ingestion batch from file"
    );

    let raw = fs::read_to_string(&mode.input_path)
        .map_err(|e| format!("failed to read {}: {e}", mode.input_path.display()))?;

    match mode.input_format {
        InputFormat::Normalized => {
            let payload = serde_json::from_str::<InputDocument>(&raw)
                .map_err(|e| format!("failed to parse {}: {e}", mode.input_path.display()))?;
            let batch = IngestionBatch::from_jobs(payload.into_jobs());
            info!(
                input_path = %mode.input_path.display(),
                jobs = batch.jobs.len(),
                "loaded normalized ingestion batch"
            );
            Ok(batch)
        }
        #[cfg(any(feature = "mock", test))]
        InputFormat::MockSource => {
            let payload = serde_json::from_str::<MockSourceInput>(&raw)
                .map_err(|e| format!("failed to parse {}: {e}", mode.input_path.display()))?;
            let adapter = MockSourceAdapter;
            let normalized = adapter.normalize(payload).map_err(|error| {
                format!(
                    "failed to normalize {} as mock-source input: {error}",
                    mode.input_path.display()
                )
            })?;
            let batch =
                IngestionBatch::from_normalization_results(normalized).map_err(|error| {
                    format!(
                        "failed to build ingestion batch from {}: {error}",
                        mode.input_path.display()
                    )
                })?;
            info!(
                input_path = %mode.input_path.display(),
                jobs = batch.jobs.len(),
                variants = batch.job_variants.len(),
                "loaded adapter-backed ingestion batch"
            );
            Ok(batch)
        }
    }
}

async fn run_scraper(mode: &ScrapeMode) -> Result<IngestionBatch, String> {
    info!(
        source = mode.source.name(),
        pages = mode.pages,
        keyword = mode.keyword.as_deref().unwrap_or(""),
        "starting scrape"
    );

    let config = ScraperConfig {
        pages: mode.pages,
        keyword: mode.keyword.clone(),
        page_delay_ms: 600,
    };

    let results = match mode.source {
        ScrapeSource::Djinni => {
            let scraper = DjinniScraper::new()?;
            scraper.scrape(&config).await?
        }
        ScrapeSource::DouUa => {
            let scraper = DouUaScraper::new()?;
            scraper.scrape(&config).await?
        }
        ScrapeSource::WorkUa => {
            let scraper = WorkUaScraper::new()?;
            scraper.scrape(&config).await?
        }
        ScrapeSource::RobotaUa => {
            let scraper = RobotaUaScraper::new()?;
            scraper.scrape(&config).await?
        }
    };

    if results.is_empty() {
        return Err(
            "scraper returned no jobs — site structure may have changed, check selectors"
                .to_string(),
        );
    }

    let batch = IngestionBatch::from_normalization_results(results).map_err(|error| {
        format!(
            "scraper '{}' returned invalid normalization results: {error}",
            mode.source.name()
        )
    })?;

    info!(
        source = mode.source.name(),
        jobs = batch.jobs.len(),
        variants = batch.job_variants.len(),
        "scrape finished"
    );

    Ok(batch)
}

// ── Daemon loop ──────────────────────────────────────────────────────────────

/// Runs all configured sources on repeat, sleeping `interval_minutes` between rounds.
/// Never returns unless a fatal error occurs.
async fn run_daemon(mode: &DaemonMode, pool: &sqlx::PgPool) -> Result<(), String> {
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

            match run_scraper(&scrape_mode).await {
                Ok(batch) => match db::upsert_batch(pool, &batch).await {
                    Ok(summary) => info!(
                        source = source.name(),
                        jobs_written = summary.jobs_written,
                        variants_created = summary.variants_created,
                        variants_updated = summary.variants_updated,
                        variants_unchanged = summary.variants_unchanged,
                        variants_inactivated = summary.variants_inactivated,
                        "daemon round complete"
                    ),
                    Err(e) => error!(source = source.name(), error = %e, "db upsert failed"),
                },
                Err(e) => {
                    warn!(source = source.name(), error = %e, "scrape failed, skipping source")
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

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use std::fs;

    use crate::models::{InputDocument, canonical_job_id, compute_dedupe_key};

    use super::{FileMode, InputFormat, load_batch};

    #[test]
    fn parses_wrapped_job_input() {
        let payload = serde_json::from_str::<InputDocument>(
            r#"{"jobs":[{"id":"job-1","title":"Backend Engineer","company_name":"Nova","description_text":"Rust","last_seen_at":"2026-04-14T08:00:00Z"}]}"#,
        )
        .expect("wrapped payload should parse");

        assert_eq!(payload.into_jobs().len(), 1);
    }

    #[test]
    fn parses_array_job_input() {
        let payload = serde_json::from_str::<InputDocument>(
            r#"[{"id":"job-1","title":"Backend Engineer","company_name":"Nova","description_text":"Rust","last_seen_at":"2026-04-14T08:00:00Z"}]"#,
        )
        .expect("array payload should parse");

        let jobs = payload.into_jobs();
        assert_eq!(jobs.len(), 1);
        assert!(jobs[0].is_active);
    }

    #[test]
    fn returns_read_error_for_missing_file() {
        let error = load_batch(&FileMode {
            input_path: "/definitely/missing.json".into(),
            input_format: InputFormat::Normalized,
        })
        .expect_err("missing file should fail");

        assert!(error.contains("failed to read"));
    }

    #[test]
    fn parses_mock_source_input_into_jobs() {
        let path =
            std::env::temp_dir().join(format!("ingestion-mock-source-{}.json", std::process::id()));
        fs::write(
            &path,
            r#"{
              "fetched_at": "2026-04-14T10:00:00Z",
              "jobs": [
                {
                  "source_job_id": "777",
                  "source_url": "https://example.com/jobs/777",
                  "position": "Platform Engineer",
                  "employer": "SignalHire",
                  "city": "Kyiv",
                  "work_mode": "remote",
                  "seniority": "senior",
                  "description": "Normalization and dedupe",
                  "compensation": {
                    "min": 5000,
                    "max": 6500,
                    "currency": "USD"
                  },
                  "posted_at": "2026-04-14T09:00:00Z",
                  "last_seen_at": "2026-04-14T10:00:00Z",
                  "active": true
                }
              ]
            }"#,
        )
        .expect("mock input file should be written");

        let batch = load_batch(&FileMode {
            input_path: path.clone(),
            input_format: InputFormat::MockSource,
        })
        .expect("mock source jobs should load");

        fs::remove_file(path).ok();

        let expected_dedupe_key = compute_dedupe_key(&batch.jobs[0]);

        assert_eq!(batch.jobs.len(), 1);
        assert_eq!(batch.jobs[0].id, canonical_job_id(&expected_dedupe_key));
        assert_eq!(batch.jobs[0].company_name, "SignalHire");
        assert_eq!(batch.job_variants.len(), 1);
        assert_eq!(batch.job_variants[0].job_id, batch.jobs[0].id);
        assert_eq!(batch.job_variants[0].dedupe_key, expected_dedupe_key);
        assert_eq!(batch.job_variants[0].source, "mock_source");
    }
}
