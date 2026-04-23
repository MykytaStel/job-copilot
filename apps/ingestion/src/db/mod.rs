use std::collections::BTreeSet;

use sqlx::PgPool;
use tracing::info;

use crate::models::IngestionBatch;

mod market_role_heuristics {
    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../shared/rust/market_role_heuristics.rs"
    ));
}

mod reconciliation;
mod snapshots;
mod upserts;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UpsertSummary {
    pub jobs_written: usize,
    pub variants_created: usize,
    pub variants_updated: usize,
    pub variants_unchanged: usize,
    pub variants_inactivated: usize,
    pub jobs_inactivated: usize,
    pub jobs_reactivated: usize,
    pub sources_refreshed: usize,
}

impl Default for UpsertSummary {
    fn default() -> Self {
        Self {
            jobs_written: 0,
            variants_created: 0,
            variants_updated: 0,
            variants_unchanged: 0,
            variants_inactivated: 0,
            jobs_inactivated: 0,
            jobs_reactivated: 0,
            sources_refreshed: 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarketSnapshotSummary {
    pub snapshot_date: String,
    pub snapshots_written: usize,
}

pub async fn upsert_batch(pool: &PgPool, batch: &IngestionBatch) -> Result<UpsertSummary, String> {
    batch.validate()?;

    let mut tx = pool
        .begin()
        .await
        .map_err(|error| format!("failed to begin transaction: {error}"))?;
    let resolved_batch = upserts::resolve_batch(&mut tx, batch).await?;
    let mut summary = UpsertSummary::default();
    let mut affected_job_ids = BTreeSet::new();
    let preserve_lifecycle = !resolved_batch.job_variants.is_empty();

    for job in &resolved_batch.jobs {
        upserts::upsert_job(&mut tx, job, preserve_lifecycle).await?;
        affected_job_ids.insert(job.id.clone());
    }

    for variant in &resolved_batch.job_variants {
        match upserts::upsert_job_variant(&mut tx, variant).await? {
            upserts::VariantWriteResult::Created => summary.variants_created += 1,
            upserts::VariantWriteResult::Updated => summary.variants_updated += 1,
            upserts::VariantWriteResult::Unchanged => summary.variants_unchanged += 1,
        }
    }

    let refreshes = reconciliation::build_source_refreshes(&resolved_batch);
    summary.sources_refreshed = refreshes.len();

    for (source, refresh) in refreshes {
        let inactivated = reconciliation::mark_missing_variants_inactive(
            &mut tx,
            &source,
            &refresh.seen_source_job_ids,
            &refresh.refreshed_at,
        )
        .await?;

        summary.variants_inactivated += inactivated.variants_inactivated;
        affected_job_ids.extend(inactivated.job_ids);
    }

    if preserve_lifecycle && !affected_job_ids.is_empty() {
        let affected_job_ids = affected_job_ids.into_iter().collect::<Vec<_>>();
        let reconcile = reconciliation::reconcile_jobs(&mut tx, &affected_job_ids).await?;
        summary.jobs_inactivated = reconcile.jobs_inactivated;
        summary.jobs_reactivated = reconcile.jobs_reactivated;
        reconciliation::create_profile_notifications(&mut tx, &affected_job_ids).await?;
    }

    tx.commit()
        .await
        .map_err(|error| format!("failed to commit transaction: {error}"))?;

    summary.jobs_written = resolved_batch.jobs.len();
    info!(
        jobs_written = summary.jobs_written,
        variants_created = summary.variants_created,
        variants_updated = summary.variants_updated,
        variants_unchanged = summary.variants_unchanged,
        variants_inactivated = summary.variants_inactivated,
        jobs_inactivated = summary.jobs_inactivated,
        jobs_reactivated = summary.jobs_reactivated,
        "upsert batch complete"
    );
    Ok(summary)
}

pub async fn refresh_market_snapshots(pool: &PgPool) -> Result<MarketSnapshotSummary, String> {
    snapshots::run_refresh(pool).await
}

#[cfg(test)]
mod tests {
    use std::env;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use sqlx::PgPool;
    use sqlx::postgres::PgPoolOptions;

    use crate::adapters::SourceAdapter;
    use crate::adapters::mock_source::MockSourceAdapter;
    use crate::models::{
        IngestionBatch, JobVariant, MockSourceInput, NormalizedJob, canonical_job_id,
        compute_dedupe_key,
    };

    use super::reconciliation::{SourceRefresh, build_source_refreshes};
    use super::upserts::merge_job;
    use super::{refresh_market_snapshots, upsert_batch};
    use crate::db_runtime::run_migrations;

    const DEFAULT_TEST_DATABASE_URL: &str =
        "postgres://jobcopilot:jobcopilot@127.0.0.1:5432/jobcopilot";

    fn variant(source_job_id: &str, fetched_at: &str) -> JobVariant {
        JobVariant {
            id: format!("variant_mock_source_{source_job_id}"),
            job_id: format!("job_mock_source_{source_job_id}"),
            dedupe_key: format!(
                "title=platform engineer {source_job_id}|company=signalhire|location=kyiv|remote_type=remote|seniority=senior|posted_on=2026-04-14"
            ),
            source: "mock_source".to_string(),
            source_job_id: source_job_id.to_string(),
            source_url: format!("https://example.com/jobs/{source_job_id}"),
            raw_hash: "abc".repeat(21) + "a",
            raw_payload: serde_json::json!({ "source_job_id": source_job_id }),
            fetched_at: fetched_at.to_string(),
            last_seen_at: fetched_at.to_string(),
            is_active: true,
        }
    }

    #[test]
    fn groups_variant_refreshes_by_source() {
        let batch = crate::models::IngestionBatch {
            jobs: Vec::new(),
            job_variants: vec![
                variant("001", "2026-04-14T10:00:00Z"),
                variant("002", "2026-04-14T09:00:00Z"),
            ],
        };

        let refreshes = build_source_refreshes(&batch);

        assert_eq!(refreshes.len(), 1);
        assert_eq!(
            refreshes.get("mock_source"),
            Some(&SourceRefresh {
                refreshed_at: "2026-04-14T10:00:00Z".to_string(),
                seen_source_job_ids: vec!["001".to_string(), "002".to_string()],
            })
        );
    }

    #[test]
    fn merge_job_keeps_earliest_posted_at_and_latest_last_seen_at() {
        let mut current = NormalizedJob {
            id: "job_1".to_string(),
            title: "Platform Engineer".to_string(),
            company_name: "SignalHire".to_string(),
            location: Some("Kyiv".to_string()),
            remote_type: Some("remote".to_string()),
            seniority: Some("senior".to_string()),
            description_text: "Older".to_string(),
            salary_min: Some(4000),
            salary_max: Some(5000),
            salary_currency: Some("USD".to_string()),
            posted_at: Some("2026-04-15T10:00:00Z".to_string()),
            last_seen_at: "2026-04-15T10:00:00Z".to_string(),
            is_active: false,
        };
        let incoming = NormalizedJob {
            id: "job_1".to_string(),
            title: "Platform Engineer".to_string(),
            company_name: "SignalHire".to_string(),
            location: None,
            remote_type: Some("remote".to_string()),
            seniority: Some("senior".to_string()),
            description_text: "Newer".to_string(),
            salary_min: Some(4200),
            salary_max: Some(5200),
            salary_currency: Some("USD".to_string()),
            posted_at: Some("2026-04-14T08:00:00Z".to_string()),
            last_seen_at: "2026-04-16T09:00:00Z".to_string(),
            is_active: true,
        };

        merge_job(&mut current, &incoming);

        assert_eq!(current.description_text, "Newer");
        assert_eq!(current.posted_at.as_deref(), Some("2026-04-14T08:00:00Z"));
        assert_eq!(current.last_seen_at, "2026-04-16T09:00:00Z");
        assert!(current.is_active);
    }

    #[derive(Debug)]
    struct TestDatabase {
        admin_database_url: String,
        database_name: String,
        pool: PgPool,
    }

    impl TestDatabase {
        async fn try_new() -> Option<Self> {
            let base_database_url = env::var("DATABASE_URL")
                .ok()
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty())
                .unwrap_or_else(|| DEFAULT_TEST_DATABASE_URL.to_string());
            let admin_database_url = match with_database_name(&base_database_url, "postgres") {
                Ok(value) => value,
                Err(error) => {
                    eprintln!("skipping ingestion db integration tests: {error}");
                    return None;
                }
            };
            let database_name = format!(
                "ingestion_test_{}_{}",
                std::process::id(),
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("system time should be after epoch")
                    .as_nanos()
            );

            let admin_pool = match PgPoolOptions::new()
                .max_connections(1)
                .connect(&admin_database_url)
                .await
            {
                Ok(pool) => pool,
                Err(error) => {
                    eprintln!(
                        "skipping ingestion db integration tests: failed to connect to Postgres at '{admin_database_url}': {error}"
                    );
                    return None;
                }
            };

            if let Err(error) = sqlx::query(&format!("CREATE DATABASE \"{database_name}\""))
                .execute(&admin_pool)
                .await
            {
                eprintln!(
                    "skipping ingestion db integration tests: failed to create database '{database_name}': {error}"
                );
                admin_pool.close().await;
                return None;
            }

            admin_pool.close().await;

            let database_url = match with_database_name(&base_database_url, &database_name) {
                Ok(value) => value,
                Err(error) => {
                    eprintln!("skipping ingestion db integration tests: {error}");
                    return None;
                }
            };
            let pool = match PgPoolOptions::new()
                .max_connections(5)
                .connect(&database_url)
                .await
            {
                Ok(pool) => pool,
                Err(error) => {
                    eprintln!(
                        "skipping ingestion db integration tests: failed to connect to test database '{database_name}': {error}"
                    );
                    let _ = cleanup_database(&admin_database_url, &database_name).await;
                    return None;
                }
            };

            if let Err(error) = run_migrations(&pool).await {
                eprintln!(
                    "skipping ingestion db integration tests: failed to run migrations in '{database_name}': {error}"
                );
                pool.close().await;
                let _ = cleanup_database(&admin_database_url, &database_name).await;
                return None;
            }

            Some(Self {
                admin_database_url,
                database_name,
                pool,
            })
        }

        async fn cleanup(self) {
            self.pool.close().await;
            let _ = cleanup_database(&self.admin_database_url, &self.database_name).await;
        }
    }

    #[derive(Debug, sqlx::FromRow)]
    struct VariantState {
        job_id: String,
        dedupe_key: String,
        raw_hash: String,
        last_seen_at: String,
        is_active: bool,
        inactivated_at: Option<String>,
    }

    #[derive(Debug, sqlx::FromRow)]
    struct JobState {
        id: String,
        title: String,
        last_seen_at: String,
        is_active: bool,
        inactivated_at: Option<String>,
        reactivated_at: Option<String>,
    }

    #[derive(Debug, sqlx::FromRow)]
    struct MarketSnapshotRow {
        snapshot_type: String,
        payload: serde_json::Value,
    }

    fn with_database_name(database_url: &str, database_name: &str) -> Result<String, String> {
        let (prefix, query_suffix) = match database_url.split_once('?') {
            Some((prefix, query)) => (prefix, format!("?{query}")),
            None => (database_url, String::new()),
        };
        let slash_index = prefix.rfind('/').ok_or_else(|| {
            format!("database URL '{database_url}' does not contain a database name")
        })?;

        if slash_index + 1 >= prefix.len() {
            return Err(format!(
                "database URL '{database_url}' does not contain a database name"
            ));
        }

        Ok(format!(
            "{}{}{}",
            &prefix[..=slash_index],
            database_name,
            query_suffix
        ))
    }

    async fn cleanup_database(admin_database_url: &str, database_name: &str) -> Result<(), String> {
        let admin_pool = PgPoolOptions::new()
            .max_connections(1)
            .connect(admin_database_url)
            .await
            .map_err(|error| {
                format!("failed to reconnect to admin database '{admin_database_url}': {error}")
            })?;

        sqlx::query(
            r#"
            SELECT pg_terminate_backend(pid)
            FROM pg_stat_activity
            WHERE datname = $1
              AND pid <> pg_backend_pid()
            "#,
        )
        .bind(database_name)
        .execute(&admin_pool)
        .await
        .map_err(|error| {
            format!("failed to terminate connections for '{database_name}': {error}")
        })?;

        sqlx::query(&format!("DROP DATABASE IF EXISTS \"{database_name}\""))
            .execute(&admin_pool)
            .await
            .map_err(|error| format!("failed to drop database '{database_name}': {error}"))?;

        admin_pool.close().await;
        Ok(())
    }

    fn load_mock_source_fixture(name: &str) -> IngestionBatch {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join(name);
        let raw = fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("failed to read fixture '{}': {error}", path.display()));
        let payload = serde_json::from_str::<MockSourceInput>(&raw).unwrap_or_else(|error| {
            panic!("failed to parse fixture '{}': {error}", path.display())
        });
        let adapter = MockSourceAdapter;
        let normalized = adapter.normalize(payload).unwrap_or_else(|error| {
            panic!("failed to normalize fixture '{}': {error}", path.display())
        });

        IngestionBatch::from_normalization_results(normalized).unwrap_or_else(|error| {
            panic!(
                "failed to build batch from fixture '{}': {error}",
                path.display()
            )
        })
    }

    async fn fetch_variant_state(pool: &PgPool, source_job_id: &str) -> VariantState {
        sqlx::query_as::<_, VariantState>(
            r#"
            SELECT
                job_id,
                dedupe_key,
                raw_hash,
                TO_CHAR(last_seen_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS"Z"') AS last_seen_at,
                is_active,
                TO_CHAR(inactivated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS"Z"') AS inactivated_at
            FROM job_variants
            WHERE source = 'mock_source'
              AND source_job_id = $1
            "#,
        )
        .bind(source_job_id)
        .fetch_one(pool)
        .await
        .unwrap_or_else(|error| {
            panic!("failed to fetch variant state for '{source_job_id}': {error}")
        })
    }

    async fn fetch_job_state(pool: &PgPool, job_id: &str) -> JobState {
        sqlx::query_as::<_, JobState>(
            r#"
            SELECT
                id,
                title,
                TO_CHAR(last_seen_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS"Z"') AS last_seen_at,
                is_active,
                TO_CHAR(inactivated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS"Z"') AS inactivated_at,
                TO_CHAR(reactivated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS"Z"') AS reactivated_at
            FROM jobs
            WHERE id = $1
            "#,
        )
        .bind(job_id)
        .fetch_one(pool)
        .await
        .unwrap_or_else(|error| panic!("failed to fetch job state for '{job_id}': {error}"))
    }

    async fn count_active_variants(pool: &PgPool) -> i64 {
        sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COUNT(*)::bigint
            FROM job_variants
            WHERE is_active = TRUE
            "#,
        )
        .fetch_one(pool)
        .await
        .expect("active variant count should query")
    }

    async fn fetch_market_snapshots(pool: &PgPool) -> Vec<MarketSnapshotRow> {
        sqlx::query_as::<_, MarketSnapshotRow>(
            r#"
            SELECT snapshot_type, payload
            FROM market_snapshots
            ORDER BY snapshot_type ASC
            "#,
        )
        .fetch_all(pool)
        .await
        .expect("market snapshots should query")
    }

    #[tokio::test]
    async fn unchanged_rerun_keeps_source_variants_unchanged() {
        let Some(test_db) = TestDatabase::try_new().await else {
            return;
        };

        let initial_batch = load_mock_source_fixture("mock_source_jobs_initial.json");
        let first_summary = upsert_batch(&test_db.pool, &initial_batch)
            .await
            .expect("initial batch should upsert");
        let second_summary = upsert_batch(&test_db.pool, &initial_batch)
            .await
            .expect("unchanged rerun should upsert");

        assert_eq!(first_summary.variants_created, 2);
        assert_eq!(first_summary.variants_updated, 0);
        assert_eq!(first_summary.variants_unchanged, 0);
        assert_eq!(second_summary.variants_created, 0);
        assert_eq!(second_summary.variants_updated, 0);
        assert_eq!(second_summary.variants_unchanged, 2);
        assert_eq!(second_summary.variants_inactivated, 0);
        assert_eq!(second_summary.jobs_inactivated, 0);
        assert_eq!(second_summary.jobs_reactivated, 0);
        assert_eq!(count_active_variants(&test_db.pool).await, 2);

        test_db.cleanup().await;
    }

    #[tokio::test]
    async fn updated_source_payload_updates_variant_without_flipping_lifecycle() {
        let Some(test_db) = TestDatabase::try_new().await else {
            return;
        };

        let initial_batch = load_mock_source_fixture("mock_source_jobs_initial.json");
        upsert_batch(&test_db.pool, &initial_batch)
            .await
            .expect("initial batch should upsert");

        let before = fetch_variant_state(&test_db.pool, "data-002").await;
        let update_batch = load_mock_source_fixture("mock_source_jobs_payload_updated.json");
        let summary = upsert_batch(&test_db.pool, &update_batch)
            .await
            .expect("payload update should upsert");
        let after = fetch_variant_state(&test_db.pool, "data-002").await;

        assert_eq!(summary.variants_created, 0);
        assert_eq!(summary.variants_updated, 1);
        assert_eq!(summary.variants_unchanged, 1);
        assert_eq!(summary.variants_inactivated, 0);
        assert_eq!(summary.jobs_inactivated, 0);
        assert_eq!(summary.jobs_reactivated, 0);
        assert_eq!(before.job_id, after.job_id);
        assert_eq!(before.dedupe_key, after.dedupe_key);
        assert_ne!(before.raw_hash, after.raw_hash);
        assert_eq!(after.last_seen_at, before.last_seen_at);
        assert!(after.is_active);

        test_db.cleanup().await;
    }

    #[tokio::test]
    async fn missing_source_variant_inactivates_canonical_job() {
        let Some(test_db) = TestDatabase::try_new().await else {
            return;
        };

        let initial_batch = load_mock_source_fixture("mock_source_jobs_initial.json");
        upsert_batch(&test_db.pool, &initial_batch)
            .await
            .expect("initial batch should upsert");

        let platform_before = fetch_variant_state(&test_db.pool, "platform-001").await;
        let inactivation_batch = load_mock_source_fixture("mock_source_jobs_inactivated.json");
        let summary = upsert_batch(&test_db.pool, &inactivation_batch)
            .await
            .expect("inactivation batch should upsert");
        let platform_after = fetch_variant_state(&test_db.pool, "platform-001").await;
        let job_after = fetch_job_state(&test_db.pool, &platform_before.job_id).await;

        assert_eq!(summary.variants_created, 0);
        assert_eq!(summary.variants_updated, 1);
        assert_eq!(summary.variants_inactivated, 1);
        assert_eq!(summary.jobs_inactivated, 1);
        assert_eq!(summary.jobs_reactivated, 0);
        assert!(!platform_after.is_active);
        assert_eq!(
            platform_after.inactivated_at.as_deref(),
            Some("2026-04-16T09:00:00Z")
        );
        assert_eq!(job_after.id, platform_before.job_id);
        assert!(!job_after.is_active);
        assert_eq!(job_after.last_seen_at, "2026-04-14T10:00:00Z");
        assert_eq!(
            job_after.inactivated_at.as_deref(),
            Some("2026-04-16T09:00:00Z")
        );
        assert_eq!(job_after.reactivated_at, None);

        test_db.cleanup().await;
    }

    #[tokio::test]
    async fn returning_source_variant_reactivates_canonical_job() {
        let Some(test_db) = TestDatabase::try_new().await else {
            return;
        };

        let initial_batch = load_mock_source_fixture("mock_source_jobs_initial.json");
        upsert_batch(&test_db.pool, &initial_batch)
            .await
            .expect("initial batch should upsert");
        let platform_before = fetch_variant_state(&test_db.pool, "platform-001").await;

        let inactivation_batch = load_mock_source_fixture("mock_source_jobs_inactivated.json");
        upsert_batch(&test_db.pool, &inactivation_batch)
            .await
            .expect("inactivation batch should upsert");

        let reactivation_batch = load_mock_source_fixture("mock_source_jobs_reactivated.json");
        let summary = upsert_batch(&test_db.pool, &reactivation_batch)
            .await
            .expect("reactivation batch should upsert");
        let platform_after = fetch_variant_state(&test_db.pool, "platform-001").await;
        let job_after = fetch_job_state(&test_db.pool, &platform_before.job_id).await;

        assert_eq!(summary.variants_created, 0);
        assert_eq!(summary.variants_updated, 2);
        assert_eq!(summary.variants_inactivated, 0);
        assert_eq!(summary.jobs_inactivated, 0);
        assert_eq!(summary.jobs_reactivated, 1);
        assert!(platform_after.is_active);
        assert_eq!(platform_after.inactivated_at, None);
        assert_eq!(platform_after.last_seen_at, "2026-04-17T09:00:00Z");
        assert_eq!(job_after.id, platform_before.job_id);
        assert!(job_after.is_active);
        assert_eq!(job_after.title, "Senior Platform Ingestion Engineer");
        assert_eq!(job_after.last_seen_at, "2026-04-17T09:00:00Z");
        assert_eq!(job_after.inactivated_at, None);
        assert_eq!(
            job_after.reactivated_at.as_deref(),
            Some("2026-04-17T09:00:00Z")
        );

        test_db.cleanup().await;
    }

    #[tokio::test]
    async fn stale_source_snapshot_does_not_reactivate_newer_inactivation() {
        let Some(test_db) = TestDatabase::try_new().await else {
            return;
        };

        let initial_batch = load_mock_source_fixture("mock_source_jobs_initial.json");
        upsert_batch(&test_db.pool, &initial_batch)
            .await
            .expect("initial batch should upsert");
        let platform_before = fetch_variant_state(&test_db.pool, "platform-001").await;

        let inactivation_batch = load_mock_source_fixture("mock_source_jobs_inactivated.json");
        upsert_batch(&test_db.pool, &inactivation_batch)
            .await
            .expect("inactivation batch should upsert");

        let stale_summary = upsert_batch(&test_db.pool, &initial_batch)
            .await
            .expect("stale batch should be ignored safely");
        let platform_after = fetch_variant_state(&test_db.pool, "platform-001").await;
        let job_after = fetch_job_state(&test_db.pool, &platform_before.job_id).await;

        assert_eq!(stale_summary.jobs_written, 0);
        assert_eq!(stale_summary.variants_created, 0);
        assert_eq!(stale_summary.variants_updated, 0);
        assert_eq!(stale_summary.variants_unchanged, 0);
        assert_eq!(stale_summary.variants_inactivated, 0);
        assert_eq!(stale_summary.jobs_inactivated, 0);
        assert_eq!(stale_summary.jobs_reactivated, 0);
        assert!(!platform_after.is_active);
        assert_eq!(
            platform_after.inactivated_at.as_deref(),
            Some("2026-04-16T09:00:00Z")
        );
        assert_eq!(job_after.id, platform_before.job_id);
        assert!(!job_after.is_active);
        assert_eq!(job_after.reactivated_at, None);

        test_db.cleanup().await;
    }

    #[tokio::test]
    async fn dedupe_collision_on_source_variant_update_fails_fast() {
        let Some(test_db) = TestDatabase::try_new().await else {
            return;
        };

        let initial_batch = load_mock_source_fixture("mock_source_jobs_initial.json");
        upsert_batch(&test_db.pool, &initial_batch)
            .await
            .expect("initial batch should upsert");

        let mut conflicting_batch = load_mock_source_fixture("mock_source_jobs_initial.json");
        let target_job = conflicting_batch.jobs[1].clone();
        let conflicting_job = &mut conflicting_batch.jobs[0];
        conflicting_job.title = target_job.title.clone();
        conflicting_job.company_name = target_job.company_name.clone();
        conflicting_job.location = target_job.location.clone();
        conflicting_job.remote_type = target_job.remote_type.clone();
        conflicting_job.seniority = target_job.seniority.clone();
        conflicting_job.posted_at = target_job.posted_at.clone();
        conflicting_job.id = canonical_job_id(&compute_dedupe_key(conflicting_job));

        let conflicting_variant = &mut conflicting_batch.job_variants[0];
        conflicting_variant.dedupe_key = compute_dedupe_key(conflicting_job);
        conflicting_variant.job_id = conflicting_job.id.clone();

        let error = upsert_batch(&test_db.pool, &conflicting_batch)
            .await
            .expect_err("dedupe collision should be rejected");

        assert!(error.contains("changed dedupe fingerprint"));
        assert!(error.contains("already belongs to canonical job"));

        test_db.cleanup().await;
    }

    #[tokio::test]
    async fn refresh_market_snapshots_upserts_one_snapshot_per_type_for_the_day() {
        let Some(test_db) = TestDatabase::try_new().await else {
            return;
        };

        let initial_batch = load_mock_source_fixture("mock_source_jobs_initial.json");
        upsert_batch(&test_db.pool, &initial_batch)
            .await
            .expect("initial batch should upsert");

        let first_summary = refresh_market_snapshots(&test_db.pool)
            .await
            .expect("market snapshots should refresh");
        let second_summary = refresh_market_snapshots(&test_db.pool)
            .await
            .expect("market snapshots should refresh idempotently");

        assert_eq!(first_summary.snapshots_written, 4);
        assert_eq!(first_summary.snapshot_date, second_summary.snapshot_date);
        assert_eq!(second_summary.snapshots_written, 4);

        let snapshots = fetch_market_snapshots(&test_db.pool).await;
        assert_eq!(snapshots.len(), 4);

        let by_type = snapshots
            .into_iter()
            .map(|snapshot| (snapshot.snapshot_type, snapshot.payload))
            .collect::<std::collections::BTreeMap<_, _>>();

        assert_eq!(
            by_type
                .get("overview")
                .and_then(|payload| payload.get("active_jobs_count"))
                .and_then(|value| value.as_i64()),
            Some(2)
        );
        assert!(
            by_type
                .get("company_stats")
                .and_then(|payload| payload.as_array())
                .is_some_and(|items| items.len() >= 2),
            "company stats snapshot should contain active companies"
        );
        assert!(
            by_type
                .get("salary_trends")
                .and_then(|payload| payload.as_array())
                .is_some_and(|items| !items.is_empty()),
            "salary trends snapshot should contain salary buckets"
        );
        assert_eq!(
            by_type
                .get("role_demand")
                .and_then(|payload| payload.as_array())
                .map(|items| items.len()),
            Some(8)
        );

        test_db.cleanup().await;
    }
}
