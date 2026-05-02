use std::collections::BTreeMap;

use sqlx::FromRow;
use sqlx::types::Json;
use tracing::info;

use crate::models::IngestionBatch;

mod job_alert_scoring {
    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../shared/rust/job_alert_scoring.rs"
    ));
}

use job_alert_scoring::{
    AlertJob, AlertProfile, JOB_ALERT_SCORE_THRESHOLD, alert_profile_has_signals, score_job_alert,
};

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
    insert_market_company_hiring_again_notifications(tx, job_ids).await?;

    Ok(())
}

const JOB_ROLE_CLASSIFIER_SQL: &str = r#"
CASE
    WHEN title ILIKE '%engineering manager%'
      OR title ILIKE '%head of engineering%'
      OR title ILIKE '%vp of engineering%'
      OR title ILIKE '%software development manager%'
    THEN 'engineering_manager'
    WHEN title ILIKE '%tech lead%'
      OR title ILIKE '%technical lead%'
      OR title ILIKE '%lead engineer%'
      OR title ILIKE '%lead developer%'
      OR title ILIKE '%lead software engineer%'
    THEN 'tech_lead'
    WHEN title ILIKE '%product manager%'
      OR title ILIKE '%product owner%'
      OR title ILIKE '%digital product manager%'
    THEN 'product_manager'
    WHEN title ILIKE '%project manager%'
      OR title ILIKE '%delivery manager%'
      OR title ILIKE '%program coordinator%'
    THEN 'project_manager'
    WHEN title ILIKE '%product designer%'
      OR title ILIKE '%ui ux designer%'
      OR title ILIKE '%ui/ux designer%'
      OR title ILIKE '%ux designer%'
      OR title ILIKE '%ui designer%'
      OR title ILIKE '%interaction designer%'
    THEN 'product_designer'
    WHEN title ~* '(^|[^a-z])(qa|sdet|tester)([^a-z]|$)'
      OR title ILIKE '%quality assurance%'
      OR title ILIKE '%test engineer%'
      OR title ILIKE '%automation qa%'
      OR title ILIKE '%manual qa%'
      OR title ILIKE '%software tester%'
    THEN 'qa_engineer'
    WHEN title ILIKE '%machine learning%'
      OR title ILIKE '%ml engineer%'
      OR title ILIKE '%ai engineer%'
      OR title ILIKE '%ai developer%'
      OR title ILIKE '%artificial intelligence%'
      OR title ILIKE '%data scientist%'
      OR title ILIKE '%deep learning%'
      OR title ILIKE '%computer vision%'
      OR title ILIKE '%nlp engineer%'
      OR title ILIKE '%research engineer%'
    THEN 'ml_engineer'
    WHEN title ILIKE '%data engineer%'
      OR title ILIKE '%big data%'
      OR title ILIKE '%etl developer%'
      OR title ILIKE '%data analyst%'
      OR title ILIKE '%analytics engineer%'
      OR title ILIKE '%bi developer%'
      OR title ILIKE '%business intelligence%'
      OR title ILIKE '%database developer%'
    THEN 'data_engineer'
    WHEN title ILIKE '%mobile engineer%'
      OR title ILIKE '%mobile developer%'
      OR title ILIKE '%react native%'
      OR title ILIKE '%ios developer%'
      OR title ILIKE '%android developer%'
      OR title ILIKE '%cross-platform%'
      OR title ILIKE '%expo developer%'
    THEN 'mobile_engineer'
    WHEN title ILIKE '%fullstack%'
      OR title ILIKE '%full-stack%'
      OR title ILIKE '%full stack%'
    THEN 'fullstack_engineer'
    WHEN title ILIKE '%frontend%'
      OR title ILIKE '%front-end%'
      OR title ILIKE '%front end%'
      OR title ILIKE '%react developer%'
      OR title ILIKE '%vue developer%'
      OR title ILIKE '%angular developer%'
      OR title ILIKE '%javascript developer%'
      OR title ILIKE '%typescript developer%'
      OR title ILIKE '%svelte developer%'
      OR title ILIKE '%nuxt developer%'
      OR title ILIKE '%ui engineer%'
      OR title ILIKE '%web engineer%'
      OR title ILIKE '%web developer%'
      OR title ILIKE '%nextjs developer%'
    THEN 'frontend_engineer'
    WHEN title ILIKE '%devops%'
      OR title ILIKE '%cloud engineer%'
      OR title ILIKE '%site reliability%'
      OR title ~* '(^|[^a-z])sre([^a-z]|$)'
      OR title ILIKE '%reliability engineer%'
      OR title ILIKE '%infrastructure engineer%'
      OR title ILIKE '%systems engineer%'
      OR title ILIKE '%kubernetes engineer%'
      OR title ILIKE '%ci/cd engineer%'
    THEN 'devops_engineer'
    WHEN title ILIKE '%backend%'
      OR title ILIKE '%back-end%'
      OR title ILIKE '%back end%'
      OR title ILIKE '%server-side%'
      OR title ILIKE '%server developer%'
      OR title ILIKE '%api developer%'
      OR title ILIKE '%platform engineer%'
      OR title ILIKE '%software engineer%'
      OR title ILIKE '%rust engineer%'
      OR title ILIKE '%go developer%'
      OR title ILIKE '%python developer%'
      OR title ILIKE '%java developer%'
      OR title ILIKE '%node.js developer%'
      OR title ILIKE '%nodejs developer%'
      OR title ILIKE '%php developer%'
      OR title ILIKE '%ruby developer%'
    THEN 'backend_engineer'
    ELSE NULL
