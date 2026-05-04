use std::env;
use std::fs;
use std::path::PathBuf;

use job_copilot_domain::source::SourceId;
use tracing::info;

#[cfg(any(feature = "mock", test))]
use crate::adapters::SourceAdapter;
#[cfg(any(feature = "mock", test))]
use crate::adapters::mock_source::MockSourceAdapter;
#[cfg(any(feature = "mock", test))]
use crate::models::MockSourceInput;
use crate::models::{IngestionBatch, InputDocument};
use crate::scrapers::ScraperConfig;
use crate::scrapers::djinni::DjinniScraper;
use crate::scrapers::dou_ua::DouUaScraper;
use crate::scrapers::robota_ua::RobotaUaScraper;
use crate::scrapers::work_ua::WorkUaScraper;

pub(crate) struct Config {
    pub(crate) database_url: String,
    pub(crate) run_mode: RunMode,
}

pub(crate) enum RunMode {
    File(FileMode),
    Scrape(ScrapeMode),
    Daemon(DaemonMode),
}

pub(crate) struct FileMode {
    pub(crate) input_path: PathBuf,
    pub(crate) input_format: InputFormat,
}

pub(crate) struct ScrapeMode {
    pub(crate) source: ScrapeSource,
    pub(crate) pages: u32,
    pub(crate) keyword: Option<String>,
}

pub(crate) struct ScrapeOutput {
    pub(crate) batch: IngestionBatch,
    pub(crate) jobs_attempted: u32,
    pub(crate) jobs_failed: u32,
    pub(crate) errors: Vec<String>,
}

