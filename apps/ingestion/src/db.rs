use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;

use crate::models::{IngestionBatch, JobVariant, NormalizedJob};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UpsertSummary {
    pub jobs_written: usize,
    pub variants_created: usize,
    pub variants_updated: usize,
    pub variants_unchanged: usize,
}

pub async fn connect(database_url: &str) -> Result<PgPool, String> {
    PgPoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await
        .map_err(|error| format!("failed to connect to Postgres: {error}"))
}

pub async fn upsert_batch(pool: &PgPool, batch: &IngestionBatch) -> Result<UpsertSummary, String> {
    let mut tx = pool
        .begin()
        .await
        .map_err(|error| format!("failed to begin transaction: {error}"))?;
    let mut summary = UpsertSummary::default();

    for job in &batch.jobs {
        upsert_job(&mut tx, job).await?;
    }

    for variant in &batch.job_variants {
        match upsert_job_variant(&mut tx, variant).await? {
            VariantWriteResult::Created => summary.variants_created += 1,
            VariantWriteResult::Updated => summary.variants_updated += 1,
            VariantWriteResult::Unchanged => summary.variants_unchanged += 1,
        }
    }

    tx.commit()
        .await
        .map_err(|error| format!("failed to commit transaction: {error}"))?;

    summary.jobs_written = batch.jobs.len();
    Ok(summary)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum VariantWriteResult {
    Created,
    Updated,
    Unchanged,
}

async fn upsert_job(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    job: &NormalizedJob,
) -> Result<(), String> {
    sqlx::query(
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
            last_seen_at,
            is_active
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
            $12::timestamptz,
            $13
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
            last_seen_at = EXCLUDED.last_seen_at,
            is_active = EXCLUDED.is_active
        "#,
    )
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
                $3::text AS source,
                $4::text AS source_job_id,
                $5::text AS source_url,
                $6::text AS raw_hash,
                $7::jsonb AS raw_payload,
                $8::timestamptz AS fetched_at
        ),
        existing AS (
            SELECT raw_hash
            FROM job_variants
            WHERE source = $3 AND source_job_id = $4
        ),
        upserted AS (
            INSERT INTO job_variants (
                id,
                job_id,
                source,
                source_job_id,
                source_url,
                raw_hash,
                raw_payload,
                fetched_at
            )
            SELECT
                id,
                job_id,
                source,
                source_job_id,
                source_url,
                raw_hash,
                raw_payload,
                fetched_at
            FROM incoming
            ON CONFLICT (source, source_job_id)
            DO UPDATE SET
                id = EXCLUDED.id,
                job_id = EXCLUDED.job_id,
                source_url = EXCLUDED.source_url,
                raw_hash = EXCLUDED.raw_hash,
                raw_payload = EXCLUDED.raw_payload,
                fetched_at = EXCLUDED.fetched_at
            RETURNING 1
        )
        SELECT CASE
            WHEN NOT EXISTS (SELECT 1 FROM existing) THEN 'created'
            WHEN EXISTS (SELECT 1 FROM existing WHERE raw_hash = $6) THEN 'unchanged'
            ELSE 'updated'
        END
        FROM upserted
        "#,
    )
    .bind(&variant.id)
    .bind(&variant.job_id)
    .bind(&variant.source)
    .bind(&variant.source_job_id)
    .bind(&variant.source_url)
    .bind(&variant.raw_hash)
    .bind(sqlx::types::Json(&variant.raw_payload))
    .bind(&variant.fetched_at)
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

impl Default for UpsertSummary {
    fn default() -> Self {
        Self {
            jobs_written: 0,
            variants_created: 0,
            variants_updated: 0,
            variants_unchanged: 0,
        }
    }
}