END
"#;

async fn insert_market_company_hiring_again_notifications(
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
                LOWER(BTRIM(company_name)) AS normalized_company_name,
                first_seen_at,
                ({JOB_ROLE_CLASSIFIER_SQL}) AS role_key
            FROM jobs
            WHERE id = ANY($1::text[])
              AND first_seen_at = last_seen_at
              AND is_active = TRUE
        ),
        company_resume_jobs AS (
            SELECT candidate_jobs.*
            FROM candidate_jobs
            WHERE role_key IS NOT NULL
              AND NOT EXISTS (
                  SELECT 1
                  FROM jobs previous
                  WHERE LOWER(BTRIM(previous.company_name)) = candidate_jobs.normalized_company_name
                    AND previous.id NOT IN (SELECT id FROM candidate_jobs)
                    AND previous.first_seen_at < candidate_jobs.first_seen_at
                    AND previous.first_seen_at >= candidate_jobs.first_seen_at - INTERVAL '30 days'
              )
              AND EXISTS (
                  SELECT 1
                  FROM jobs previous
                  WHERE LOWER(BTRIM(previous.company_name)) = candidate_jobs.normalized_company_name
                    AND previous.id NOT IN (SELECT id FROM candidate_jobs)
                    AND previous.first_seen_at < candidate_jobs.first_seen_at - INTERVAL '30 days'
              )
        ),
        profiles_with_roles AS (
            SELECT
                p.id AS profile_id,
                ARRAY_REMOVE(
                    ARRAY(
                        SELECT DISTINCT role_key
                        FROM (
                            SELECT NULLIF(BTRIM(p.primary_role), '') AS role_key
                            UNION ALL
                            SELECT value AS role_key
                            FROM jsonb_array_elements_text(
                                COALESCE(p.search_preferences->'preferred_roles', '[]'::jsonb)
                            ) AS preferred(value)
                        ) roles
                        WHERE role_key IS NOT NULL
                    ),
                    NULL
                ) AS target_roles
            FROM profiles p
            LEFT JOIN notification_preferences np ON np.profile_id = p.id
            WHERE COALESCE(np.market_intelligence_updates, TRUE)
        ),
        matched_resume_events AS (
            SELECT
                p.profile_id,
                j.normalized_company_name,
                j.company_name,
                j.role_key,
                role_labels.role_label,
                MAX(j.first_seen_at) AS resume_at,
                ARRAY_AGG(j.id ORDER BY j.first_seen_at DESC, j.id ASC) AS job_ids,
                (ARRAY_AGG(j.title ORDER BY j.first_seen_at DESC, j.id ASC))[1] AS sample_title
            FROM company_resume_jobs j
            INNER JOIN profiles_with_roles p ON j.role_key = ANY(p.target_roles)
            INNER JOIN (
                VALUES
                    ('frontend_engineer', 'Frontend Engineer'),
                    ('backend_engineer', 'Backend Engineer'),
                    ('fullstack_engineer', 'Fullstack Engineer'),
                    ('mobile_engineer', 'Mobile Engineer'),
                    ('devops_engineer', 'DevOps Engineer'),
                    ('data_engineer', 'Data Engineer'),
                    ('ml_engineer', 'ML Engineer'),
                    ('qa_engineer', 'QA Engineer'),
                    ('product_designer', 'Product Designer'),
                    ('product_manager', 'Product Manager'),
                    ('project_manager', 'Project Manager'),
                    ('tech_lead', 'Tech Lead'),
                    ('engineering_manager', 'Engineering Manager')
            ) AS role_labels(role_key, role_label) ON role_labels.role_key = j.role_key
            GROUP BY p.profile_id, j.normalized_company_name, j.company_name, j.role_key, role_labels.role_label
        ),
        daily_existing AS (
            SELECT
                profile_id,
                COUNT(*)::int AS notification_count
            FROM notifications
            WHERE type = 'market_company_hiring_again'
              AND created_at >= DATE_TRUNC('day', NOW())
              AND created_at < DATE_TRUNC('day', NOW()) + INTERVAL '1 day'
            GROUP BY profile_id
        ),
        ranked AS (
            SELECT
                matched_resume_events.*,
                COALESCE(daily_existing.notification_count, 0) AS existing_count,
                ROW_NUMBER() OVER (
                    PARTITION BY matched_resume_events.profile_id
                    ORDER BY matched_resume_events.resume_at DESC, matched_resume_events.company_name ASC, matched_resume_events.role_key ASC
                ) AS daily_rank
            FROM matched_resume_events
            LEFT JOIN daily_existing ON daily_existing.profile_id = matched_resume_events.profile_id
        )
        INSERT INTO notifications (id, profile_id, type, title, body, payload)
        SELECT
            md5(profile_id || ':market_company_hiring_again:' || normalized_company_name || ':' || role_key || ':' || (resume_at::date)::text),
            profile_id,
            'market_company_hiring_again',
            company_name || ' is hiring again for ' || role_label,
            sample_title || ' is a fresh matching opening after a quiet period at ' || company_name || '.',
            jsonb_build_object(
                'company_name', company_name,
                'normalized_company_name', normalized_company_name,
                'role_id', role_key,
                'role_label', role_label,
                'job_ids', job_ids,
                'resume_at', resume_at
            )
        FROM ranked
        WHERE existing_count + daily_rank <= 3
        ON CONFLICT (id) DO NOTHING
        "#,
    );

    sqlx::query(&query)
        .bind(job_ids)
        .execute(&mut **tx)
        .await
        .map_err(|error| format!("failed to create market company hiring alerts: {error}"))?;

    Ok(())
}

