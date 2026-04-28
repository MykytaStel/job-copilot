use sqlx::PgPool;
use sqlx::types::Json;

use super::MarketSnapshotSummary;
use super::market_role_heuristics::{
    MARKET_ROLE_GROUP_CLASSIFIER_CASE_SQL, MARKET_ROLE_GROUP_ORDER_ARRAY_SQL,
    MARKET_ROLE_GROUPS_VALUES_SQL,
};

pub(super) async fn run_refresh(pool: &PgPool) -> Result<MarketSnapshotSummary, String> {
    let snapshot_date = chrono::Utc::now().date_naive();
    let overview_payload = build_overview(pool).await?;
    let company_stats_payload = build_company_stats(pool).await?;
    let salary_trends_payload = build_salary_trends(pool).await?;
    let role_demand_payload = build_role_demand(pool).await?;

    upsert(pool, snapshot_date, "overview", overview_payload).await?;
    upsert(pool, snapshot_date, "company_stats", company_stats_payload).await?;
    upsert(pool, snapshot_date, "salary_trends", salary_trends_payload).await?;
    upsert(pool, snapshot_date, "role_demand", role_demand_payload).await?;

    Ok(MarketSnapshotSummary {
        snapshot_date: snapshot_date.format("%Y-%m-%d").to_string(),
        snapshots_written: 4,
    })
}

async fn build_overview(pool: &PgPool) -> Result<serde_json::Value, String> {
    sqlx::query_scalar::<_, serde_json::Value>(
        r#"
        SELECT jsonb_build_object(
            'new_jobs_this_week',
            COUNT(*) FILTER (
                WHERE is_active AND first_seen_at >= NOW() - INTERVAL '7 days'
            )::bigint,
            'active_companies_count',
            COUNT(DISTINCT company_name) FILTER (
                WHERE is_active
                  AND company_name IS NOT NULL
                  AND BTRIM(company_name) <> ''
                  AND LOWER(BTRIM(company_name)) NOT IN ('unknown', 'uknonwn', 'unknonwn', 'n/a', 'na', 'none', 'null', '—', '-')
            )::bigint,
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

async fn build_company_stats(pool: &PgPool) -> Result<serde_json::Value, String> {
    let query = format!(
        r#"
        WITH active_company_jobs AS (
            SELECT
                jobs.id,
                jobs.title,
                jobs.company_name,
                LOWER(REGEXP_REPLACE(BTRIM(jobs.company_name), '\s+', ' ', 'g')) AS normalized_company_name,
                jobs.first_seen_at,
                jobs.last_seen_at,
                {role_group_classifier} AS role_group
            FROM jobs
            WHERE company_name IS NOT NULL
              AND BTRIM(company_name) <> ''
              AND LOWER(BTRIM(company_name)) NOT IN ('unknown', 'uknonwn', 'unknonwn', 'n/a', 'na', 'none', 'null', '—', '-')
              AND jobs.is_active
        ),
        company_stats AS (
            SELECT
                company_name,
                normalized_company_name,
                COUNT(*)::bigint AS active_jobs,
                COUNT(*) FILTER (
                    WHERE first_seen_at >= NOW() - INTERVAL '7 days'
                )::bigint AS this_week,
                COUNT(*) FILTER (
                    WHERE first_seen_at >= NOW() - INTERVAL '14 days'
                      AND first_seen_at < NOW() - INTERVAL '7 days'
                )::bigint AS prev_week,
                ARRAY_REMOVE(ARRAY_AGG(DISTINCT role_group), NULL)::text[] AS top_role_groups,
                (ARRAY_AGG(id ORDER BY last_seen_at DESC))[1:5]::text[] AS latest_job_ids
            FROM active_company_jobs
            GROUP BY company_name, normalized_company_name
        )
        SELECT COALESCE(
            jsonb_agg(
                jsonb_build_object(
                    'company_name', company_name,
                    'normalized_company_name', normalized_company_name,
                    'active_jobs', active_jobs,
                    'this_week', this_week,
                    'prev_week', prev_week,
                    'sources', COALESCE(sources.sources, ARRAY[]::text[]),
                    'top_role_groups', top_role_groups,
                    'latest_job_ids', latest_job_ids,
                    'data_quality_flags', ARRAY[]::text[]
                )
                ORDER BY active_jobs DESC, company_name ASC
            ),
            '[]'::jsonb
        )
        FROM company_stats
        LEFT JOIN LATERAL (
            SELECT ARRAY_AGG(DISTINCT variants.source ORDER BY variants.source)::text[] AS sources
            FROM job_variants variants
            WHERE variants.job_id = ANY(company_stats.latest_job_ids)
        ) sources ON TRUE
        "#,
        role_group_classifier = MARKET_ROLE_GROUP_CLASSIFIER_CASE_SQL,
    );

    sqlx::query_scalar::<_, serde_json::Value>(&query)
        .fetch_one(pool)
        .await
        .map_err(|error| format!("failed to build market company stats snapshot: {error}"))
}

async fn build_salary_trends(pool: &PgPool) -> Result<serde_json::Value, String> {
    sqlx::query_scalar::<_, serde_json::Value>(
        r#"
        WITH filtered_jobs AS (
            SELECT
                LOWER(TRIM(seniority)) AS seniority,
                COALESCE(NULLIF(UPPER(TRIM(salary_currency)), ''), 'UNKNOWN') AS salary_currency,
                ROUND((salary_min + COALESCE(salary_max, salary_min))::numeric / 2.0)::integer AS salary_midpoint
            FROM jobs
            WHERE is_active
              AND salary_min IS NOT NULL
              AND salary_min > 0
              AND (salary_max IS NULL OR salary_max >= salary_min)
              AND COALESCE(NULLIF(UPPER(TRIM(salary_currency)), ''), 'UNKNOWN') <> 'UNKNOWN'
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
                salary_currency AS currency,
                ROUND(PERCENTILE_CONT(0.25) WITHIN GROUP (ORDER BY salary_midpoint))::integer AS p25,
                ROUND(PERCENTILE_CONT(0.50) WITHIN GROUP (ORDER BY salary_midpoint))::integer AS median,
                ROUND(PERCENTILE_CONT(0.75) WITHIN GROUP (ORDER BY salary_midpoint))::integer AS p75,
                COUNT(*)::bigint AS sample_count
            FROM dominant_jobs
            GROUP BY seniority, salary_currency
        )
        SELECT COALESCE(
            jsonb_agg(
                jsonb_build_object(
                    'seniority', seniority,
                    'currency', currency,
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

async fn build_role_demand(pool: &PgPool) -> Result<serde_json::Value, String> {
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

async fn upsert(
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
