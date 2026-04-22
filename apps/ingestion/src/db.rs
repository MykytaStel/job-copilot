use std::collections::{BTreeMap, BTreeSet};

use chrono::Utc;
use sqlx::PgPool;
use sqlx::types::Json;
use tracing::{info, warn};

use crate::models::{IngestionBatch, JobVariant, NormalizedJob};

mod market_role_heuristics {
    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../shared/rust/market_role_heuristics.rs"
    ));
}

use market_role_heuristics::{
    MARKET_ROLE_GROUP_CLASSIFIER_CASE_SQL, MARKET_ROLE_GROUP_ORDER_ARRAY_SQL,
    MARKET_ROLE_GROUPS_VALUES_SQL,
};

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarketSnapshotSummary {
    pub snapshot_date: String,
    pub snapshots_written: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SourceRefresh {
    refreshed_at: String,
    seen_source_job_ids: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ReconcileSummary {
    jobs_inactivated: usize,
    jobs_reactivated: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, sqlx::FromRow)]
struct ExistingVariantState {
    job_id: String,
    dedupe_key: String,
    raw_hash: String,
    last_seen_at: String,
    fetched_at: String,
    is_active: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum VariantWriteResult {
    Created,
    Updated,
    Unchanged,
}

pub async fn upsert_batch(pool: &PgPool, batch: &IngestionBatch) -> Result<UpsertSummary, String> {
    batch.validate()?;

    let mut tx = pool
        .begin()
        .await
        .map_err(|error| format!("failed to begin transaction: {error}"))?;
    let resolved_batch = resolve_batch(&mut tx, batch).await?;
    let mut summary = UpsertSummary::default();
    let mut affected_job_ids = BTreeSet::new();
    let preserve_lifecycle = !resolved_batch.job_variants.is_empty();

    for job in &resolved_batch.jobs {
        upsert_job(&mut tx, job, preserve_lifecycle).await?;
        affected_job_ids.insert(job.id.clone());
    }

    for variant in &resolved_batch.job_variants {
        match upsert_job_variant(&mut tx, variant).await? {
            VariantWriteResult::Created => summary.variants_created += 1,
            VariantWriteResult::Updated => summary.variants_updated += 1,
            VariantWriteResult::Unchanged => summary.variants_unchanged += 1,
        }
    }

    let refreshes = build_source_refreshes(&resolved_batch);
    summary.sources_refreshed = refreshes.len();

    for (source, refresh) in refreshes {
        let inactivated = mark_missing_variants_inactive(
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
        let reconcile = reconcile_jobs(&mut tx, &affected_job_ids).await?;
        summary.jobs_inactivated = reconcile.jobs_inactivated;
        summary.jobs_reactivated = reconcile.jobs_reactivated;
        create_profile_notifications(&mut tx, &affected_job_ids).await?;
    }

    tx.commit()
        .await
        .map_err(|error| format!("failed to commit transaction: {error}"))?;

    summary.jobs_written = resolved_batch.jobs.len();
    Ok(summary)
}

pub async fn refresh_market_snapshots(pool: &PgPool) -> Result<MarketSnapshotSummary, String> {
    let snapshot_date = Utc::now().date_naive();
    let overview_payload = build_market_overview_snapshot(pool).await?;
    let company_stats_payload = build_market_company_stats_snapshot(pool).await?;
    let salary_trends_payload = build_market_salary_trends_snapshot(pool).await?;
    let role_demand_payload = build_market_role_demand_snapshot(pool).await?;

    upsert_market_snapshot(pool, snapshot_date, "overview", overview_payload).await?;
    upsert_market_snapshot(pool, snapshot_date, "company_stats", company_stats_payload).await?;
    upsert_market_snapshot(pool, snapshot_date, "salary_trends", salary_trends_payload).await?;
    upsert_market_snapshot(pool, snapshot_date, "role_demand", role_demand_payload).await?;

    Ok(MarketSnapshotSummary {
        snapshot_date: snapshot_date.format("%Y-%m-%d").to_string(),
        snapshots_written: 4,
    })
}

async fn resolve_batch(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    batch: &IngestionBatch,
) -> Result<IngestionBatch, String> {
    batch.validate()?;

    if batch.job_variants.is_empty() {
        return Ok(batch.clone());
    }

    let mut jobs_by_id = BTreeMap::<String, NormalizedJob>::new();
    let mut variants = Vec::with_capacity(batch.job_variants.len());
    let mut dedupe_job_ids = BTreeMap::<String, String>::new();
    let mut stale_variants_skipped = 0usize;

    for (job, variant) in batch.jobs.iter().zip(batch.job_variants.iter()) {
        let existing_variant =
            existing_variant_state(tx, &variant.source, &variant.source_job_id).await?;

        if should_skip_stale_variant(existing_variant.as_ref(), variant) {
            stale_variants_skipped += 1;
            warn!(
                source = %variant.source,
                source_job_id = %variant.source_job_id,
                incoming_last_seen_at = %variant.last_seen_at,
                incoming_fetched_at = %variant.fetched_at,
                existing_last_seen_at = existing_variant
                    .as_ref()
                    .map(|state| state.last_seen_at.as_str())
                    .unwrap_or(""),
                existing_fetched_at = existing_variant
                    .as_ref()
                    .map(|state| state.fetched_at.as_str())
                    .unwrap_or(""),
                incoming_is_active = variant.is_active,
                existing_is_active = existing_variant.as_ref().map(|state| state.is_active),
                "skipping stale source variant update"
            );
            continue;
        }

        if let Some(existing_variant) = existing_variant.as_ref() {
            validate_dedupe_transition(tx, existing_variant, variant).await?;
        }

        let resolved_job_id = if let Some(job_id) = dedupe_job_ids.get(&variant.dedupe_key) {
            job_id.clone()
        } else if let Some(existing_variant) = existing_variant.as_ref() {
            existing_variant.job_id.clone()
        } else if let Some(job_id) = existing_job_id_for_dedupe_key(tx, &variant.dedupe_key).await?
        {
            job_id
        } else {
            variant.job_id.clone()
        };

        dedupe_job_ids.insert(variant.dedupe_key.clone(), resolved_job_id.clone());

        let mut resolved_job = job.clone();
        resolved_job.id = resolved_job_id.clone();
        jobs_by_id
            .entry(resolved_job_id.clone())
            .and_modify(|current| merge_job(current, &resolved_job))
            .or_insert(resolved_job);

        let mut resolved_variant = variant.clone();
        resolved_variant.job_id = resolved_job_id;
        variants.push(resolved_variant);
    }

    if stale_variants_skipped > 0 {
        info!(
            stale_variants_skipped,
            "adapter-backed batch skipped stale source variants"
        );
    }

    Ok(IngestionBatch {
        jobs: jobs_by_id.into_values().collect(),
        job_variants: variants,
    })
}

async fn build_market_overview_snapshot(pool: &PgPool) -> Result<serde_json::Value, String> {
    sqlx::query_scalar::<_, serde_json::Value>(
        r#"
        SELECT jsonb_build_object(
            'new_jobs_this_week',
            COUNT(*) FILTER (
                WHERE is_active AND first_seen_at >= NOW() - INTERVAL '7 days'
            )::bigint,
            'active_companies_count',
            COUNT(DISTINCT company_name) FILTER (WHERE is_active)::bigint,
            'active_jobs_count',
            COUNT(*) FILTER (WHERE is_active)::bigint,
            'remote_percentage',
            CASE
                WHEN COUNT(*) FILTER (WHERE is_active) > 0
                THEN ROUND(
                    (
                        COUNT(*) FILTER (
                            WHERE is_active AND LOWER(remote_type) LIKE '%remote%'
                        )::numeric
                        / COUNT(*) FILTER (WHERE is_active)::numeric
                    ) * 100,
                    2
                )
                ELSE 0
            END
        )
        FROM jobs
        "#,
    )
    .fetch_one(pool)
    .await
    .map_err(|error| format!("failed to build market overview snapshot: {error}"))
}

async fn build_market_company_stats_snapshot(pool: &PgPool) -> Result<serde_json::Value, String> {
    sqlx::query_scalar::<_, serde_json::Value>(
        r#"
        WITH company_stats AS (
            SELECT
                company_name,
                COUNT(*) FILTER (WHERE is_active)::bigint AS active_jobs,
                COUNT(*) FILTER (
                    WHERE is_active AND first_seen_at >= NOW() - INTERVAL '7 days'
                )::bigint AS this_week,
                COUNT(*) FILTER (
                    WHERE is_active
                      AND first_seen_at >= NOW() - INTERVAL '14 days'
                      AND first_seen_at < NOW() - INTERVAL '7 days'
                )::bigint AS prev_week
            FROM jobs
            GROUP BY company_name
            HAVING COUNT(*) FILTER (WHERE is_active) > 0
        )
        SELECT COALESCE(
            jsonb_agg(
                jsonb_build_object(
                    'company_name', company_name,
                    'active_jobs', active_jobs,
                    'this_week', this_week,
                    'prev_week', prev_week
                )
                ORDER BY active_jobs DESC, company_name ASC
            ),
            '[]'::jsonb
        )
        FROM company_stats
        "#,
    )
    .fetch_one(pool)
    .await
    .map_err(|error| format!("failed to build market company stats snapshot: {error}"))
}

async fn build_market_salary_trends_snapshot(pool: &PgPool) -> Result<serde_json::Value, String> {
    sqlx::query_scalar::<_, serde_json::Value>(
        r#"
        WITH filtered_jobs AS (
            SELECT
                LOWER(TRIM(seniority)) AS seniority,
                COALESCE(NULLIF(UPPER(TRIM(salary_currency)), ''), 'UNKNOWN') AS salary_currency,
                salary_min
            FROM jobs
            WHERE is_active
              AND salary_min IS NOT NULL
              AND seniority IS NOT NULL
              AND last_seen_at >= NOW() - INTERVAL '30 days'
        ),
        ranked_currencies AS (
            SELECT
                seniority,
                salary_currency,
                ROW_NUMBER() OVER (
                    PARTITION BY seniority
                    ORDER BY COUNT(*) DESC, salary_currency ASC
                ) AS currency_rank
            FROM filtered_jobs
            GROUP BY seniority, salary_currency
        ),
        dominant_jobs AS (
            SELECT filtered_jobs.*
            FROM filtered_jobs
            INNER JOIN ranked_currencies
                ON ranked_currencies.seniority = filtered_jobs.seniority
               AND ranked_currencies.salary_currency = filtered_jobs.salary_currency
            WHERE ranked_currencies.currency_rank = 1
        ),
        salary_trends AS (
            SELECT
                seniority,
                ROUND(PERCENTILE_CONT(0.25) WITHIN GROUP (ORDER BY salary_min))::integer AS p25,
                ROUND(PERCENTILE_CONT(0.50) WITHIN GROUP (ORDER BY salary_min))::integer AS median,
                ROUND(PERCENTILE_CONT(0.75) WITHIN GROUP (ORDER BY salary_min))::integer AS p75,
                COUNT(*)::bigint AS sample_count
            FROM dominant_jobs
            GROUP BY seniority
        )
        SELECT COALESCE(
            jsonb_agg(
                jsonb_build_object(
                    'seniority', seniority,
                    'p25', p25,
                    'median', median,
                    'p75', p75,
                    'sample_count', sample_count
                )
                ORDER BY
                    CASE seniority
                        WHEN 'intern' THEN 0
                        WHEN 'junior' THEN 1
                        WHEN 'middle' THEN 2
                        WHEN 'mid' THEN 2
                        WHEN 'senior' THEN 3
                        WHEN 'lead' THEN 4
                        ELSE 5
                    END,
                    seniority ASC
            ),
            '[]'::jsonb
        )
        FROM salary_trends
        "#,
    )
    .fetch_one(pool)
    .await
    .map_err(|error| format!("failed to build market salary trends snapshot: {error}"))
}

async fn build_market_role_demand_snapshot(pool: &PgPool) -> Result<serde_json::Value, String> {
    let query = format!(
        r#"
        WITH role_groups(role_group) AS (
            VALUES
                {role_groups_values}
        ),
        classified_jobs AS (
            SELECT
                {role_group_classifier} AS role_group,
                first_seen_at
            FROM jobs
            WHERE is_active
        ),
        counts AS (
            SELECT
                role_group,
                COUNT(*) FILTER (
                    WHERE first_seen_at >= NOW() - INTERVAL '30 days'
                )::bigint AS this_period,
                COUNT(*) FILTER (
                    WHERE first_seen_at >= NOW() - INTERVAL '60 days'
                      AND first_seen_at < NOW() - INTERVAL '30 days'
                )::bigint AS prev_period
            FROM classified_jobs
            WHERE role_group IS NOT NULL
            GROUP BY role_group
        )
        SELECT COALESCE(
            jsonb_agg(
                jsonb_build_object(
                    'role_group', role_groups.role_group,
                    'this_period', COALESCE(counts.this_period, 0)::bigint,
                    'prev_period', COALESCE(counts.prev_period, 0)::bigint,
                    'trend',
                    CASE
                        WHEN COALESCE(counts.this_period, 0) > COALESCE(counts.prev_period, 0) THEN 'up'
                        WHEN COALESCE(counts.this_period, 0) < COALESCE(counts.prev_period, 0) THEN 'down'
                        ELSE 'stable'
                    END
                )
                ORDER BY ARRAY_POSITION(
                    {role_group_order},
                    role_groups.role_group
                )
            ),
            '[]'::jsonb
        )
        FROM role_groups
        LEFT JOIN counts USING (role_group)
        "#,
        role_groups_values = MARKET_ROLE_GROUPS_VALUES_SQL,
        role_group_classifier = MARKET_ROLE_GROUP_CLASSIFIER_CASE_SQL,
        role_group_order = MARKET_ROLE_GROUP_ORDER_ARRAY_SQL,
    );
    sqlx::query_scalar::<_, serde_json::Value>(&query)
        .fetch_one(pool)
        .await
        .map_err(|error| format!("failed to build market role demand snapshot: {error}"))
}

async fn upsert_market_snapshot(
    pool: &PgPool,
    snapshot_date: chrono::NaiveDate,
    snapshot_type: &str,
    payload: serde_json::Value,
) -> Result<(), String> {
    let snapshot_date_string = snapshot_date.format("%Y-%m-%d").to_string();
    let snapshot_id = format!("market_snapshot_{}_{}", snapshot_type, snapshot_date_string);

    sqlx::query(
        r#"
        INSERT INTO market_snapshots (id, snapshot_date, snapshot_type, payload)
        VALUES ($1, $2::date, $3, $4)
        ON CONFLICT (id)
        DO UPDATE SET
            payload = EXCLUDED.payload,
            created_at = NOW()
        "#,
    )
    .bind(snapshot_id)
    .bind(snapshot_date_string)
    .bind(snapshot_type)
    .bind(Json(payload))
    .execute(pool)
    .await
    .map_err(|error| {
        format!("failed to upsert market snapshot '{snapshot_type}' for '{snapshot_date}': {error}")
    })?;

    Ok(())
}

async fn existing_variant_state(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    source: &str,
    source_job_id: &str,
) -> Result<Option<ExistingVariantState>, String> {
    sqlx::query_as::<_, ExistingVariantState>(
        r#"
        SELECT
            job_id,
            dedupe_key,
            raw_hash,
            TO_CHAR(last_seen_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS"Z"') AS last_seen_at,
            TO_CHAR(fetched_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS"Z"') AS fetched_at,
            is_active
        FROM job_variants
        WHERE source = $1 AND source_job_id = $2
        LIMIT 1
        "#,
    )
    .bind(source)
    .bind(source_job_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(|error| {
        format!("failed to resolve existing variant '{source}:{source_job_id}': {error}")
    })
}

fn should_skip_stale_variant(
    existing_variant: Option<&ExistingVariantState>,
    incoming_variant: &JobVariant,
) -> bool {
    let Some(existing_variant) = existing_variant else {
        return false;
    };

    let incoming_last_seen = chrono::DateTime::parse_from_rfc3339(&incoming_variant.last_seen_at);
    let existing_last_seen = chrono::DateTime::parse_from_rfc3339(&existing_variant.last_seen_at);
    let incoming_fetched_at = chrono::DateTime::parse_from_rfc3339(&incoming_variant.fetched_at);
    let existing_fetched_at = chrono::DateTime::parse_from_rfc3339(&existing_variant.fetched_at);

    match (
        incoming_last_seen,
        existing_last_seen,
        incoming_fetched_at,
        existing_fetched_at,
    ) {
        (
            Ok(incoming_last_seen),
            Ok(existing_last_seen),
            Ok(incoming_fetched_at),
            Ok(existing_fetched_at),
        ) => {
            if incoming_last_seen < existing_last_seen {
                return true;
            }

            incoming_last_seen == existing_last_seen
                && incoming_fetched_at <= existing_fetched_at
                && incoming_variant.dedupe_key == existing_variant.dedupe_key
                && (incoming_variant.is_active != existing_variant.is_active
                    || incoming_variant.raw_hash != existing_variant.raw_hash)
        }
        _ => incoming_variant.last_seen_at < existing_variant.last_seen_at,
    }
}

async fn validate_dedupe_transition(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    existing_variant: &ExistingVariantState,
    incoming_variant: &JobVariant,
) -> Result<(), String> {
    if existing_variant.dedupe_key == incoming_variant.dedupe_key {
        return Ok(());
    }

    if let Some(conflict_job_id) =
        existing_job_id_for_dedupe_key(tx, &incoming_variant.dedupe_key).await?
    {
        if conflict_job_id != existing_variant.job_id {
            return Err(format!(
                "source variant '{}:{}' changed dedupe fingerprint from '{}' to '{}' but the new fingerprint already belongs to canonical job '{}' instead of '{}'",
                incoming_variant.source,
                incoming_variant.source_job_id,
                existing_variant.dedupe_key,
                incoming_variant.dedupe_key,
                conflict_job_id,
                existing_variant.job_id
            ));
        }
    }

    warn!(
        source = %incoming_variant.source,
        source_job_id = %incoming_variant.source_job_id,
        job_id = %existing_variant.job_id,
        previous_dedupe_key = %existing_variant.dedupe_key,
        incoming_dedupe_key = %incoming_variant.dedupe_key,
        "source variant dedupe fingerprint changed"
    );

    Ok(())
}

async fn existing_job_id_for_dedupe_key(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    dedupe_key: &str,
) -> Result<Option<String>, String> {
    sqlx::query_scalar::<_, String>(
        r#"
        SELECT job_id
        FROM job_variants
        WHERE dedupe_key = $1
        ORDER BY is_active DESC, last_seen_at DESC, fetched_at DESC
        LIMIT 1
        "#,
    )
    .bind(dedupe_key)
    .fetch_optional(&mut **tx)
    .await
    .map_err(|error| format!("failed to resolve dedupe key '{dedupe_key}': {error}"))
}

fn merge_job(current: &mut NormalizedJob, incoming: &NormalizedJob) {
    if incoming.last_seen_at >= current.last_seen_at {
        current.title = incoming.title.clone();
        current.company_name = incoming.company_name.clone();
        current.description_text = incoming.description_text.clone();
        current.location = pick_optional(&incoming.location, &current.location);
        current.remote_type = pick_optional(&incoming.remote_type, &current.remote_type);
        current.seniority = pick_optional(&incoming.seniority, &current.seniority);
        current.salary_min = incoming.salary_min.or(current.salary_min);
        current.salary_max = incoming.salary_max.or(current.salary_max);
        current.salary_currency =
            pick_optional(&incoming.salary_currency, &current.salary_currency);
    } else {
        current.location = pick_optional(&current.location, &incoming.location);
        current.remote_type = pick_optional(&current.remote_type, &incoming.remote_type);
        current.seniority = pick_optional(&current.seniority, &incoming.seniority);
        current.salary_min = current.salary_min.or(incoming.salary_min);
        current.salary_max = current.salary_max.or(incoming.salary_max);
        current.salary_currency =
            pick_optional(&current.salary_currency, &incoming.salary_currency);
    }

    current.posted_at = earliest_timestamp(current.posted_at.as_ref(), incoming.posted_at.as_ref());
    current.last_seen_at = current
        .last_seen_at
        .clone()
        .max(incoming.last_seen_at.clone());
    current.is_active = current.is_active || incoming.is_active;
}

fn pick_optional(primary: &Option<String>, fallback: &Option<String>) -> Option<String> {
    primary.clone().or_else(|| fallback.clone())
}

fn earliest_timestamp(left: Option<&String>, right: Option<&String>) -> Option<String> {
    match (left, right) {
        (Some(left), Some(right)) => Some(left.clone().min(right.clone())),
        (Some(left), None) => Some(left.clone()),
        (None, Some(right)) => Some(right.clone()),
        (None, None) => None,
    }
}

async fn upsert_job(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    job: &NormalizedJob,
    preserve_lifecycle: bool,
) -> Result<(), String> {
    let query = if preserve_lifecycle {
        r#"
        INSERT INTO jobs (
            id,
            title,
            company_name,
            location,
            remote_type,
            seniority,
            description_text,
            salary_min,
            salary_max,
            salary_currency,
            posted_at,
            first_seen_at,
            last_seen_at,
            is_active,
            inactivated_at,
            reactivated_at
        )
        VALUES (
            $1,
            $2,
            $3,
            $4,
            $5,
            $6,
            $7,
            $8,
            $9,
            $10,
            $11::timestamptz,
            COALESCE($11::timestamptz, $12::timestamptz),
            $12::timestamptz,
            $13,
            CASE
                WHEN $13 THEN NULL
                ELSE $12::timestamptz
            END,
            NULL
        )
        ON CONFLICT (id)
        DO UPDATE SET
            title = EXCLUDED.title,
            company_name = EXCLUDED.company_name,
            location = EXCLUDED.location,
            remote_type = EXCLUDED.remote_type,
            seniority = EXCLUDED.seniority,
            description_text = EXCLUDED.description_text,
            salary_min = EXCLUDED.salary_min,
            salary_max = EXCLUDED.salary_max,
            salary_currency = EXCLUDED.salary_currency,
            posted_at = EXCLUDED.posted_at,
            first_seen_at = LEAST(jobs.first_seen_at, EXCLUDED.first_seen_at),
            last_seen_at = GREATEST(jobs.last_seen_at, EXCLUDED.last_seen_at)
        "#
    } else {
        r#"
        INSERT INTO jobs (
            id,
            title,
            company_name,
            location,
            remote_type,
            seniority,
            description_text,
            salary_min,
            salary_max,
            salary_currency,
            posted_at,
            first_seen_at,
            last_seen_at,
            is_active,
            inactivated_at,
            reactivated_at
        )
        VALUES (
            $1,
            $2,
            $3,
            $4,
            $5,
            $6,
            $7,
            $8,
            $9,
            $10,
            $11::timestamptz,
            COALESCE($11::timestamptz, $12::timestamptz),
            $12::timestamptz,
            $13,
            CASE
                WHEN $13 THEN NULL
                ELSE $12::timestamptz
            END,
            NULL
        )
        ON CONFLICT (id)
        DO UPDATE SET
            title = EXCLUDED.title,
            company_name = EXCLUDED.company_name,
            location = EXCLUDED.location,
            remote_type = EXCLUDED.remote_type,
            seniority = EXCLUDED.seniority,
            description_text = EXCLUDED.description_text,
            salary_min = EXCLUDED.salary_min,
            salary_max = EXCLUDED.salary_max,
            salary_currency = EXCLUDED.salary_currency,
            posted_at = EXCLUDED.posted_at,
            first_seen_at = LEAST(jobs.first_seen_at, EXCLUDED.first_seen_at),
            last_seen_at = GREATEST(jobs.last_seen_at, EXCLUDED.last_seen_at),
            is_active = CASE
                WHEN EXCLUDED.last_seen_at < jobs.last_seen_at THEN jobs.is_active
                ELSE EXCLUDED.is_active
            END,
            inactivated_at = CASE
                WHEN EXCLUDED.last_seen_at < jobs.last_seen_at THEN jobs.inactivated_at
                WHEN EXCLUDED.is_active THEN NULL
                ELSE COALESCE(jobs.inactivated_at, EXCLUDED.last_seen_at)
            END,
            reactivated_at = CASE
                WHEN EXCLUDED.last_seen_at < jobs.last_seen_at THEN jobs.reactivated_at
                WHEN NOT jobs.is_active AND EXCLUDED.is_active THEN EXCLUDED.last_seen_at
                ELSE jobs.reactivated_at
            END
        "#
    };

    sqlx::query(query)
        .bind(&job.id)
        .bind(&job.title)
        .bind(&job.company_name)
        .bind(&job.location)
        .bind(&job.remote_type)
        .bind(&job.seniority)
        .bind(&job.description_text)
        .bind(job.salary_min)
        .bind(job.salary_max)
        .bind(&job.salary_currency)
        .bind(&job.posted_at)
        .bind(&job.last_seen_at)
        .bind(job.is_active)
        .execute(&mut **tx)
        .await
        .map_err(|error| format!("failed to upsert job '{}': {error}", job.id))?;

    Ok(())
}

async fn upsert_job_variant(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    variant: &JobVariant,
) -> Result<VariantWriteResult, String> {
    let outcome = sqlx::query_scalar::<_, String>(
        r#"
        WITH incoming AS (
            SELECT
                $1::text AS id,
                $2::text AS job_id,
                $3::text AS dedupe_key,
                $4::text AS source,
                $5::text AS source_job_id,
                $6::text AS source_url,
                $7::text AS raw_hash,
                $8::jsonb AS raw_payload,
                $9::timestamptz AS fetched_at,
                $10::timestamptz AS last_seen_at,
                $11::boolean AS is_active
        ),
        existing AS (
            SELECT
                raw_hash,
                dedupe_key,
                is_active,
                last_seen_at::text AS last_seen_at
            FROM job_variants
            WHERE source = $4 AND source_job_id = $5
        ),
        upserted AS (
            INSERT INTO job_variants (
                id,
                job_id,
                dedupe_key,
                source,
                source_job_id,
                source_url,
                raw_hash,
                raw_payload,
                fetched_at,
                last_seen_at,
                is_active,
                inactivated_at
            )
            SELECT
                id,
                job_id,
                dedupe_key,
                source,
                source_job_id,
                source_url,
                raw_hash,
                raw_payload,
                fetched_at,
                last_seen_at,
                is_active,
                CASE
                    WHEN is_active THEN NULL
                    ELSE last_seen_at
                END
            FROM incoming
            ON CONFLICT (source, source_job_id)
            DO UPDATE SET
                id = CASE
                    WHEN EXCLUDED.last_seen_at < job_variants.last_seen_at THEN job_variants.id
                    ELSE EXCLUDED.id
                END,
                job_id = CASE
                    WHEN EXCLUDED.last_seen_at < job_variants.last_seen_at THEN job_variants.job_id
                    ELSE EXCLUDED.job_id
                END,
                dedupe_key = CASE
                    WHEN EXCLUDED.last_seen_at < job_variants.last_seen_at THEN job_variants.dedupe_key
                    ELSE EXCLUDED.dedupe_key
                END,
                source_url = CASE
                    WHEN EXCLUDED.last_seen_at < job_variants.last_seen_at THEN job_variants.source_url
                    ELSE EXCLUDED.source_url
                END,
                raw_hash = CASE
                    WHEN EXCLUDED.last_seen_at < job_variants.last_seen_at THEN job_variants.raw_hash
                    ELSE EXCLUDED.raw_hash
                END,
                raw_payload = CASE
                    WHEN EXCLUDED.last_seen_at < job_variants.last_seen_at THEN job_variants.raw_payload
                    ELSE EXCLUDED.raw_payload
                END,
                fetched_at = CASE
                    WHEN EXCLUDED.last_seen_at < job_variants.last_seen_at THEN job_variants.fetched_at
                    ELSE GREATEST(job_variants.fetched_at, EXCLUDED.fetched_at)
                END,
                last_seen_at = GREATEST(job_variants.last_seen_at, EXCLUDED.last_seen_at),
                is_active = CASE
                    WHEN EXCLUDED.last_seen_at < job_variants.last_seen_at THEN job_variants.is_active
                    ELSE EXCLUDED.is_active
                END,
                inactivated_at = CASE
                    WHEN EXCLUDED.last_seen_at < job_variants.last_seen_at THEN job_variants.inactivated_at
                    WHEN EXCLUDED.is_active THEN NULL
                    ELSE COALESCE(job_variants.inactivated_at, EXCLUDED.last_seen_at)
                END
            RETURNING 1
        )
        SELECT CASE
            WHEN NOT EXISTS (SELECT 1 FROM existing) THEN 'created'
            WHEN EXISTS (
                SELECT 1
                FROM existing
                WHERE $10::timestamptz < last_seen_at::timestamptz
            ) THEN 'unchanged'
            WHEN EXISTS (
                SELECT 1
                FROM existing
                WHERE raw_hash = $7
                  AND dedupe_key = $3
                  AND is_active = $11
            ) THEN 'unchanged'
            ELSE 'updated'
        END
        FROM upserted
        "#,
    )
    .bind(&variant.id)
    .bind(&variant.job_id)
    .bind(&variant.dedupe_key)
    .bind(&variant.source)
    .bind(&variant.source_job_id)
    .bind(&variant.source_url)
    .bind(&variant.raw_hash)
    .bind(sqlx::types::Json(&variant.raw_payload))
    .bind(&variant.fetched_at)
    .bind(&variant.last_seen_at)
    .bind(variant.is_active)
    .fetch_one(&mut **tx)
    .await
    .map_err(|error| {
        format!(
            "failed to upsert job variant '{}:{}': {error}",
            variant.source, variant.source_job_id
        )
    })?;

    match outcome.as_str() {
        "created" => Ok(VariantWriteResult::Created),
        "updated" => Ok(VariantWriteResult::Updated),
        "unchanged" => Ok(VariantWriteResult::Unchanged),
        other => Err(format!(
            "unexpected variant upsert outcome for '{}:{}': {other}",
            variant.source, variant.source_job_id
        )),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct InactivationResult {
    variants_inactivated: usize,
    job_ids: Vec<String>,
}

const PROFILE_NOTIFICATION_MATCH_SQL: &str = r#"
(
    (
        p.primary_role IS NOT NULL
        AND BTRIM(p.primary_role) <> ''
        AND j.haystack LIKE '%' || REPLACE(LOWER(p.primary_role), '_', ' ') || '%'
    )
    OR EXISTS (
        SELECT 1
        FROM jsonb_array_elements_text(p.skills) AS skill(value)
        WHERE LENGTH(BTRIM(skill.value)) >= 2
          AND j.haystack LIKE '%' || LOWER(skill.value) || '%'
    )
    OR EXISTS (
        SELECT 1
        FROM jsonb_array_elements_text(p.keywords) AS keyword(value)
        WHERE LENGTH(BTRIM(keyword.value)) >= 2
          AND j.haystack LIKE '%' || LOWER(keyword.value) || '%'
    )
)
"#;

async fn mark_missing_variants_inactive(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    source: &str,
    seen_source_job_ids: &[String],
    refreshed_at: &str,
) -> Result<InactivationResult, String> {
    let rows = sqlx::query_scalar::<_, String>(
        r#"
        UPDATE job_variants
        SET
            is_active = FALSE,
            inactivated_at = COALESCE(inactivated_at, $2::timestamptz)
        WHERE source = $1
          AND is_active = TRUE
          AND NOT (source_job_id = ANY($3::text[]))
        RETURNING job_id
        "#,
    )
    .bind(source)
    .bind(refreshed_at)
    .bind(seen_source_job_ids)
    .fetch_all(&mut **tx)
    .await
    .map_err(|error| {
        format!("failed to mark missing variants inactive for source '{source}': {error}")
    })?;

    if !rows.is_empty() {
        info!(
            source,
            refreshed_at,
            variants_inactivated = rows.len(),
            "marked missing source variants inactive"
        );
    }

    Ok(InactivationResult {
        variants_inactivated: rows.len(),
        job_ids: rows,
    })
}

async fn reconcile_jobs(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    job_ids: &[String],
) -> Result<ReconcileSummary, String> {
    let rows = sqlx::query_as::<_, (bool, bool)>(
        r#"
        WITH current_state AS (
            SELECT id AS job_id, is_active AS was_active
            FROM jobs
            WHERE id = ANY($1::text[])
        ),
        lifecycle AS (
            SELECT
                job_id,
                BOOL_OR(is_active) AS has_active_variant,
                MAX(last_seen_at) AS last_seen_at,
                MAX(inactivated_at) AS inactivated_at
            FROM job_variants
            WHERE job_id = ANY($1::text[])
            GROUP BY job_id
        ),
        updated AS (
            UPDATE jobs
            SET
                is_active = lifecycle.has_active_variant,
                last_seen_at = GREATEST(jobs.last_seen_at, lifecycle.last_seen_at),
                inactivated_at = CASE
                    WHEN lifecycle.has_active_variant THEN NULL
                    ELSE COALESCE(jobs.inactivated_at, lifecycle.inactivated_at, lifecycle.last_seen_at)
                END,
                reactivated_at = CASE
                    WHEN NOT jobs.is_active AND lifecycle.has_active_variant THEN lifecycle.last_seen_at
                    ELSE jobs.reactivated_at
                END
            FROM lifecycle
            INNER JOIN current_state ON current_state.job_id = lifecycle.job_id
            WHERE jobs.id = lifecycle.job_id
            RETURNING current_state.was_active AS was_active, lifecycle.has_active_variant AS is_active_now
        )
        SELECT was_active, is_active_now
        FROM updated
        "#,
    )
    .bind(job_ids)
    .fetch_all(&mut **tx)
    .await
    .map_err(|error| format!("failed to reconcile canonical jobs: {error}"))?;

    let summary = ReconcileSummary {
        jobs_inactivated: rows
            .iter()
            .filter(|(was_active, is_active_now)| *was_active && !*is_active_now)
            .count(),
        jobs_reactivated: rows
            .iter()
            .filter(|(was_active, is_active_now)| !*was_active && *is_active_now)
            .count(),
    };

    if summary.jobs_inactivated > 0 || summary.jobs_reactivated > 0 {
        info!(
            jobs_inactivated = summary.jobs_inactivated,
            jobs_reactivated = summary.jobs_reactivated,
            "reconciled canonical job lifecycle state"
        );
    }

    Ok(summary)
}

async fn create_profile_notifications(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    job_ids: &[String],
) -> Result<(), String> {
    if job_ids.is_empty() {
        return Ok(());
    }

    insert_new_job_notifications(tx, job_ids).await?;
    insert_reactivated_job_notifications(tx, job_ids).await?;

    Ok(())
}

async fn insert_new_job_notifications(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    job_ids: &[String],
) -> Result<(), String> {
    let query = format!(
        r#"
        WITH candidate_jobs AS (
            SELECT
                id,
                title,
                company_name,
                first_seen_at,
                last_seen_at,
                LOWER(CONCAT_WS(' ', title, company_name, COALESCE(location, ''), description_text)) AS haystack
            FROM jobs
            WHERE id = ANY($1::text[])
              AND first_seen_at = last_seen_at
        ),
        matched AS (
            SELECT
                p.id AS profile_id,
                COUNT(*)::int AS matched_count,
                ARRAY_AGG(j.id ORDER BY j.last_seen_at DESC) AS job_ids,
                (ARRAY_AGG(j.title ORDER BY j.last_seen_at DESC))[1] AS sample_title,
                (ARRAY_AGG(j.company_name ORDER BY j.last_seen_at DESC))[1] AS sample_company
            FROM profiles p
            INNER JOIN candidate_jobs j ON TRUE
            WHERE {PROFILE_NOTIFICATION_MATCH_SQL}
            GROUP BY p.id
        )
        INSERT INTO notifications (id, profile_id, type, title, body, payload)
        SELECT
            md5(profile_id || ':new_jobs_found:' || clock_timestamp()::text),
            profile_id,
            'new_jobs_found',
            CASE
                WHEN matched_count = 1 THEN 'New job matched your profile'
                ELSE matched_count::text || ' new jobs matched your profile'
            END,
            CASE
                WHEN matched_count = 1 THEN sample_title || ' at ' || sample_company || ' matched your current profile.'
                ELSE sample_title || ' at ' || sample_company || ' matched your current profile, plus ' || (matched_count - 1)::text || ' more.'
            END,
            jsonb_build_object(
                'count', matched_count,
                'job_ids', job_ids
            )
        FROM matched
        "#,
    );

    sqlx::query(&query)
        .bind(job_ids)
        .execute(&mut **tx)
        .await
        .map_err(|error| format!("failed to create new job notifications: {error}"))?;

    Ok(())
}

async fn insert_reactivated_job_notifications(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    job_ids: &[String],
) -> Result<(), String> {
    let query = format!(
        r#"
        WITH candidate_jobs AS (
            SELECT
                id,
                title,
                company_name,
                last_seen_at,
                reactivated_at,
                LOWER(CONCAT_WS(' ', title, company_name, COALESCE(location, ''), description_text)) AS haystack
            FROM jobs
            WHERE id = ANY($1::text[])
              AND reactivated_at IS NOT NULL
              AND reactivated_at = last_seen_at
        ),
        matched AS (
            SELECT
                p.id AS profile_id,
                COUNT(*)::int AS matched_count,
                ARRAY_AGG(j.id ORDER BY j.last_seen_at DESC) AS job_ids,
                (ARRAY_AGG(j.title ORDER BY j.last_seen_at DESC))[1] AS sample_title,
                (ARRAY_AGG(j.company_name ORDER BY j.last_seen_at DESC))[1] AS sample_company
            FROM profiles p
            INNER JOIN candidate_jobs j ON TRUE
            WHERE {PROFILE_NOTIFICATION_MATCH_SQL}
            GROUP BY p.id
        )
        INSERT INTO notifications (id, profile_id, type, title, body, payload)
        SELECT
            md5(profile_id || ':job_reactivated:' || clock_timestamp()::text),
            profile_id,
            'job_reactivated',
            CASE
                WHEN matched_count = 1 THEN 'A job reactivated for your profile'
                ELSE matched_count::text || ' jobs reactivated for your profile'
            END,
            CASE
                WHEN matched_count = 1 THEN sample_title || ' at ' || sample_company || ' is active again.'
                ELSE sample_title || ' at ' || sample_company || ' is active again, plus ' || (matched_count - 1)::text || ' more.'
            END,
            jsonb_build_object(
                'count', matched_count,
                'job_ids', job_ids
            )
        FROM matched
        "#,
    );

    sqlx::query(&query)
        .bind(job_ids)
        .execute(&mut **tx)
        .await
        .map_err(|error| format!("failed to create reactivated job notifications: {error}"))?;

    Ok(())
}

fn build_source_refreshes(batch: &IngestionBatch) -> BTreeMap<String, SourceRefresh> {
    let mut refreshes = BTreeMap::<String, SourceRefresh>::new();

    for variant in &batch.job_variants {
        let refresh = refreshes
            .entry(variant.source.clone())
            .or_insert_with(|| SourceRefresh {
                refreshed_at: variant.fetched_at.clone(),
                seen_source_job_ids: Vec::new(),
            });

        if variant.fetched_at > refresh.refreshed_at {
            refresh.refreshed_at = variant.fetched_at.clone();
        }

        refresh
            .seen_source_job_ids
            .push(variant.source_job_id.clone());
    }

    for refresh in refreshes.values_mut() {
        refresh.seen_source_job_ids.sort();
        refresh.seen_source_job_ids.dedup();
    }

    refreshes
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

    use super::{
        SourceRefresh, build_source_refreshes, merge_job, refresh_market_snapshots, upsert_batch,
    };
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
