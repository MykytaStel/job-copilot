use std::collections::BTreeMap;

use tracing::info;

use crate::models::IngestionBatch;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct SourceRefresh {
    pub(super) refreshed_at: String,
    pub(super) seen_source_job_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct InactivationResult {
    pub(super) variants_inactivated: usize,
    pub(super) job_ids: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct ReconcileSummary {
    pub(super) jobs_inactivated: usize,
    pub(super) jobs_reactivated: usize,
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

pub(super) fn build_source_refreshes(batch: &IngestionBatch) -> BTreeMap<String, SourceRefresh> {
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

pub(super) async fn mark_missing_variants_inactive(
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

pub(super) async fn reconcile_jobs(
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

pub(super) async fn create_profile_notifications(
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
