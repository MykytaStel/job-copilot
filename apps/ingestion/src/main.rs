mod adapters;
mod db;
mod models;

use std::env;
use std::fs;
use std::path::PathBuf;

use tracing::info;

use crate::adapters::SourceAdapter;
use crate::adapters::mock_source::MockSourceAdapter;
use crate::models::{IngestionBatch, InputDocument, MockSourceInput};

struct Config {
    database_url: String,
    input_path: PathBuf,
    input_format: InputFormat,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum InputFormat {
    Normalized,
    MockSource,
}

#[tokio::main]
async fn main() -> Result<(), String> {
    tracing_subscriber::fmt().with_env_filter("info").init();

    let config = Config::from_env()?;
    let batch = load_batch(&config)?;

    if batch.jobs.is_empty() {
        return Err("input file does not contain any jobs".to_string());
    }

    let pool = db::connect(&config.database_url).await?;
    let summary = db::upsert_batch(&pool, &batch).await?;

    info!(
        jobs_written = summary.jobs_written,
        variants_created = summary.variants_created,
        variants_updated = summary.variants_updated,
        variants_unchanged = summary.variants_unchanged,
        input = %config.input_path.display(),
        input_format = ?config.input_format,
        "ingestion completed"
    );

    println!(
        "Wrote {} jobs; variants created: {}, updated: {}, unchanged: {}",
        summary.jobs_written,
        summary.variants_created,
        summary.variants_updated,
        summary.variants_unchanged
    );
    Ok(())
}

impl Config {
    fn from_env() -> Result<Self, String> {
        let database_url = env::var("DATABASE_URL")
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .ok_or_else(|| "DATABASE_URL is required".to_string())?;

        let mut args = env::args().skip(1);
        let Some(flag) = args.next() else {
            return Err(
                "usage: cargo run -- --input <path-to-jobs.json> [--input-format normalized|mock-source]"
                    .to_string(),
            );
        };

        if flag != "--input" {
            return Err(format!("unsupported argument '{flag}', expected '--input'"));
        }

        let Some(path) = args.next() else {
            return Err("missing value for --input".to_string());
        };

        let mut input_format = InputFormat::Normalized;
        while let Some(flag) = args.next() {
            if flag != "--input-format" {
                return Err(format!(
                    "unsupported argument '{flag}', expected '--input-format'"
                ));
            }

            let Some(value) = args.next() else {
                return Err("missing value for --input-format".to_string());
            };

            input_format = InputFormat::from_cli(&value)?;
        }

        Ok(Self {
            database_url,
            input_path: PathBuf::from(path),
            input_format,
        })
    }
}

impl InputFormat {
    fn from_cli(value: &str) -> Result<Self, String> {
        match value.trim() {
            "normalized" => Ok(Self::Normalized),
            "mock-source" => Ok(Self::MockSource),
            other => Err(format!(
                "unsupported input format '{other}', expected 'normalized' or 'mock-source'"
            )),
        }
    }
}

fn load_batch(config: &Config) -> Result<IngestionBatch, String> {
    let raw = read_input_file(&config.input_path)?;

    match config.input_format {
        InputFormat::Normalized => {
            let payload = serde_json::from_str::<InputDocument>(&raw).map_err(|error| {
                format!("failed to parse {}: {error}", config.input_path.display())
            })?;
            Ok(IngestionBatch::from_jobs(payload.into_jobs()))
        }
        InputFormat::MockSource => {
            let payload = serde_json::from_str::<MockSourceInput>(&raw).map_err(|error| {
                format!("failed to parse {}: {error}", config.input_path.display())
            })?;
            let adapter = MockSourceAdapter;
            let normalized = adapter.normalize(payload)?;
            IngestionBatch::from_normalization_results(normalized)
        }
    }
}

fn read_input_file(path: &PathBuf) -> Result<String, String> {
    fs::read_to_string(path).map_err(|error| format!("failed to read {}: {error}", path.display()))
}

#[cfg(test)]
mod tests {
    use std::fs;

    use crate::models::InputDocument;

    use super::{Config, InputFormat, load_batch, read_input_file};

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
        let error = read_input_file(&"/definitely/missing.json".into())
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

        let batch = load_batch(&Config {
            database_url: "postgres://unused".to_string(),
            input_path: path.clone(),
            input_format: InputFormat::MockSource,
        })
        .expect("mock source jobs should load");

        fs::remove_file(path).ok();

        assert_eq!(batch.jobs.len(), 1);
        assert_eq!(batch.jobs[0].id, "job_mock_source_777");
        assert_eq!(batch.jobs[0].company_name, "SignalHire");
        assert_eq!(batch.job_variants.len(), 1);
        assert_eq!(batch.job_variants[0].job_id, "job_mock_source_777");
        assert_eq!(batch.job_variants[0].source, "mock_source");
    }
}