async fn insert_new_job_notifications(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    job_ids: &[String],
) -> Result<(), String> {
    let jobs = fetch_alert_candidate_jobs(tx, job_ids, AlertJobKind::New)
        .await
        .map_err(|error| format!("failed to load new job alert candidates: {error}"))?;
    let profiles = fetch_alert_profiles(tx)
        .await
        .map_err(|error| format!("failed to load alert profiles: {error}"))?;

    for profile in &profiles {
        if !alert_profile_has_signals(&profile.profile) {
            continue;
        }

        for job in &jobs {
            let score = score_job_alert(&profile.profile, &job.job);
            if score.score <= JOB_ALERT_SCORE_THRESHOLD {
                continue;
            }

            insert_job_alert_notification(
                tx,
                &profile.profile_id,
                "new_jobs_found",
                &format!("{} matched your search profile", job.title),
                &format!(
                    "{} at {} matched your current search profile.",
                    job.title, job.company_name
                ),
                &job.job_id,
                score.score,
                &score.matched_signals,
            )
            .await?;
        }
    }

    Ok(())
}

async fn insert_reactivated_job_notifications(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    job_ids: &[String],
) -> Result<(), String> {
    let jobs = fetch_alert_candidate_jobs(tx, job_ids, AlertJobKind::Reactivated)
        .await
        .map_err(|error| format!("failed to load reactivated job alert candidates: {error}"))?;
    let profiles = fetch_alert_profiles(tx)
        .await
        .map_err(|error| format!("failed to load alert profiles: {error}"))?;

    for profile in &profiles {
        if !alert_profile_has_signals(&profile.profile) {
            continue;
        }

        for job in &jobs {
            let score = score_job_alert(&profile.profile, &job.job);
            if score.score <= JOB_ALERT_SCORE_THRESHOLD {
                continue;
            }

            insert_job_alert_notification(
                tx,
                &profile.profile_id,
                "job_reactivated",
                &format!("{} is active again", job.title),
                &format!("{} at {} is active again.", job.title, job.company_name),
                &job.job_id,
                score.score,
                &score.matched_signals,
            )
            .await?;
        }
    }

    Ok(())
}

#[derive(Clone, Copy)]
enum AlertJobKind {
    New,
    Reactivated,
}

#[derive(FromRow)]
struct AlertJobRow {
    id: String,
    title: String,
    company_name: String,
    location: Option<String>,
    remote_type: Option<String>,
    seniority: Option<String>,
    description_text: String,
}

