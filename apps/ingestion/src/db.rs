use std::collections::{BTreeMap, BTreeSet};

use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;

use crate::models::{IngestionBatch, JobVariant, NormalizedJob};

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
struct SourceRefresh {
    refreshed_at: String,
    seen_source_job_ids: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ReconcileSummary {
    jobs_inactivated: usize,
    jobs_reactivated: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum VariantWriteResult {
    Created,
    Updated,
    Unchanged,
}

pub async fn connect(database_url: &str) -> Result<PgPool, String> {
    PgPoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await
        .map_err(|error| format!("failed to connect to Postgres: {error}"))
}

/// Apply all engine-api migrations so ingestion can run standalone without
/// engine-api having started first. Safe to call multiple times (sqlx is idempotent).
pub async fn run_migrations(pool: &PgPool) -> Result<(), String> {
    sqlx::migrate!("../engine-api/migrations")
        .run(pool)
        .await
        .map_err(|error| format!("failed to run migrations: {error}"))
}

pub async fn upsert_batch(pool: &PgPool, batch: &IngestionBatch) -> Result<UpsertSummary, String> {
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

async fn resolve_batch(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    batch: &IngestionBatch,
) -> Result<IngestionBatch, String> {
    if batch.job_variants.is_empty() {
        return Ok(batch.clone());
    }

    if batch.jobs.len() != batch.job_variants.len() {
        return Err(format!(
            "adapter-backed batches must contain one job per variant; got {} jobs and {} variants",
            batch.jobs.len(),
            batch.job_variants.len()
        ));
    }

    let mut jobs_by_id = BTreeMap::<String, NormalizedJob>::new();
    let mut variants = Vec::with_capacity(batch.job_variants.len());
    let mut dedupe_job_ids = BTreeMap::<String, String>::new();

    for (job, variant) in batch.jobs.iter().zip(batch.job_variants.iter()) {
        let resolved_job_id = if let Some(job_id) = dedupe_job_ids.get(&variant.dedupe_key) {
            job_id.clone()
        } else if let Some(job_id) =
            existing_job_id_for_variant(tx, &variant.source, &variant.source_job_id).await?
        {
            job_id
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

    Ok(IngestionBatch {
        jobs: jobs_by_id.into_values().collect(),
        job_variants: variants,
    })
}

async fn existing_job_id_for_variant(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    source: &str,
    source_job_id: &str,
) -> Result<Option<String>, String> {
    sqlx::query_scalar::<_, String>(
        r#"
        SELECT job_id
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
            is_active = EXCLUDED.is_active,
            inactivated_at = CASE
                WHEN EXCLUDED.is_active THEN NULL
                ELSE COALESCE(jobs.inactivated_at, EXCLUDED.last_seen_at)
            END,
            reactivated_at = CASE
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
            SELECT raw_hash
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
                id = EXCLUDED.id,
                job_id = EXCLUDED.job_id,
                dedupe_key = EXCLUDED.dedupe_key,
                source_url = EXCLUDED.source_url,
                raw_hash = EXCLUDED.raw_hash,
                raw_payload = EXCLUDED.raw_payload,
                fetched_at = EXCLUDED.fetched_at,
                last_seen_at = GREATEST(job_variants.last_seen_at, EXCLUDED.last_seen_at),
                is_active = EXCLUDED.is_active,
                inactivated_at = CASE
                    WHEN EXCLUDED.is_active THEN NULL
                    ELSE COALESCE(job_variants.inactivated_at, EXCLUDED.last_seen_at)
                END
            RETURNING 1
        )
        SELECT CASE
            WHEN NOT EXISTS (SELECT 1 FROM existing) THEN 'created'
            WHEN EXISTS (SELECT 1 FROM existing WHERE raw_hash = $7) THEN 'unchanged'
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

    Ok(ReconcileSummary {
        jobs_inactivated: rows
            .iter()
            .filter(|(was_active, is_active_now)| *was_active && !*is_active_now)
            .count(),
        jobs_reactivated: rows
            .iter()
            .filter(|(was_active, is_active_now)| !*was_active && *is_active_now)
            .count(),
    })
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
    use crate::models::{JobVariant, NormalizedJob};

    use super::{SourceRefresh, build_source_refreshes, merge_job};

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
}
