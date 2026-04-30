use crate::db::repositories::RepositoryError;
use crate::domain::job::model::JobView;
use crate::domain::market::model::{
    MarketCompanyDetail, MarketCompanyEntry, MarketCompanyVelocityEntry,
    MarketCompanyVelocityPoint, MarketCompanyVelocityTrend, MarketFreezeSignalEntry,
    MarketOverview, MarketRegionDemandEntry, MarketRoleDemandEntry, MarketSalaryBySeniorityEntry,
    MarketSalaryTrend, MarketSource, MarketTechDemandEntry, MarketTrendDirection,
};
use sqlx::FromRow;

mod market_role_heuristics {
    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../shared/rust/market_role_heuristics.rs"
    ));
}

use super::JobsRepository;
use super::queries::job_view_query;
use super::rows::JobViewRow;
use market_role_heuristics::{
    MARKET_ROLE_GROUP_CLASSIFIER_CASE_SQL, MARKET_ROLE_GROUP_ORDER_ARRAY_SQL,
    MARKET_ROLE_GROUPS_VALUES_SQL,
};

fn has_usable_company_name(company_name: &str) -> bool {
    let normalized = company_name
        .trim()
        .to_lowercase()
        .replace([' ', '.', '_', '-'], "");

    !normalized.is_empty()
        && !matches!(
            normalized.as_str(),
            "unknown" | "uknonwn" | "unknonwn" | "na" | "n/a" | "none" | "null" | "—" | "-"
        )
}