struct AlertJobCandidate {
    job_id: String,
    title: String,
    company_name: String,
    job: AlertJob,
}

#[derive(FromRow)]
struct AlertProfileRow {
    id: String,
    primary_role: Option<String>,
    seniority: Option<String>,
    skills: Json<Vec<String>>,
    keywords: Json<Vec<String>>,
    search_preferences: Option<Json<serde_json::Value>>,
}

struct AlertProfileCandidate {
    profile_id: String,
    profile: AlertProfile,
}

async fn fetch_alert_candidate_jobs(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    job_ids: &[String],
    kind: AlertJobKind,
) -> Result<Vec<AlertJobCandidate>, sqlx::Error> {
    let lifecycle_filter = match kind {
        AlertJobKind::New => "reactivated_at IS NULL",
        AlertJobKind::Reactivated => "reactivated_at IS NOT NULL AND reactivated_at = last_seen_at",
    };
    let query = format!(
        r#"
        SELECT
            id,
            title,
            company_name,
            location,
            remote_type,
            seniority,
            description_text
        FROM jobs
        WHERE id = ANY($1::text[])
          AND duplicate_of IS NULL
          AND is_active = TRUE
          AND {lifecycle_filter}
        "#,
    );

    let rows = sqlx::query_as::<_, AlertJobRow>(&query)
        .bind(job_ids)
        .fetch_all(&mut **tx)
        .await?;

    Ok(rows
        .into_iter()
        .map(|row| AlertJobCandidate {
            job_id: row.id,
            title: row.title.clone(),
            company_name: row.company_name.clone(),
            job: AlertJob {
                title: row.title,
                company_name: row.company_name,
                location: row.location,
                remote_type: row.remote_type,
                seniority: row.seniority,
                description_text: row.description_text,
            },
        })
        .collect())
}

async fn fetch_alert_profiles(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
) -> Result<Vec<AlertProfileCandidate>, sqlx::Error> {
    let rows = sqlx::query_as::<_, AlertProfileRow>(
        r#"
        SELECT
            p.id,
            p.primary_role,
            p.seniority,
            p.skills,
            p.keywords,
            p.search_preferences
        FROM profiles p
        LEFT JOIN notification_preferences np ON np.profile_id = p.id
        WHERE COALESCE(np.new_jobs_matching_profile, TRUE)
        "#,
    )
    .fetch_all(&mut **tx)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| AlertProfileCandidate {
            profile_id: row.id,
            profile: AlertProfile {
                primary_role: row.primary_role,
                seniority: row.seniority,
                skills: row.skills.0,
                keywords: row.keywords.0,
                preferred_roles: preference_array(&row.search_preferences, "preferred_roles"),
                include_keywords: preference_array(&row.search_preferences, "include_keywords"),
                exclude_keywords: preference_array(&row.search_preferences, "exclude_keywords"),
                work_modes: preference_array(&row.search_preferences, "work_modes"),
            },
        })
        .collect())
}

fn preference_array(preferences: &Option<Json<serde_json::Value>>, key: &str) -> Vec<String> {
    preferences
        .as_ref()
        .and_then(|preferences| preferences.0.get(key))
        .and_then(|value| value.as_array())
        .map(|values| {
            values
                .iter()
                .filter_map(|value| value.as_str())
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}

#[allow(clippy::too_many_arguments)]
async fn insert_job_alert_notification(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    profile_id: &str,
    notification_type: &str,
    title: &str,
    body: &str,
    job_id: &str,
    score: u8,
    matched_signals: &[String],
) -> Result<(), String> {
    sqlx::query(
        r#"
        INSERT INTO notifications (id, profile_id, type, title, body, payload)
        VALUES (
            md5($1),
            $2,
            $3,
            $4,
            $5,
            jsonb_build_object(
                'count', 1,
                'job_id', $6::text,
                'job_ids', jsonb_build_array($6::text),
                'score', $7::int,
                'threshold', $8::int,
                'matched_signals', $9::jsonb
            )
        )
        ON CONFLICT (id) DO NOTHING
        "#,
    )
    .bind(format!("{profile_id}:{notification_type}:{job_id}"))
    .bind(profile_id)
    .bind(notification_type)
    .bind(title)
    .bind(body)
    .bind(job_id)
    .bind(i32::from(score))
    .bind(i32::from(JOB_ALERT_SCORE_THRESHOLD))
    .bind(Json(matched_signals.to_vec()))
    .execute(&mut **tx)
    .await
    .map_err(|error| format!("failed to create job alert notification: {error}"))?;

    Ok(())
}