/// Runs all configured sources in a loop with a fixed interval.
pub(crate) struct DaemonMode {
    pub(crate) sources: Vec<ScrapeSource>,
    pub(crate) pages: u32,
    pub(crate) interval_minutes: u64,
    pub(crate) keyword: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum InputFormat {
    Normalized,
    #[cfg(any(feature = "mock", test))]
    MockSource,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ScrapeSource(SourceId);

impl Config {
    pub(crate) fn from_env() -> Result<Self, String> {
        let database_url = env::var("DATABASE_URL")
            .ok()
            .map(|v| v.trim().to_string())
            .filter(|v| !v.is_empty())
            .ok_or_else(|| "DATABASE_URL is required".to_string())?;

        let mut args = env::args().skip(1);
        let flag = args.next().ok_or_else(usage_error)?;

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
                                    sources.push(ScrapeSource::DJINNI);
                                    sources.push(ScrapeSource::DOU_UA);
                                    sources.push(ScrapeSource::WORK_UA);
                                    sources.push(ScrapeSource::ROBOTA_UA);
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

                if sources.is_empty() {
                    sources.push(ScrapeSource::DJINNI);
                    sources.push(ScrapeSource::DOU_UA);
                    sources.push(ScrapeSource::WORK_UA);
                    sources.push(ScrapeSource::ROBOTA_UA);
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
    pub(crate) const DJINNI: Self = Self(SourceId::Djinni);
    pub(crate) const DOU_UA: Self = Self(SourceId::DouUa);
    pub(crate) const WORK_UA: Self = Self(SourceId::WorkUa);
    pub(crate) const ROBOTA_UA: Self = Self(SourceId::RobotaUa);

    fn from_cli(value: &str) -> Result<Self, String> {
        match value.trim().to_lowercase().as_str() {
            "djinni" => Ok(Self::DJINNI),
            "dou" | "douua" | "dou_ua" | "dou.ua" => Ok(Self::DOU_UA),
            "workua" | "work_ua" | "work.ua" => Ok(Self::WORK_UA),
            "robotaua" | "robota_ua" | "robota.ua" => Ok(Self::ROBOTA_UA),
            other => Err(format!(
                "unsupported source '{other}', expected 'djinni', 'douua', 'workua', or 'robotaua'"
            )),
        }
    }

    pub(crate) fn source_id(self) -> SourceId {
        self.0
    }

    pub(crate) fn name(self) -> &'static str {
        self.0.canonical_key()
    }
}

pub(crate) fn load_batch(mode: &FileMode) -> Result<IngestionBatch, String> {
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

pub(crate) async fn run_scraper(mode: &ScrapeMode) -> Result<ScrapeOutput, String> {
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

    let scrape_run = match mode.source.source_id() {
        SourceId::Djinni => {
            let scraper = DjinniScraper::new()?;
            scraper.scrape(&config).await?
        }
        SourceId::DouUa => {
            let scraper = DouUaScraper::new()?;
            scraper.scrape(&config).await?
        }
        SourceId::WorkUa => {
            let scraper = WorkUaScraper::new()?;
            scraper.scrape(&config).await?
        }
        SourceId::RobotaUa => {
            let scraper = RobotaUaScraper::new()?;
            scraper.scrape(&config).await?
        }
    };

    if scrape_run.jobs.is_empty() && scrape_run.jobs_attempted == 0 {
        return Err(
            "scraper returned no jobs — site structure may have changed, check selectors"
                .to_string(),
        );
    }

    let batch = IngestionBatch::from_normalization_results(scrape_run.jobs).map_err(|error| {
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

    Ok(ScrapeOutput {
        batch,
        jobs_attempted: scrape_run.jobs_attempted,
        jobs_failed: scrape_run.jobs_failed,
        errors: scrape_run.errors,
    })
}

#[cfg(test)]
mod tests {
    use std::fs;

    use job_copilot_domain::source::SourceId;

    use crate::models::{InputDocument, canonical_job_id, compute_dedupe_key};

    use super::{FileMode, InputFormat, ScrapeSource, load_batch};

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

    #[test]
    fn scrape_source_from_cli_maps_all_known_aliases() {
        assert_eq!(
            ScrapeSource::from_cli("djinni").unwrap().source_id(),
            SourceId::Djinni
        );
        assert_eq!(
            ScrapeSource::from_cli("dou").unwrap().source_id(),
            SourceId::DouUa
        );
        assert_eq!(
            ScrapeSource::from_cli("douua").unwrap().source_id(),
            SourceId::DouUa
        );
        assert_eq!(
            ScrapeSource::from_cli("dou_ua").unwrap().source_id(),
            SourceId::DouUa
        );
        assert_eq!(
            ScrapeSource::from_cli("dou.ua").unwrap().source_id(),
            SourceId::DouUa
        );
        assert_eq!(
            ScrapeSource::from_cli("workua").unwrap().source_id(),
            SourceId::WorkUa
        );
        assert_eq!(
            ScrapeSource::from_cli("work_ua").unwrap().source_id(),
            SourceId::WorkUa
        );
        assert_eq!(
            ScrapeSource::from_cli("work.ua").unwrap().source_id(),
            SourceId::WorkUa
        );
        assert_eq!(
            ScrapeSource::from_cli("robotaua").unwrap().source_id(),
            SourceId::RobotaUa
        );
        assert_eq!(
            ScrapeSource::from_cli("robota_ua").unwrap().source_id(),
            SourceId::RobotaUa
        );
        assert_eq!(
            ScrapeSource::from_cli("robota.ua").unwrap().source_id(),
            SourceId::RobotaUa
        );
    }

    #[test]
    fn scrape_source_from_cli_rejects_unknown_source() {
        assert!(ScrapeSource::from_cli("linkedin").is_err());
    }

    #[test]
    fn scrape_source_name_returns_shared_canonical_key() {
        assert_eq!(ScrapeSource::DJINNI.name(), "djinni");
        assert_eq!(ScrapeSource::DOU_UA.name(), "dou_ua");
        assert_eq!(ScrapeSource::WORK_UA.name(), "work_ua");
        assert_eq!(ScrapeSource::ROBOTA_UA.name(), "robota_ua");
    }
}
