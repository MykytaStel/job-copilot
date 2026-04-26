use crate::db::repositories::RepositoryError;
use crate::domain::market::model::{
    MarketCompanyEntry, MarketOverview, MarketRoleDemandEntry, MarketSalaryTrend, MarketSource,
    MarketTrendDirection,
};
use sqlx::FromRow;

mod market_role_heuristics {
    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../shared/rust/market_role_heuristics.rs"
    ));
}

use super::JobsRepository;
use market_role_heuristics::{
    MARKET_ROLE_GROUP_CLASSIFIER_CASE_SQL, MARKET_ROLE_GROUP_ORDER_ARRAY_SQL,
    MARKET_ROLE_GROUPS_VALUES_SQL,
};

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

    pub async fn market_overview(
        &self,
    ) -> Result<(MarketOverview, MarketSource), RepositoryError> {
        let Some(pool) = self.database.pool() else {
            return Err(RepositoryError::DatabaseDisabled);
        };

        if let Some(payload) = Self::fetch_fresh_snapshot(pool, "market_overview").await? {
            if let Ok(overview) = serde_json::from_value::<MarketOverview>(payload) {
                return Ok((overview, MarketSource::Snapshot));
            }
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
                COUNT(DISTINCT company_name) FILTER (WHERE is_active)::bigint AS active_companies_count,
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

        if let Some(payload) = Self::fetch_fresh_snapshot(pool, "market_companies").await? {
            if let Ok(mut entries) =
                serde_json::from_value::<Vec<MarketCompanyEntry>>(payload)
            {
                entries.truncate(limit as usize);
                return Ok((entries, MarketSource::Snapshot));
            }
        }

        tracing::warn!("market_companies: no fresh snapshot, falling back to live jobs query");

        #[derive(FromRow)]
        struct MarketCompanyRow {
            company_name: String,
            active_jobs: i64,
            this_week: i64,
            prev_week: i64,
        }

        let rows = sqlx::query_as::<_, MarketCompanyRow>(
            r#"
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
            ORDER BY active_jobs DESC
            LIMIT $1
            "#,
        )
        .bind(limit)
        .fetch_all(pool)
        .await?;

        Ok((
            rows.into_iter()
                .map(|row| MarketCompanyEntry {
                    company_name: row.company_name,
                    active_jobs: row.active_jobs,
                    this_week: row.this_week,
                    prev_week: row.prev_week,
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

    pub async fn market_salary_trends(
        &self,
    ) -> Result<(Vec<MarketSalaryTrend>, MarketSource), RepositoryError> {
        self.fetch_market_salary_trends(None).await
    }

    async fn fetch_market_salary_trends(
        &self,
        seniority: Option<&str>,
    ) -> Result<(Vec<MarketSalaryTrend>, MarketSource), RepositoryError> {
        let Some(pool) = self.database.pool() else {
            return Err(RepositoryError::DatabaseDisabled);
        };

        if let Some(payload) = Self::fetch_fresh_snapshot(pool, "market_salary_trends").await? {
            if let Ok(mut trends) = serde_json::from_value::<Vec<MarketSalaryTrend>>(payload) {
                if let Some(filter) = seniority {
                    trends.retain(|t| t.seniority == filter);
                }
                return Ok((trends, MarketSource::Snapshot));
            }
        }

        tracing::warn!(
            "market_salary_trends: no fresh snapshot, falling back to live jobs query"
        );

        #[derive(FromRow)]
        struct MarketSalaryTrendRow {
            seniority: String,
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
                    salary_min
                FROM jobs
                WHERE is_active
                  AND salary_min IS NOT NULL
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
                ROUND(PERCENTILE_CONT(0.25) WITHIN GROUP (ORDER BY salary_min))::integer AS p25,
                ROUND(PERCENTILE_CONT(0.50) WITHIN GROUP (ORDER BY salary_min))::integer AS median,
                ROUND(PERCENTILE_CONT(0.75) WITHIN GROUP (ORDER BY salary_min))::integer AS p75,
                COUNT(*)::bigint AS sample_count
            FROM dominant_jobs
            GROUP BY seniority
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

        if let Some(payload) = Self::fetch_fresh_snapshot(pool, "market_role_demand").await? {
            if let Ok(entries) = serde_json::from_value::<Vec<MarketRoleDemandEntry>>(payload) {
                return Ok((entries, MarketSource::Snapshot));
            }
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
}

fn compare_market_counts(this_period: i64, prev_period: i64) -> MarketTrendDirection {
    match this_period.cmp(&prev_period) {
        std::cmp::Ordering::Greater => MarketTrendDirection::Up,
        std::cmp::Ordering::Less => MarketTrendDirection::Down,
        std::cmp::Ordering::Equal => MarketTrendDirection::Stable,
    }
}