fn normalize_company_name(company_name: &str) -> String {
    company_name
        .trim()
        .to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn company_slug_sql(expression: &str) -> String {
    format!(
        "LOWER(REGEXP_REPLACE(REGEXP_REPLACE(BTRIM({expression}), '[^[:alnum:]]+', '-', 'g'), '(^-+|-+$)', '', 'g'))"
    )
}

impl JobsRepository {
    async fn fetch_fresh_snapshot(
        pool: &sqlx::PgPool,
        snapshot_type: &str,
    ) -> Result<Option<serde_json::Value>, RepositoryError> {
        #[derive(FromRow)]
        struct SnapshotRow {
            payload: serde_json::Value,
        }

        let row = sqlx::query_as::<_, SnapshotRow>(
            r#"
            SELECT payload
            FROM market_snapshots
            WHERE snapshot_type = $1
              AND created_at >= NOW() - INTERVAL '24 hours'
            ORDER BY created_at DESC
            LIMIT 1
            "#,
        )
        .bind(snapshot_type)
        .fetch_optional(pool)
        .await?;

        Ok(row.map(|r| r.payload))
    }

    pub async fn market_overview(&self) -> Result<(MarketOverview, MarketSource), RepositoryError> {
        let Some(pool) = self.database.pool() else {
            return Err(RepositoryError::DatabaseDisabled);
        };

        if let Some(payload) = Self::fetch_fresh_snapshot(pool, "overview").await?
            && let Ok(overview) = serde_json::from_value::<MarketOverview>(payload)
        {
            return Ok((overview, MarketSource::Snapshot));
        }

        tracing::warn!("market_overview: no fresh snapshot, falling back to live jobs query");

        #[derive(FromRow)]
        struct MarketOverviewRow {
            new_jobs_this_week: i64,
            active_companies_count: i64,
            active_jobs_count: i64,
            remote_jobs_count: i64,
        }

        let row = sqlx::query_as::<_, MarketOverviewRow>(
            r#"
            SELECT
                COUNT(*) FILTER (
                    WHERE is_active AND first_seen_at >= NOW() - INTERVAL '7 days'
                )::bigint AS new_jobs_this_week,
                COUNT(DISTINCT company_name) FILTER (
                    WHERE is_active
                      AND company_name IS NOT NULL
                      AND BTRIM(company_name) <> ''
                      AND LOWER(BTRIM(company_name)) NOT IN ('unknown', 'uknonwn', 'unknonwn', 'n/a', 'na', 'none', 'null', '—', '-')
                )::bigint AS active_companies_count,
                COUNT(*) FILTER (WHERE is_active)::bigint AS active_jobs_count,
                COUNT(*) FILTER (
                    WHERE is_active AND LOWER(remote_type) LIKE '%remote%'
                )::bigint AS remote_jobs_count
            FROM jobs
            "#,
        )
        .fetch_one(pool)
        .await?;

        let remote_percentage = if row.active_jobs_count > 0 {
            row.remote_jobs_count as f64 / row.active_jobs_count as f64 * 100.0
        } else {
            0.0
        };

        Ok((
            MarketOverview {
                new_jobs_this_week: row.new_jobs_this_week,
                active_companies_count: row.active_companies_count,
                active_jobs_count: row.active_jobs_count,
                remote_percentage,
            },
            MarketSource::Live,
        ))
    }

    pub async fn market_companies(
        &self,
        limit: i64,
    ) -> Result<(Vec<MarketCompanyEntry>, MarketSource), RepositoryError> {
        let Some(pool) = self.database.pool() else {
            return Err(RepositoryError::DatabaseDisabled);
        };

        if let Some(payload) = Self::fetch_fresh_snapshot(pool, "company_stats").await?
            && let Ok(mut entries) = serde_json::from_value::<Vec<MarketCompanyEntry>>(payload)
        {
            entries.retain(|entry| has_usable_company_name(&entry.company_name));
            for entry in &mut entries {
                if entry.normalized_company_name.trim().is_empty() {
                    entry.normalized_company_name = normalize_company_name(&entry.company_name);
                }
            }
            entries.truncate(limit as usize);
            return Ok((entries, MarketSource::Snapshot));
        }

        tracing::warn!("market_companies: no fresh snapshot, falling back to live jobs query");

        #[derive(FromRow)]
        struct MarketCompanyRow {
            company_name: String,
            normalized_company_name: String,
            active_jobs: i64,
            this_week: i64,
            prev_week: i64,
            sources: Vec<String>,
            top_role_groups: Vec<String>,
            latest_job_ids: Vec<String>,
        }

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
                WHERE jobs.company_name IS NOT NULL
                  AND BTRIM(jobs.company_name) <> ''
                  AND LOWER(BTRIM(jobs.company_name)) NOT IN ('unknown', 'uknonwn', 'unknonwn', 'n/a', 'na', 'none', 'null', '—', '-')
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
            SELECT
                company_stats.company_name,
                company_stats.normalized_company_name,
                company_stats.active_jobs,
                company_stats.this_week,
                company_stats.prev_week,
                COALESCE(sources.sources, ARRAY[]::text[]) AS sources,
                company_stats.top_role_groups,
                company_stats.latest_job_ids
            FROM company_stats
            LEFT JOIN LATERAL (
                SELECT ARRAY_AGG(DISTINCT variants.source ORDER BY variants.source)::text[] AS sources
                FROM job_variants variants
                WHERE variants.job_id = ANY(company_stats.latest_job_ids)
            ) sources ON TRUE
            ORDER BY company_stats.active_jobs DESC, company_stats.company_name ASC
            LIMIT $1
            "#,
            role_group_classifier = MARKET_ROLE_GROUP_CLASSIFIER_CASE_SQL,
        );

        let rows = sqlx::query_as::<_, MarketCompanyRow>(&query)
            .bind(limit)
            .fetch_all(pool)
            .await?;

        Ok((
            rows.into_iter()
                .map(|row| MarketCompanyEntry {
                    company_name: row.company_name,
                    normalized_company_name: row.normalized_company_name,
                    active_jobs: row.active_jobs,
                    this_week: row.this_week,
                    prev_week: row.prev_week,
                    sources: row.sources,
                    top_role_groups: row.top_role_groups,
                    latest_job_ids: row.latest_job_ids,
                    data_quality_flags: Vec::new(),
                })
                .collect(),
            MarketSource::Live,
        ))
    }

    pub async fn market_salary_trend(
        &self,
        seniority: &str,
    ) -> Result<(Option<MarketSalaryTrend>, MarketSource), RepositoryError> {
        let (trends, source) = self.fetch_market_salary_trends(Some(seniority)).await?;
        Ok((trends.into_iter().next(), source))
    }

    pub async fn market_company_detail(
        &self,
        company_slug: &str,
    ) -> Result<Option<MarketCompanyDetail>, RepositoryError> {
        let Some(pool) = self.database.pool() else {
            return Err(RepositoryError::DatabaseDisabled);
        };

        let company_slug = company_slug.trim().to_lowercase();
        if company_slug.is_empty() {
            return Ok(None);
        }

        #[derive(FromRow)]
        struct MarketCompanyDetailStatsRow {
            company_name: String,
            normalized_company_name: String,
            total_jobs: i64,
            active_jobs: i64,
            avg_salary: Option<i32>,
        }

        let slug_expr = company_slug_sql("company_name");
        let stats_query = format!(
            r#"
            SELECT
                MIN(BTRIM(company_name)) AS company_name,
                LOWER(REGEXP_REPLACE(BTRIM(MIN(company_name)), '\s+', ' ', 'g')) AS normalized_company_name,
                COUNT(*)::bigint AS total_jobs,
                COUNT(*) FILTER (WHERE is_active)::bigint AS active_jobs,
                ROUND(AVG(
                    CASE
                        WHEN is_active AND salary_min IS NOT NULL AND salary_max IS NOT NULL
                            THEN (salary_min + salary_max)::numeric / 2
                        WHEN is_active AND salary_min IS NOT NULL
                            THEN salary_min::numeric
                        WHEN is_active AND salary_max IS NOT NULL
                            THEN salary_max::numeric
                        ELSE NULL
                    END
                ))::int AS avg_salary
            FROM jobs
            WHERE duplicate_of IS NULL
              AND company_name IS NOT NULL
              AND BTRIM(company_name) <> ''
              AND {slug_expr} = $1
            GROUP BY {slug_expr}
            "#
        );

        let Some(stats) = sqlx::query_as::<_, MarketCompanyDetailStatsRow>(&stats_query)
            .bind(&company_slug)
            .fetch_optional(pool)
            .await?
        else {
            return Ok(None);
        };

        #[derive(FromRow)]
        struct VelocityRow {
            date: String,
            job_count: i64,
        }

        let velocity_query = format!(
            r#"
            WITH days AS (
                SELECT generate_series(
                    CURRENT_DATE - INTERVAL '6 days',
                    CURRENT_DATE,
                    INTERVAL '1 day'
                )::date AS day
            ),
            company_jobs AS (
                SELECT first_seen_at::date AS day
                FROM jobs
                WHERE duplicate_of IS NULL
                  AND is_active
                  AND company_name IS NOT NULL
                  AND BTRIM(company_name) <> ''
                  AND {slug_expr} = $1
                  AND first_seen_at >= CURRENT_DATE - INTERVAL '6 days'
            )
            SELECT
                days.day::text AS date,
                COUNT(company_jobs.day)::bigint AS job_count
            FROM days
            LEFT JOIN company_jobs ON company_jobs.day = days.day
            GROUP BY days.day
            ORDER BY days.day ASC
            "#
        );

        let velocity = sqlx::query_as::<_, VelocityRow>(&velocity_query)
            .bind(&company_slug)
            .fetch_all(pool)
            .await?
            .into_iter()
            .map(|row| MarketCompanyVelocityPoint {
                date: row.date,
                job_count: row.job_count,
            })
            .collect();

        let jobs_where = format!(
            "WHERE jobs.duplicate_of IS NULL AND jobs.is_active = TRUE AND {} = $1",
            company_slug_sql("jobs.company_name")
        );
        let jobs_query = job_view_query(Some(&jobs_where), Some("LIMIT 100"));

        let active_job_views = sqlx::query_as::<_, JobViewRow>(jobs_query.as_str())
            .bind(&company_slug)
            .fetch_all(pool)
            .await?
            .into_iter()
            .map(JobView::from)
            .collect();

        Ok(Some(MarketCompanyDetail {
            company_name: stats.company_name,
            normalized_company_name: stats.normalized_company_name,
            total_jobs: stats.total_jobs,
            active_jobs: stats.active_jobs,
            avg_salary: stats.avg_salary,
            velocity,
            active_job_views,
        }))
    }

    pub async fn market_company_velocity(
        &self,
    ) -> Result<(Vec<MarketCompanyVelocityEntry>, MarketSource), RepositoryError> {
        let Some(pool) = self.database.pool() else {
            return Err(RepositoryError::DatabaseDisabled);
        };

        #[derive(FromRow)]
        struct MarketCompanyVelocityRow {
            company: String,
            job_count: i64,
            this_week: i64,
            prev_week: i64,
        }

        let rows = sqlx::query_as::<_, MarketCompanyVelocityRow>(
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
            )
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
            ORDER BY job_count DESC, company ASC
            LIMIT 10
            "#,
        )
        .fetch_all(pool)
        .await?;

        Ok((
            rows.into_iter()
                .map(|row| MarketCompanyVelocityEntry {
                    company: row.company,
                    job_count: row.job_count,
                    trend: compare_company_velocity(row.this_week, row.prev_week),
                })
                .collect(),
            MarketSource::Live,
        ))
    }

    pub async fn market_freeze_signals(
        &self,
    ) -> Result<(Vec<MarketFreezeSignalEntry>, MarketSource), RepositoryError> {
        let Some(pool) = self.database.pool() else {
            return Err(RepositoryError::DatabaseDisabled);
        };

        #[derive(FromRow)]
        struct MarketFreezeSignalRow {
            company: String,
            last_posted_at: String,
            days_since_last_post: i32,
            historical_count: i64,
        }

        let rows = sqlx::query_as::<_, MarketFreezeSignalRow>(
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
            SELECT
                company,
                TO_CHAR(last_posted_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS"Z"') AS last_posted_at,
                GREATEST(
                    0,
                    FLOOR(EXTRACT(EPOCH FROM (NOW() - last_posted_at)) / 86400)
                )::integer AS days_since_last_post,
                historical_count
            FROM company_stats
            WHERE historical_count >= 5
              AND recent_count = 0
            ORDER BY days_since_last_post DESC, company ASC
            LIMIT 10
            "#,
        )
        .fetch_all(pool)
        .await?;

        Ok((
            rows.into_iter()
                .map(|row| MarketFreezeSignalEntry {
                    company: row.company,
                    last_posted_at: row.last_posted_at,
                    days_since_last_post: row.days_since_last_post.max(0) as u32,
                    historical_count: row.historical_count.max(0) as u32,
                })
                .collect(),
            MarketSource::Live,
        ))
    }

    pub async fn market_salary_trends(
        &self,
    ) -> Result<(Vec<MarketSalaryTrend>, MarketSource), RepositoryError> {
        self.fetch_market_salary_trends(None).await
    }

    pub async fn market_salary_by_seniority(
        &self,
    ) -> Result<(Vec<MarketSalaryBySeniorityEntry>, MarketSource), RepositoryError> {
        let Some(pool) = self.database.pool() else {
            return Err(RepositoryError::DatabaseDisabled);
        };

        #[derive(FromRow)]
        struct MarketSalaryBySeniorityRow {
            seniority: String,
            median_min: i32,
            median_max: i32,
            sample_size: i64,
        }

        let rows = sqlx::query_as::<_, MarketSalaryBySeniorityRow>(
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
            )
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
            ORDER BY
                CASE seniority
                    WHEN 'junior' THEN 1
                    WHEN 'mid' THEN 2
                    WHEN 'senior' THEN 3
                    WHEN 'lead_staff' THEN 4
                    ELSE 5
                END,
                seniority ASC
            "#,
        )
        .fetch_all(pool)
        .await?;

        Ok((
            rows.into_iter()
                .map(|row| MarketSalaryBySeniorityEntry {
                    seniority: row.seniority,
                    median_min: row.median_min.max(0) as u32,
                    median_max: row.median_max.max(0) as u32,
                    sample_size: row.sample_size.max(0) as u32,
                })
                .collect(),
            MarketSource::Live,
        ))
    }

    async fn fetch_market_salary_trends(
        &self,
        seniority: Option<&str>,
    ) -> Result<(Vec<MarketSalaryTrend>, MarketSource), RepositoryError> {
        let Some(pool) = self.database.pool() else {
            return Err(RepositoryError::DatabaseDisabled);
        };

        if let Some(payload) = Self::fetch_fresh_snapshot(pool, "salary_trends").await?
            && let Ok(mut trends) = serde_json::from_value::<Vec<MarketSalaryTrend>>(payload)
        {
            if let Some(filter) = seniority {
                trends.retain(|t| t.seniority == filter);
            }
            trends.retain(|t| t.currency.trim().to_uppercase() != "UNKNOWN");
            if !trends.is_empty() {
                return Ok((trends, MarketSource::Snapshot));
            }
        }

        tracing::warn!("market_salary_trends: no fresh snapshot, falling back to live jobs query");

        #[derive(FromRow)]
        struct MarketSalaryTrendRow {
            seniority: String,
            currency: String,
            p25: i32,
            median: i32,
            p75: i32,
            sample_count: i64,
        }

        let rows = sqlx::query_as::<_, MarketSalaryTrendRow>(
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
                  AND ($1::text IS NULL OR LOWER(TRIM(seniority)) = $1)
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
            )
            SELECT
                seniority,
                salary_currency AS currency,
                ROUND(PERCENTILE_CONT(0.25) WITHIN GROUP (ORDER BY salary_midpoint))::integer AS p25,
                ROUND(PERCENTILE_CONT(0.50) WITHIN GROUP (ORDER BY salary_midpoint))::integer AS median,
                ROUND(PERCENTILE_CONT(0.75) WITHIN GROUP (ORDER BY salary_midpoint))::integer AS p75,
                COUNT(*)::bigint AS sample_count
            FROM dominant_jobs
            GROUP BY seniority, salary_currency
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
            "#,
        )
        .bind(seniority.map(|value| value.trim().to_lowercase()))
        .fetch_all(pool)
        .await?;

        Ok((
            rows.into_iter()
                .map(|row| MarketSalaryTrend {
                    seniority: row.seniority,
                    currency: row.currency,
                    p25: row.p25,
                    median: row.median,
                    p75: row.p75,
                    sample_count: row.sample_count,
                })
                .collect(),
            MarketSource::Live,
        ))
    }

    pub async fn market_role_demand(
        &self,
        period_days: i32,
    ) -> Result<(Vec<MarketRoleDemandEntry>, MarketSource), RepositoryError> {
        let Some(pool) = self.database.pool() else {
            return Err(RepositoryError::DatabaseDisabled);
        };

        if let Some(payload) = Self::fetch_fresh_snapshot(pool, "role_demand").await?
            && let Ok(entries) = serde_json::from_value::<Vec<MarketRoleDemandEntry>>(payload)
        {
            return Ok((entries, MarketSource::Snapshot));
        }

        tracing::warn!("market_role_demand: no fresh snapshot, falling back to live jobs query");

        #[derive(FromRow)]
        struct MarketRoleDemandRow {
            role_group: String,
            this_period: i64,
            prev_period: i64,
        }

        // Title-heuristic classifier until market snapshots are populated from role-aware aggregates.
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
                        WHERE first_seen_at >= NOW() - make_interval(days => CAST($1 AS integer))
                    )::bigint AS this_period,
                    COUNT(*) FILTER (
                        WHERE first_seen_at >= NOW() - make_interval(days => CAST($1 * 2 AS integer))
                          AND first_seen_at < NOW() - make_interval(days => CAST($1 AS integer))
                    )::bigint AS prev_period
                FROM classified_jobs
                WHERE role_group IS NOT NULL
                GROUP BY role_group
            )
            SELECT
                role_groups.role_group,
                COALESCE(counts.this_period, 0)::bigint AS this_period,
                COALESCE(counts.prev_period, 0)::bigint AS prev_period
            FROM role_groups
            LEFT JOIN counts USING (role_group)
            ORDER BY ARRAY_POSITION(
                {role_group_order},
                role_groups.role_group
            )
            "#,
            role_groups_values = MARKET_ROLE_GROUPS_VALUES_SQL,
            role_group_classifier = MARKET_ROLE_GROUP_CLASSIFIER_CASE_SQL,
            role_group_order = MARKET_ROLE_GROUP_ORDER_ARRAY_SQL,
        );
        let rows = sqlx::query_as::<_, MarketRoleDemandRow>(&query)
            .bind(period_days)
            .fetch_all(pool)
            .await?;

        Ok((
            rows.into_iter()
                .map(|row| MarketRoleDemandEntry {
                    trend: compare_market_counts(row.this_period, row.prev_period),
                    role_group: row.role_group,
                    this_period: row.this_period,
                    prev_period: row.prev_period,
                })
                .collect(),
            MarketSource::Live,
        ))
    }

    pub async fn market_region_breakdown(
        &self,
    ) -> Result<(Vec<MarketRegionDemandEntry>, MarketSource), RepositoryError> {
        let Some(pool) = self.database.pool() else {
            return Err(RepositoryError::DatabaseDisabled);
        };

        #[derive(FromRow)]
        struct MarketRegionDemandRow {
            region: String,
            job_count: i64,
            top_roles: Vec<String>,
        }

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
                SELECT
                    region,
                    COUNT(*)::bigint AS job_count
                FROM classified_jobs
                GROUP BY region
            ),
            role_counts AS (
                SELECT
                    region,
                    role_group,
                    COUNT(*)::bigint AS role_count
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
            SELECT
                region_groups.region,
                COALESCE(counts.job_count, 0)::bigint AS job_count,
                COALESCE(top_roles.top_roles, ARRAY[]::text[]) AS top_roles
            FROM region_groups
            LEFT JOIN counts USING (region)
            LEFT JOIN top_roles USING (region)
            ORDER BY region_groups.region_rank ASC
            "#,
            role_group_classifier = MARKET_ROLE_GROUP_CLASSIFIER_CASE_SQL,
        );

        let rows = sqlx::query_as::<_, MarketRegionDemandRow>(&query)
            .fetch_all(pool)
            .await?;

        Ok((
            rows.into_iter()
                .map(|row| MarketRegionDemandEntry {
                    region: row.region,
                    job_count: row.job_count.max(0) as u32,
                    top_roles: row.top_roles,
                })
                .collect(),
            MarketSource::Live,
        ))
    }

    pub async fn market_tech_demand(
        &self,
    ) -> Result<(Vec<MarketTechDemandEntry>, MarketSource), RepositoryError> {
        let Some(pool) = self.database.pool() else {
            return Err(RepositoryError::DatabaseDisabled);
        };

        #[derive(FromRow)]
        struct MarketTechDemandRow {
            skill: String,
            job_count: i64,
            percentage: f64,
        }

        let rows = sqlx::query_as::<_, MarketTechDemandRow>(
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
                SELECT
                    title || ' ' || description_text AS searchable_text
                FROM jobs
                WHERE is_active
                  AND last_seen_at >= NOW() - INTERVAL '30 days'
            ),
            total AS (
                SELECT COUNT(*)::bigint AS active_jobs
                FROM active_period_jobs
            )
            SELECT
                tech_skills.skill,
                COUNT(active_period_jobs.searchable_text)::bigint AS job_count,
                CASE
                    WHEN total.active_jobs > 0 THEN
                        COUNT(active_period_jobs.searchable_text)::double precision / total.active_jobs::double precision * 100.0
                    ELSE 0.0
                END AS percentage
            FROM tech_skills
            CROSS JOIN total
            LEFT JOIN active_period_jobs
                ON active_period_jobs.searchable_text ~* tech_skills.pattern
            GROUP BY tech_skills.skill, total.active_jobs
            HAVING COUNT(active_period_jobs.searchable_text) > 0
            ORDER BY job_count DESC, tech_skills.skill ASC
            LIMIT 20
            "#,
        )
        .fetch_all(pool)
        .await?;

        Ok((
            rows.into_iter()
                .map(|row| MarketTechDemandEntry {
                    skill: row.skill,
                    job_count: row.job_count.max(0) as u32,
                    percentage: row.percentage as f32,
                })
                .collect(),
            MarketSource::Live,
        ))
    }
}

fn compare_market_counts(this_period: i64, prev_period: i64) -> MarketTrendDirection {
    match this_period.cmp(&prev_period) {
        std::cmp::Ordering::Greater => MarketTrendDirection::Up,
        std::cmp::Ordering::Less => MarketTrendDirection::Down,
        std::cmp::Ordering::Equal => MarketTrendDirection::Stable,
    }
}

fn compare_company_velocity(this_week: i64, prev_week: i64) -> MarketCompanyVelocityTrend {
    match this_week.cmp(&prev_week) {
        std::cmp::Ordering::Greater => MarketCompanyVelocityTrend::Growing,
        std::cmp::Ordering::Less => MarketCompanyVelocityTrend::Declining,
        std::cmp::Ordering::Equal => MarketCompanyVelocityTrend::Stable,
    }
}
