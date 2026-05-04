use std::collections::BTreeMap;

use chrono::Duration;
use sqlx::types::Json;
use tracing::warn;

use crate::error::{IngestionError, Result};

use crate::models::{IngestionBatch, JobVariant, NormalizedJob};
use crate::scrapers::{compute_job_quality_score, extract_skills};

#[derive(Debug, Clone, PartialEq, Eq, sqlx::FromRow)]
pub(super) struct ExistingVariantState {
    pub(super) job_id: String,
    pub(super) dedupe_key: String,
    pub(super) raw_hash: String,
    pub(super) last_seen_at: String,
    pub(super) fetched_at: String,
    pub(super) is_active: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, sqlx::FromRow)]
struct DuplicateCandidate {
    id: String,
    title: String,
    salary_min: Option<i32>,
    salary_max: Option<i32>,
    salary_currency: Option<String>,
    salary_usd_min: Option<i32>,
    salary_usd_max: Option<i32>,
    posted_at: Option<String>,
    first_seen_at: String,
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
) -> Result<IngestionBatch> {
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

        let mut resolved_job = job.clone();
        resolved_job.id = resolved_job_id;
        if existing_variant.is_none() {
            resolved_job.duplicate_of =
                find_cross_source_duplicate(tx, &resolved_job, variant).await?;
        }
        dedupe_job_ids.insert(variant.dedupe_key.clone(), resolved_job.id.clone());
        let mut resolved_variant = variant.clone();
        resolved_variant.job_id = resolved_job.id.clone();
        jobs_by_id
            .entry(resolved_job.id.clone())
            .and_modify(|current| merge_job(current, &resolved_job))
            .or_insert(resolved_job);
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
) -> Result<()> {
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
            extracted_skills,
            salary_min,
            salary_max,
            salary_currency,
            salary_usd_min,
            salary_usd_max,
            quality_score,
            company_meta,
            posted_at,
            first_seen_at,
            last_seen_at,
            is_active,
            inactivated_at,
            reactivated_at,
            duplicate_of
        )
        VALUES (
            $1,
            $2,
            $3,
            $4,
            $5,
            $6,
            $7,
            $8::jsonb,
            $9,
            $10,
            $11,
            $12,
            $13,
            $14,
            $15::jsonb,
            $16::timestamptz,
            COALESCE($16::timestamptz, $17::timestamptz),
            $17::timestamptz,
            $18,
            CASE
                WHEN $18 THEN NULL
                ELSE $17::timestamptz
            END,
            NULL,
            $19
        )
        ON CONFLICT (id)
        DO UPDATE SET
            title = EXCLUDED.title,
            company_name = EXCLUDED.company_name,
            location = EXCLUDED.location,
            remote_type = EXCLUDED.remote_type,
            seniority = EXCLUDED.seniority,
            description_text = EXCLUDED.description_text,
            extracted_skills = EXCLUDED.extracted_skills,
            salary_min = EXCLUDED.salary_min,
            salary_max = EXCLUDED.salary_max,
            salary_currency = EXCLUDED.salary_currency,
            salary_usd_min = EXCLUDED.salary_usd_min,
            salary_usd_max = EXCLUDED.salary_usd_max,
            quality_score = EXCLUDED.quality_score,
            company_meta = EXCLUDED.company_meta,
            posted_at = EXCLUDED.posted_at,
            duplicate_of = EXCLUDED.duplicate_of,
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
            extracted_skills,
            salary_min,
            salary_max,
            salary_currency,
            salary_usd_min,
            salary_usd_max,
            quality_score,
            company_meta,
            posted_at,
            first_seen_at,
            last_seen_at,
            is_active,
            inactivated_at,
            reactivated_at,
            duplicate_of
        )
        VALUES (
            $1,
            $2,
            $3,
            $4,
            $5,
            $6,
            $7,
            $8::jsonb,
            $9,
            $10,
            $11,
            $12,
            $13,
            $14,
            $15::jsonb,
            $16::timestamptz,
            COALESCE($16::timestamptz, $17::timestamptz),
            $17::timestamptz,
            $18,
            CASE
                WHEN $18 THEN NULL
                ELSE $17::timestamptz
            END,
            NULL,
            $19
        )
        ON CONFLICT (id)
        DO UPDATE SET
            title = EXCLUDED.title,
            company_name = EXCLUDED.company_name,
            location = EXCLUDED.location,
            remote_type = EXCLUDED.remote_type,
            seniority = EXCLUDED.seniority,
            description_text = EXCLUDED.description_text,
            extracted_skills = EXCLUDED.extracted_skills,
            salary_min = EXCLUDED.salary_min,
            salary_max = EXCLUDED.salary_max,
            salary_currency = EXCLUDED.salary_currency,
            salary_usd_min = EXCLUDED.salary_usd_min,
            salary_usd_max = EXCLUDED.salary_usd_max,
            quality_score = EXCLUDED.quality_score,
            company_meta = EXCLUDED.company_meta,
            posted_at = EXCLUDED.posted_at,
            duplicate_of = EXCLUDED.duplicate_of,
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
    let extracted_skills = if job.extracted_skills.is_empty() {
        extract_skills(&job.description_text)
    } else {
        job.extracted_skills.clone()
    };
    let quality_score = job.quality_score.unwrap_or_else(|| {
        let mut scored_job = job.clone();
        scored_job.extracted_skills = extracted_skills.clone();
        compute_job_quality_score(&scored_job)
    });
    let company_meta = job.company_meta.as_ref().map(Json);

    sqlx::query(query)
        .bind(&job.id)
        .bind(&job.title)
        .bind(&job.company_name)
        .bind(&job.location)
        .bind(&job.remote_type)
        .bind(&job.seniority)
        .bind(&job.description_text)
        .bind(Json(&extracted_skills))
        .bind(job.salary_min)
        .bind(job.salary_max)
        .bind(&job.salary_currency)
        .bind(job.salary_usd_min)
        .bind(job.salary_usd_max)
        .bind(quality_score)
        .bind(company_meta)
        .bind(&job.posted_at)
        .bind(&job.last_seen_at)
        .bind(job.is_active)
        .bind(&job.duplicate_of)
        .execute(&mut **tx)
        .await
        .map_err(IngestionError::Database)?;

    Ok(())
}

