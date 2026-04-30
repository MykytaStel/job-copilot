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
    let company_velocity_payload = build_company_velocity(pool).await?;
    let freeze_signals_payload = build_freeze_signals(pool).await?;
    let salary_by_seniority_payload = build_salary_by_seniority(pool).await?;
    let region_breakdown_payload = build_region_breakdown(pool).await?;
    let tech_demand_payload = build_tech_demand(pool).await?;

    upsert(pool, snapshot_date, "overview", overview_payload).await?;
    upsert(pool, snapshot_date, "company_stats", company_stats_payload).await?;
    upsert(pool, snapshot_date, "salary_trends", salary_trends_payload).await?;
    upsert(pool, snapshot_date, "role_demand", role_demand_payload).await?;
    upsert(
        pool,
        snapshot_date,
        "company_velocity",
        company_velocity_payload,
    )
    .await?;
    upsert(
        pool,
        snapshot_date,
        "freeze_signals",
        freeze_signals_payload,
    )
    .await?;
    upsert(
        pool,
        snapshot_date,
        "salary_by_seniority",
        salary_by_seniority_payload,
    )
    .await?;
    upsert(
        pool,
        snapshot_date,
        "region_breakdown",
        region_breakdown_payload,
    )
    .await?;
    upsert(pool, snapshot_date, "tech_demand", tech_demand_payload).await?;

    Ok(MarketSnapshotSummary {
        snapshot_date: snapshot_date.format("%Y-%m-%d").to_string(),
        snapshots_written: 9,
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

async fn build_company_velocity(pool: &PgPool) -> Result<serde_json::Value, String> {
    sqlx::query_scalar::<_, serde_json::Value>(
        r#"
        WITH recent_company_jobs AS (
            SELECT
                BTRIM(company_name) AS company,
                LOWER(REGEXP_REPLACE(BTRIM(company_name), '\s+', ' ', 'g')) AS normalized_company,
                first_seen_at
            FROM jobs
            WHERE company_name IS NOT NULL
              AND BTRIM(company_name) <> ''
              AND LOWER(BTRIM(company_name)) NOT IN ('unknown', 'uknonwn', 'unknonwn', 'n/a', 'na', 'none', 'null', '—', '-')
              AND first_seen_at >= NOW() - INTERVAL '30 days'
        ),
        velocity_stats AS (
            SELECT
                MIN(company) AS company,
                COUNT(*)::bigint AS job_count,
                COUNT(*) FILTER (
                    WHERE first_seen_at >= NOW() - INTERVAL '7 days'
                )::bigint AS this_week,
                COUNT(*) FILTER (
                    WHERE first_seen_at >= NOW() - INTERVAL '14 days'
                      AND first_seen_at < NOW() - INTERVAL '7 days'
                )::bigint AS prev_week
            FROM recent_company_jobs
            GROUP BY normalized_company
            HAVING COUNT(*) >= 3
        )
        SELECT COALESCE(
            jsonb_agg(
                jsonb_build_object(
                    'company', company,
                    'job_count', job_count,
                    'trend',
                    CASE
                        WHEN this_week > prev_week THEN 'growing'
                        WHEN this_week < prev_week THEN 'declining'
                        ELSE 'stable'
                    END
                )
                ORDER BY job_count DESC, company ASC
            ),
            '[]'::jsonb
        )
        FROM velocity_stats
        "#,
    )
    .fetch_one(pool)
    .await
    .map_err(|error| format!("failed to build market company velocity snapshot: {error}"))
}

async fn build_freeze_signals(pool: &PgPool) -> Result<serde_json::Value, String> {
    sqlx::query_scalar::<_, serde_json::Value>(
        r#"
        WITH recent_company_jobs AS (
            SELECT
                BTRIM(company_name) AS company,
                LOWER(REGEXP_REPLACE(BTRIM(company_name), '\s+', ' ', 'g')) AS normalized_company,
                first_seen_at
            FROM jobs
            WHERE company_name IS NOT NULL
              AND BTRIM(company_name) <> ''
              AND LOWER(BTRIM(company_name)) NOT IN ('unknown', 'uknonwn', 'unknonwn', 'n/a', 'na', 'none', 'null', '—', '-')
              AND first_seen_at >= NOW() - INTERVAL '60 days'
        ),
        company_stats AS (
            SELECT
                MIN(company) AS company,
                MAX(first_seen_at) AS last_posted_at,
                COUNT(*)::bigint AS historical_count,
                COUNT(*) FILTER (
                    WHERE first_seen_at >= NOW() - INTERVAL '14 days'
                )::bigint AS recent_count
            FROM recent_company_jobs
            GROUP BY normalized_company
        )
        SELECT COALESCE(
            jsonb_agg(
                jsonb_build_object(
                    'company', company,
                    'last_posted_at', TO_CHAR(last_posted_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS"Z"'),
                    'days_since_last_post', GREATEST(
                        0,
                        FLOOR(EXTRACT(EPOCH FROM (NOW() - last_posted_at)) / 86400)
                    )::integer,
                    'historical_count', historical_count
                )
                ORDER BY GREATEST(0, FLOOR(EXTRACT(EPOCH FROM (NOW() - last_posted_at)) / 86400)) DESC, company ASC
            ),
            '[]'::jsonb
        )
        FROM company_stats
        WHERE historical_count >= 5
          AND recent_count = 0
        "#,
    )
    .fetch_one(pool)
    .await
    .map_err(|error| format!("failed to build market freeze signals snapshot: {error}"))
}

async fn build_salary_by_seniority(pool: &PgPool) -> Result<serde_json::Value, String> {
    sqlx::query_scalar::<_, serde_json::Value>(
        r#"
        WITH normalized_jobs AS (
            SELECT
                CASE
                    WHEN LOWER(TRIM(seniority)) IN ('junior', 'jr', 'junior/middle', 'junior-middle')
                        THEN 'junior'
                    WHEN LOWER(TRIM(seniority)) IN ('middle', 'mid', 'regular', 'intermediate')
                        THEN 'mid'
                    WHEN LOWER(TRIM(seniority)) IN ('senior', 'sr')
                        THEN 'senior'
                    WHEN LOWER(TRIM(seniority)) IN ('lead', 'staff', 'lead/staff', 'principal', 'architect')
                        THEN 'lead_staff'
                    ELSE NULL
                END AS seniority,
                CASE UPPER(TRIM(salary_currency))
                    WHEN 'USD' THEN salary_min::numeric
                    WHEN 'EUR' THEN salary_min::numeric * 1.1
                    WHEN 'UAH' THEN salary_min::numeric * 0.024
                    ELSE NULL
                END AS salary_usd_min,
                CASE UPPER(TRIM(salary_currency))
                    WHEN 'USD' THEN salary_max::numeric
                    WHEN 'EUR' THEN salary_max::numeric * 1.1
                    WHEN 'UAH' THEN salary_max::numeric * 0.024
                    ELSE NULL
                END AS salary_usd_max
            FROM jobs
            WHERE is_active
              AND last_seen_at >= NOW() - INTERVAL '60 days'
              AND seniority IS NOT NULL
              AND BTRIM(seniority) <> ''
              AND salary_min IS NOT NULL
              AND salary_max IS NOT NULL
              AND salary_min > 0
              AND salary_max > 0
              AND salary_max >= salary_min
              AND salary_currency IS NOT NULL
              AND UPPER(TRIM(salary_currency)) IN ('USD', 'EUR', 'UAH')
        ),
        salary_by_seniority AS (
            SELECT
                seniority,
                ROUND(PERCENTILE_CONT(0.50) WITHIN GROUP (ORDER BY salary_usd_min))::integer AS median_min,
                ROUND(PERCENTILE_CONT(0.50) WITHIN GROUP (ORDER BY salary_usd_max))::integer AS median_max,
                COUNT(*)::bigint AS sample_size
            FROM normalized_jobs
            WHERE seniority IS NOT NULL
              AND salary_usd_min IS NOT NULL
              AND salary_usd_max IS NOT NULL
            GROUP BY seniority
            HAVING COUNT(*) >= 10
        )
        SELECT COALESCE(
            jsonb_agg(
                jsonb_build_object(
                    'seniority', seniority,
                    'median_min', median_min,
                    'median_max', median_max,
                    'sample_size', sample_size
                )
                ORDER BY
                    CASE seniority
                        WHEN 'junior' THEN 1
                        WHEN 'mid' THEN 2
                        WHEN 'senior' THEN 3
                        WHEN 'lead_staff' THEN 4
                        ELSE 5
                    END,
                    seniority ASC
            ),
            '[]'::jsonb
        )
        FROM salary_by_seniority
        "#,
    )
    .fetch_one(pool)
    .await
    .map_err(|error| format!("failed to build market salary by seniority snapshot: {error}"))
}

async fn build_region_breakdown(pool: &PgPool) -> Result<serde_json::Value, String> {
    let query = format!(
        r#"
        WITH region_groups(region_rank, region) AS (
            VALUES
                (1, 'Remote'),
                (2, 'Kyiv'),
                (3, 'Lviv'),
                (4, 'Other Ukraine'),
                (5, 'Abroad/Relocation')
        ),
        classified_jobs AS (
            SELECT
                CASE
                    WHEN LOWER(BTRIM(COALESCE(remote_type, ''))) = 'remote'
                    THEN 'Remote'
                    WHEN COALESCE(location, '') ILIKE '%kyiv%'
                      OR COALESCE(location, '') ILIKE '%київ%'
                    THEN 'Kyiv'
                    WHEN COALESCE(location, '') ILIKE '%lviv%'
                      OR COALESCE(location, '') ILIKE '%львів%'
                    THEN 'Lviv'
                    WHEN COALESCE(location, '') ~* '(poland|warsaw|krakow|germany|berlin|munich|spain|barcelona|madrid|portugal|lisbon|netherlands|amsterdam|uk|united kingdom|london|ireland|dublin|czech|prague|romania|bucharest|bulgaria|sofia|estonia|tallinn|latvia|riga|lithuania|vilnius|usa|united states|canada|relocation|relocate|abroad|польща|німеччина|германія|іспанія|португалія|чехія|румунія|болгарія|естонія|латвія|литва|сша|канада|релокац)'
                    THEN 'Abroad/Relocation'
                    ELSE 'Other Ukraine'
                END AS region,
                {role_group_classifier} AS role_group
            FROM jobs
            WHERE is_active
        ),
        counts AS (
            SELECT region, COUNT(*)::bigint AS job_count
            FROM classified_jobs
            GROUP BY region
        ),
        role_counts AS (
            SELECT region, role_group, COUNT(*)::bigint AS role_count
            FROM classified_jobs
            WHERE role_group IS NOT NULL
            GROUP BY region, role_group
        ),
        ranked_roles AS (
            SELECT
                region,
                role_group,
                ROW_NUMBER() OVER (
                    PARTITION BY region
                    ORDER BY role_count DESC, role_group ASC
                ) AS role_rank
            FROM role_counts
        ),
        top_roles AS (
            SELECT
                region,
                ARRAY_AGG(role_group ORDER BY role_rank)::text[] AS top_roles
            FROM ranked_roles
            WHERE role_rank <= 3
            GROUP BY region
        )
        SELECT COALESCE(
            jsonb_agg(
                jsonb_build_object(
                    'region', region_groups.region,
                    'job_count', COALESCE(counts.job_count, 0)::bigint,
                    'top_roles', COALESCE(top_roles.top_roles, ARRAY[]::text[])
                )
                ORDER BY region_groups.region_rank ASC
            ),
            '[]'::jsonb
        )
        FROM region_groups
        LEFT JOIN counts USING (region)
        LEFT JOIN top_roles USING (region)
        "#,
        role_group_classifier = MARKET_ROLE_GROUP_CLASSIFIER_CASE_SQL,
    );

    sqlx::query_scalar::<_, serde_json::Value>(&query)
        .fetch_one(pool)
        .await
        .map_err(|error| format!("failed to build market region breakdown snapshot: {error}"))
}

async fn build_tech_demand(pool: &PgPool) -> Result<serde_json::Value, String> {
    sqlx::query_scalar::<_, serde_json::Value>(
        r#"
        WITH tech_skills(skill, pattern) AS (
            VALUES
                ('React', '\mreact\M'),
                ('Vue', '\mvue\M'),
                ('Angular', '\mangular\M'),
                ('TypeScript', '\mtypescript\M|\mts\M'),
                ('JavaScript', '\mjavascript\M|\mjs\M'),
                ('Node.js', '\mnode[.]?js\M'),
                ('Python', '\mpython\M'),
                ('Rust', '\mrust\M'),
                ('Go', '\mgo\M|\mgolang\M'),
                ('Java', '\mjava\M'),
                ('Kotlin', '\mkotlin\M'),
                ('PostgreSQL', '\mpostgresql\M|\mpostgres\M'),
                ('Redis', '\mredis\M'),
                ('Docker', '\mdocker\M'),
                ('Kubernetes', '\mkubernetes\M|\mk8s\M'),
                ('AWS', '\maws\M|amazon web services'),
                ('GCP', '\mgcp\M|google cloud'),
                ('Next.js', '\mnext[.]?js\M'),
                ('GraphQL', '\mgraphql\M'),
                ('FastAPI', '\mfastapi\M'),
                ('Django', '\mdjango\M'),
                ('Spring Boot', '\mspring[[:space:]]+boot\M')
        ),
        active_period_jobs AS (
            SELECT title || ' ' || description_text AS searchable_text
            FROM jobs
            WHERE is_active
              AND last_seen_at >= NOW() - INTERVAL '30 days'
        ),
        total AS (
            SELECT COUNT(*)::bigint AS active_jobs FROM active_period_jobs
        ),
        skill_counts AS (
            SELECT
                tech_skills.skill,
                COUNT(active_period_jobs.searchable_text)::bigint AS job_count,
                CASE
                    WHEN total.active_jobs > 0 THEN
                        COUNT(active_period_jobs.searchable_text)::double precision
                        / total.active_jobs::double precision * 100.0
                    ELSE 0.0
                END AS percentage
            FROM tech_skills
            CROSS JOIN total
            LEFT JOIN active_period_jobs
                ON active_period_jobs.searchable_text ~* tech_skills.pattern
            GROUP BY tech_skills.skill, total.active_jobs
            HAVING COUNT(active_period_jobs.searchable_text) > 0
        )
        SELECT COALESCE(
            jsonb_agg(
                jsonb_build_object(
                    'skill', skill,
                    'job_count', job_count,
                    'percentage', percentage
                )
                ORDER BY job_count DESC, skill ASC
            ),
            '[]'::jsonb
        )
        FROM skill_counts
        "#,
    )
    .fetch_one(pool)
    .await
    .map_err(|error| format!("failed to build market tech demand snapshot: {error}"))
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
