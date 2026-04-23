use std::collections::BTreeMap;

use sqlx::types::Json;
use tracing::warn;

use crate::models::{IngestionBatch, JobVariant, NormalizedJob};

#[derive(Debug, Clone, PartialEq, Eq, sqlx::FromRow)]
pub(super) struct ExistingVariantState {
    pub(super) job_id: String,
    pub(super) dedupe_key: String,
    pub(super) raw_hash: String,
    pub(super) last_seen_at: String,
    pub(super) fetched_at: String,
    pub(super) is_active: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum VariantWriteResult {
    Created,
    Updated,
    Unchanged,
}

pub(super) async fn resolve_batch(
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
        tracing::info!(
            stale_variants_skipped,
            "adapter-backed batch skipped stale source variants"
        );
    }

    Ok(IngestionBatch {
        jobs: jobs_by_id.into_values().collect(),
        job_variants: variants,
    })
}

pub(super) async fn upsert_job(
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

pub(super) async fn upsert_job_variant(
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
    .bind(Json(&variant.raw_payload))
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

pub(super) fn merge_job(current: &mut NormalizedJob, incoming: &NormalizedJob) {
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