pub(super) async fn upsert_job_variant(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    variant: &JobVariant,
) -> Result<VariantWriteResult> {
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
    .map_err(IngestionError::Database)?;

    match outcome.as_str() {
        "created" => Ok(VariantWriteResult::Created),
        "updated" => Ok(VariantWriteResult::Updated),
        "unchanged" => Ok(VariantWriteResult::Unchanged),
        other => Err(IngestionError::Validation(format!(
            "unexpected variant upsert outcome for '{}:{}': {other}",
            variant.source, variant.source_job_id
        ))),
    }
}

async fn find_cross_source_duplicate(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    job: &NormalizedJob,
    variant: &JobVariant,
) -> Result<Option<String>> {
    let candidates = sqlx::query_as::<_, DuplicateCandidate>(
        r#"
        SELECT
            jobs.id,
            jobs.title,
            jobs.salary_min,
            jobs.salary_max,
            jobs.salary_currency,
            jobs.salary_usd_min,
            jobs.salary_usd_max,
            TO_CHAR(jobs.posted_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS"Z"') AS posted_at,
            TO_CHAR(jobs.first_seen_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS"Z"') AS first_seen_at
        FROM jobs
        WHERE jobs.id <> $1
          AND jobs.duplicate_of IS NULL
          AND LOWER(jobs.company_name) = LOWER($2)
          AND EXISTS (
              SELECT 1
              FROM job_variants
              WHERE job_variants.job_id = jobs.id
                AND job_variants.source <> $3
          )
          AND ABS(EXTRACT(EPOCH FROM (
              COALESCE(jobs.posted_at, jobs.first_seen_at) - COALESCE($4::timestamptz, $5::timestamptz)
          ))) <= 604800
        ORDER BY jobs.is_active DESC, jobs.last_seen_at DESC, jobs.first_seen_at ASC
        LIMIT 25
        "#,
    )
    .bind(&job.id)
    .bind(&job.company_name)
    .bind(&variant.source)
    .bind(&job.posted_at)
    .bind(&job.last_seen_at)
    .fetch_all(&mut **tx)
    .await
    .map_err(IngestionError::Database)?;

    Ok(candidates
        .into_iter()
        .find(|candidate| is_fuzzy_duplicate(job, candidate))
        .map(|candidate| candidate.id))
}

fn is_fuzzy_duplicate(job: &NormalizedJob, candidate: &DuplicateCandidate) -> bool {
    title_similarity(&job.title, &candidate.title) > 0.8
        && salary_matches(
            job.salary_min,
            job.salary_max,
            job.salary_currency.as_deref(),
            job.salary_usd_min,
            job.salary_usd_max,
            candidate.salary_min,
            candidate.salary_max,
            candidate.salary_currency.as_deref(),
            candidate.salary_usd_min,
            candidate.salary_usd_max,
        )
        && posted_within_days(
            job.posted_at.as_deref().unwrap_or(&job.last_seen_at),
            candidate
                .posted_at
                .as_deref()
                .unwrap_or(&candidate.first_seen_at),
            7,
        )
}

fn title_similarity(left: &str, right: &str) -> f64 {
    let left = normalized_title(left);
    let right = normalized_title(right);

    if left.is_empty() || right.is_empty() {
        return 0.0;
    }

    let distance = levenshtein(&left, &right) as f64;
    let max_len = left.chars().count().max(right.chars().count()) as f64;
    1.0 - (distance / max_len)
}

fn normalized_title(value: &str) -> String {
    value
        .split_whitespace()
        .filter(|chunk| !chunk.is_empty())
        .map(|chunk| {
            chunk
                .chars()
                .filter(|character| character.is_alphanumeric())
                .flat_map(char::to_lowercase)
                .collect::<String>()
        })
        .filter(|chunk| !chunk.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

fn levenshtein(left: &str, right: &str) -> usize {
    let right_chars = right.chars().collect::<Vec<_>>();
    let mut previous = (0..=right_chars.len()).collect::<Vec<_>>();
    let mut current = vec![0; right_chars.len() + 1];

    for (left_index, left_char) in left.chars().enumerate() {
        current[0] = left_index + 1;

        for (right_index, right_char) in right_chars.iter().enumerate() {
            let substitution_cost = usize::from(left_char != *right_char);
            current[right_index + 1] = (previous[right_index + 1] + 1)
                .min(current[right_index] + 1)
                .min(previous[right_index] + substitution_cost);
        }

        std::mem::swap(&mut previous, &mut current);
    }

    previous[right_chars.len()]
}

#[allow(clippy::too_many_arguments)]
fn salary_matches(
    left_min: Option<i32>,
    left_max: Option<i32>,
    left_currency: Option<&str>,
    left_usd_min: Option<i32>,
    left_usd_max: Option<i32>,
    right_min: Option<i32>,
    right_max: Option<i32>,
    right_currency: Option<&str>,
    right_usd_min: Option<i32>,
    right_usd_max: Option<i32>,
) -> bool {
    let left_range = salary_range(left_usd_min, left_usd_max).or_else(|| {
        same_currency(left_currency, right_currency).and_then(|_| salary_range(left_min, left_max))
    });
    let right_range = salary_range(right_usd_min, right_usd_max).or_else(|| {
        same_currency(left_currency, right_currency)
            .and_then(|_| salary_range(right_min, right_max))
    });

    match (left_range, right_range) {
        (Some(left), Some(right)) => ranges_are_close(left, right),
        (None, None) => true,
        _ => false,
    }
}

fn same_currency(left: Option<&str>, right: Option<&str>) -> Option<()> {
    match (left, right) {
        (Some(left), Some(right)) if left.eq_ignore_ascii_case(right) => Some(()),
        _ => None,
    }
}

fn salary_range(min: Option<i32>, max: Option<i32>) -> Option<(i32, i32)> {
    match (min, max) {
        (Some(min), Some(max)) => Some((min.min(max), min.max(max))),
        (Some(value), None) | (None, Some(value)) => Some((value, value)),
        (None, None) => None,
    }
}

fn ranges_are_close(left: (i32, i32), right: (i32, i32)) -> bool {
    if left.0 <= right.1 && right.0 <= left.1 {
        return true;
    }

    let left_midpoint = (left.0 + left.1) as f64 / 2.0;
    let right_midpoint = (right.0 + right.1) as f64 / 2.0;
    let larger = left_midpoint.abs().max(right_midpoint.abs()).max(1.0);

    ((left_midpoint - right_midpoint).abs() / larger) <= 0.1
}

fn posted_within_days(left: &str, right: &str, days: i64) -> bool {
    let Ok(left) = chrono::DateTime::parse_from_rfc3339(left) else {
        return false;
    };
    let Ok(right) = chrono::DateTime::parse_from_rfc3339(right) else {
        return false;
    };

    (left - right).num_seconds().abs() <= Duration::days(days).num_seconds()
}

async fn existing_variant_state(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    source: &str,
    source_job_id: &str,
) -> Result<Option<ExistingVariantState>> {
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
    .map_err(IngestionError::Database)
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
) -> Result<()> {
    if existing_variant.dedupe_key == incoming_variant.dedupe_key {
        return Ok(());
    }

    if let Some(conflict_job_id) =
        existing_job_id_for_dedupe_key(tx, &incoming_variant.dedupe_key).await?
        && conflict_job_id != existing_variant.job_id
    {
        return Err(IngestionError::Validation(format!(
            "source variant '{}:{}' changed dedupe fingerprint from '{}' to '{}' but the new fingerprint already belongs to canonical job '{}' instead of '{}'",
            incoming_variant.source,
            incoming_variant.source_job_id,
            existing_variant.dedupe_key,
            incoming_variant.dedupe_key,
            conflict_job_id,
            existing_variant.job_id
        )));
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
) -> Result<Option<String>> {
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
    .map_err(IngestionError::Database)
}

pub(super) fn merge_job(current: &mut NormalizedJob, incoming: &NormalizedJob) {
    current.duplicate_of = current
        .duplicate_of
        .clone()
        .or_else(|| incoming.duplicate_of.clone());

    if incoming.last_seen_at >= current.last_seen_at {
        current.title = incoming.title.clone();
        current.company_name = incoming.company_name.clone();
        current.company_meta = pick_company_meta(&incoming.company_meta, &current.company_meta);
        current.description_text = incoming.description_text.clone();
        current.extracted_skills = incoming.extracted_skills.clone();
        current.location = pick_optional(&incoming.location, &current.location);
        current.remote_type = pick_optional(&incoming.remote_type, &current.remote_type);
        current.seniority = pick_optional(&incoming.seniority, &current.seniority);
        current.salary_min = incoming.salary_min.or(current.salary_min);
        current.salary_max = incoming.salary_max.or(current.salary_max);
        current.salary_currency =
            pick_optional(&incoming.salary_currency, &current.salary_currency);
        current.salary_usd_min = incoming.salary_usd_min.or(current.salary_usd_min);
        current.salary_usd_max = incoming.salary_usd_max.or(current.salary_usd_max);
        current.quality_score = incoming.quality_score.or(current.quality_score);
    } else {
        current.company_meta = pick_company_meta(&current.company_meta, &incoming.company_meta);
        if current.extracted_skills.is_empty() {
            current.extracted_skills = incoming.extracted_skills.clone();
        }
        current.location = pick_optional(&current.location, &incoming.location);
        current.remote_type = pick_optional(&current.remote_type, &incoming.remote_type);
        current.seniority = pick_optional(&current.seniority, &incoming.seniority);
        current.salary_min = current.salary_min.or(incoming.salary_min);
        current.salary_max = current.salary_max.or(incoming.salary_max);
        current.salary_currency =
            pick_optional(&current.salary_currency, &incoming.salary_currency);
        current.salary_usd_min = current.salary_usd_min.or(incoming.salary_usd_min);
        current.salary_usd_max = current.salary_usd_max.or(incoming.salary_usd_max);
        current.quality_score = current.quality_score.or(incoming.quality_score);
    }

    current.posted_at = earliest_timestamp(current.posted_at.as_ref(), incoming.posted_at.as_ref());
    current.last_seen_at = current
        .last_seen_at
        .clone()
        .max(incoming.last_seen_at.clone());
    current.is_active = current.is_active || incoming.is_active;
    current.quality_score = Some(compute_job_quality_score(current));
}

fn pick_optional(primary: &Option<String>, fallback: &Option<String>) -> Option<String> {
    primary.clone().or_else(|| fallback.clone())
}

fn pick_company_meta(
    primary: &Option<crate::models::CompanyMeta>,
    fallback: &Option<crate::models::CompanyMeta>,
) -> Option<crate::models::CompanyMeta> {
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

#[cfg(test)]
mod tests {
    use super::{DuplicateCandidate, is_fuzzy_duplicate, title_similarity};
    use crate::models::NormalizedJob;

    fn job(title: &str, company_name: &str, salary_min: Option<i32>) -> NormalizedJob {
        NormalizedJob {
            id: "job_new".to_string(),
            duplicate_of: None,
            title: title.to_string(),
            company_name: company_name.to_string(),
            company_meta: None,
            location: Some("Kyiv".to_string()),
            remote_type: Some("remote".to_string()),
            seniority: Some("senior".to_string()),
            description_text: "Rust PostgreSQL backend role".to_string(),
            extracted_skills: Vec::new(),
            salary_min,
            salary_max: salary_min.map(|value| value + 500),
            salary_currency: Some("USD".to_string()),
            salary_usd_min: salary_min,
            salary_usd_max: salary_min.map(|value| value + 500),
            quality_score: None,
            posted_at: Some("2026-04-20T09:00:00Z".to_string()),
            last_seen_at: "2026-04-20T10:00:00Z".to_string(),
            is_active: true,
        }
    }

    fn candidate(title: &str, salary_min: Option<i32>, posted_at: &str) -> DuplicateCandidate {
        DuplicateCandidate {
            id: "job_existing".to_string(),
            title: title.to_string(),
            salary_min,
            salary_max: salary_min.map(|value| value + 500),
            salary_currency: Some("USD".to_string()),
            salary_usd_min: salary_min,
            salary_usd_max: salary_min.map(|value| value + 500),
            posted_at: Some(posted_at.to_string()),
            first_seen_at: posted_at.to_string(),
        }
    }

    #[test]
    fn title_similarity_accepts_minor_wording_changes() {
        assert!(title_similarity("Senior Rust Backend Engineer", "Sr Rust Backend Engineer") > 0.8);
        assert!(title_similarity("Senior Rust Backend Engineer", "Product Designer") < 0.8);
    }

    #[test]
    fn fuzzy_duplicate_requires_similar_title_salary_and_recent_posting() {
        let incoming = job("Senior Rust Backend Engineer", "SignalHire", Some(4500));

        assert!(is_fuzzy_duplicate(
            &incoming,
            &candidate(
                "Senior Rust Backend Engineer",
                Some(4600),
                "2026-04-24T09:00:00Z"
            )
        ));
        assert!(!is_fuzzy_duplicate(
            &incoming,
            &candidate(
                "Senior Rust Backend Engineer",
                Some(7000),
                "2026-04-24T09:00:00Z"
            )
        ));
        assert!(!is_fuzzy_duplicate(
            &incoming,
            &candidate(
                "Senior Rust Backend Engineer",
                Some(4600),
                "2026-05-01T09:00:00Z"
            )
        ));
    }
}
