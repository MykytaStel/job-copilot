use crate::db::Database;
use crate::db::repositories::RepositoryError;
use crate::domain::analytics::model::{JobSourceCount, SalaryBucket};
use crate::domain::job::model::{
    Job, JobFeedSummary, JobLifecycleStage, JobSourceVariant, JobView,
};
use crate::domain::market::model::{
    MarketCompanyEntry, MarketOverview, MarketRoleDemandEntry, MarketSalaryTrend,
    MarketTrendDirection,
};
use sqlx::FromRow;

#[derive(Clone)]
pub struct JobsRepository {
    database: Database,
}

#[derive(FromRow)]
struct JobRow {
    id: String,
    title: String,
    company_name: String,
    location: Option<String>,
    remote_type: Option<String>,
    seniority: Option<String>,
    description_text: String,
    salary_min: Option<i32>,
    salary_max: Option<i32>,
    salary_currency: Option<String>,
    posted_at: Option<String>,
    last_seen_at: String,
    is_active: bool,
}

#[derive(FromRow)]
struct JobViewRow {
    id: String,
    title: String,
    company_name: String,
    location: Option<String>,
    remote_type: Option<String>,
    seniority: Option<String>,
    description_text: String,
    salary_min: Option<i32>,
    salary_max: Option<i32>,
    salary_currency: Option<String>,
    posted_at: Option<String>,
    first_seen_at: String,
    last_seen_at: String,
    is_active: bool,
    inactivated_at: Option<String>,
    reactivated_at: Option<String>,
    variant_source: Option<String>,
    variant_source_job_id: Option<String>,
    variant_source_url: Option<String>,
    variant_raw_payload: Option<serde_json::Value>,
    variant_fetched_at: Option<String>,
    variant_last_seen_at: Option<String>,
    variant_is_active: Option<bool>,
    variant_inactivated_at: Option<String>,
}

#[derive(FromRow)]
struct JobFeedSummaryRow {
    total_jobs: i64,
    active_jobs: i64,
    inactive_jobs: i64,
    reactivated_jobs: i64,
}

impl JobsRepository {
    pub fn new(database: Database) -> Self {
        Self { database }
    }

    pub async fn get_by_id(&self, id: &str) -> Result<Option<Job>, RepositoryError> {
        let Some(pool) = self.database.pool() else {
            return Err(RepositoryError::DatabaseDisabled);
        };

        let row = sqlx::query_as::<_, JobRow>(
            r#"
            SELECT
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
                posted_at::text AS posted_at,
                last_seen_at::text AS last_seen_at,
                is_active
            FROM jobs
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;

        Ok(row.map(Job::from))
    }

    pub async fn get_view_by_id(&self, id: &str) -> Result<Option<JobView>, RepositoryError> {
        let Some(pool) = self.database.pool() else {
            return Err(RepositoryError::DatabaseDisabled);
        };

        let row =
            sqlx::query_as::<_, JobViewRow>(&job_view_query(Some("WHERE jobs.id = $1"), None))
                .bind(id)
                .fetch_optional(pool)
                .await?;

        Ok(row.map(JobView::from))
    }

    #[allow(dead_code)]
    pub async fn list_recent(&self, limit: i64) -> Result<Vec<Job>, RepositoryError> {
        let Some(pool) = self.database.pool() else {
            return Err(RepositoryError::DatabaseDisabled);
        };

        let rows = sqlx::query_as::<_, JobRow>(
            r#"
            SELECT
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
                posted_at::text AS posted_at,
                last_seen_at::text AS last_seen_at,
                is_active
            FROM jobs
            ORDER BY last_seen_at DESC, posted_at DESC NULLS LAST
            LIMIT $1
            "#,
        )
        .bind(limit)
        .fetch_all(pool)
        .await?;

        Ok(rows.into_iter().map(Job::from).collect())
    }

    pub async fn list_recent_views(&self, limit: i64) -> Result<Vec<JobView>, RepositoryError> {
        let Some(pool) = self.database.pool() else {
            return Err(RepositoryError::DatabaseDisabled);
        };

        let rows = sqlx::query_as::<_, JobViewRow>(&job_view_query(None, Some("LIMIT $1")))
            .bind(limit)
            .fetch_all(pool)
            .await?;

        Ok(rows.into_iter().map(JobView::from).collect())
    }

    pub async fn list_filtered_views(
        &self,
        limit: i64,
        lifecycle: Option<&str>,
        source: Option<&str>,
    ) -> Result<Vec<JobView>, RepositoryError> {
        let Some(pool) = self.database.pool() else {
            return Err(RepositoryError::DatabaseDisabled);
        };

        let lifecycle_cond = match lifecycle {
            Some("inactive") => Some("jobs.is_active = FALSE"),
            Some("reactivated") => Some(
                "jobs.is_active = TRUE AND jobs.reactivated_at IS NOT NULL AND jobs.reactivated_at::text = jobs.last_seen_at::text",
            ),
            Some("active") => Some(
                "jobs.is_active = TRUE AND NOT (jobs.reactivated_at IS NOT NULL AND jobs.reactivated_at::text = jobs.last_seen_at::text)",
            ),
            _ => None,
        };

        let mut builder = sqlx::QueryBuilder::new(JOB_VIEW_BASE_SELECT);

        let mut has_where = false;

        if let Some(cond) = lifecycle_cond {
            builder.push("\nWHERE ");
            builder.push(cond);
            has_where = true;
        }

        if let Some(src) = source {
            builder.push(if has_where { "\nAND " } else { "\nWHERE " });
            builder.push("EXISTS (SELECT 1 FROM job_variants WHERE job_id = jobs.id AND source = ");
            builder.push_bind(src);
            builder.push(")");
        }

        builder.push("\nORDER BY jobs.last_seen_at DESC, jobs.posted_at DESC NULLS LAST\nLIMIT ");
        builder.push_bind(limit);

        let rows = builder
            .build_query_as::<JobViewRow>()
            .fetch_all(pool)
            .await?;

        Ok(rows.into_iter().map(JobView::from).collect())
    }

    pub async fn feed_summary(&self) -> Result<JobFeedSummary, RepositoryError> {
        let Some(pool) = self.database.pool() else {
            return Err(RepositoryError::DatabaseDisabled);
        };

        let row = sqlx::query_as::<_, JobFeedSummaryRow>(
            r#"
            SELECT
                COUNT(*)::bigint AS total_jobs,
                COUNT(*) FILTER (WHERE is_active)::bigint AS active_jobs,
                COUNT(*) FILTER (WHERE NOT is_active)::bigint AS inactive_jobs,
                COUNT(*) FILTER (
                    WHERE is_active
                      AND reactivated_at IS NOT NULL
                      AND reactivated_at::text = last_seen_at::text
                )::bigint AS reactivated_jobs
            FROM jobs
            "#,
        )
        .fetch_one(pool)
        .await?;

        Ok(JobFeedSummary {
            total_jobs: row.total_jobs,
            active_jobs: row.active_jobs,
            inactive_jobs: row.inactive_jobs,
            reactivated_jobs: row.reactivated_jobs,
        })
    }

    pub async fn salary_intelligence(&self) -> Result<Vec<SalaryBucket>, RepositoryError> {
        let Some(pool) = self.database.pool() else {
            return Err(RepositoryError::DatabaseDisabled);
        };

        #[derive(FromRow)]
        struct SalaryRow {
            seniority: Option<String>,
            salary_currency: Option<String>,
            salary_min_min: Option<i32>,
            salary_max_max: Option<i32>,
            salary_avg: Option<f64>,
            job_count: i64,
        }

        let rows = sqlx::query_as::<_, SalaryRow>(
            r#"
            SELECT
                seniority,
                salary_currency,
                MIN(salary_min)                                   AS salary_min_min,
                MAX(salary_max)                                   AS salary_max_max,
                AVG((salary_min + salary_max) / 2.0)              AS salary_avg,
                COUNT(*)                                          AS job_count
            FROM jobs
            WHERE salary_min IS NOT NULL
            GROUP BY seniority, salary_currency
            ORDER BY salary_currency NULLS LAST, seniority NULLS LAST
            "#,
        )
        .fetch_all(pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| SalaryBucket {
                seniority: r.seniority,
                currency: r.salary_currency,
                min: r.salary_min_min,
                max: r.salary_max_max,
                avg: r.salary_avg,
                job_count: r.job_count,
            })
            .collect())
    }

    pub async fn jobs_by_source(&self) -> Result<Vec<JobSourceCount>, RepositoryError> {
        let Some(pool) = self.database.pool() else {
            return Err(RepositoryError::DatabaseDisabled);
        };

        #[derive(sqlx::FromRow)]
        struct JobSourceCountRow {
            source: String,
            count: i64,
        }

        let rows = sqlx::query_as::<_, JobSourceCountRow>(
            r#"
            SELECT
                COALESCE(source, 'unknown') AS source,
                COUNT(DISTINCT job_id)::bigint AS count
            FROM job_variants
            GROUP BY source
            ORDER BY count DESC
            "#,
        )
        .fetch_all(pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| JobSourceCount {
                source: r.source,
                count: r.count,
            })
            .collect())
    }

    pub async fn search_active(
        &self,
        query: &str,
        limit: i64,
    ) -> Result<Vec<Job>, RepositoryError> {
        let Some(pool) = self.database.pool() else {
            return Err(RepositoryError::DatabaseDisabled);
        };

        let rows = sqlx::query_as::<_, JobRow>(
            r#"
            SELECT
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
                posted_at::text AS posted_at,
                last_seen_at::text AS last_seen_at,
                is_active
            FROM jobs
            WHERE
                is_active = TRUE
                AND to_tsvector(
                    'simple',
                    concat_ws(
                        ' ',
                        coalesce(title, ''),
                        coalesce(company_name, ''),
                        coalesce(description_text, '')
                    )
                ) @@ plainto_tsquery('simple', $1)
            ORDER BY
                ts_rank_cd(
                    to_tsvector(
                        'simple',
                        concat_ws(
                            ' ',
                            coalesce(title, ''),
                            coalesce(company_name, ''),
                            coalesce(description_text, '')
                        )
                    ),
                    plainto_tsquery('simple', $1)
                ) DESC,
                last_seen_at DESC,
                posted_at DESC NULLS LAST
            LIMIT $2
            "#,
        )
        .bind(query)
        .bind(limit)
        .fetch_all(pool)
        .await?;

        Ok(rows.into_iter().map(Job::from).collect())
    }

    pub async fn market_overview(&self) -> Result<MarketOverview, RepositoryError> {
        let Some(pool) = self.database.pool() else {
            return Err(RepositoryError::DatabaseDisabled);
        };

        #[derive(sqlx::FromRow)]
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

        Ok(MarketOverview {
            new_jobs_this_week: row.new_jobs_this_week,
            active_companies_count: row.active_companies_count,
            active_jobs_count: row.active_jobs_count,
            remote_percentage,
        })
    }

    pub async fn market_companies(
        &self,
        limit: i64,
    ) -> Result<Vec<MarketCompanyEntry>, RepositoryError> {
        let Some(pool) = self.database.pool() else {
            return Err(RepositoryError::DatabaseDisabled);
        };

        #[derive(sqlx::FromRow)]
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

        Ok(rows
            .into_iter()
            .map(|r| MarketCompanyEntry {
                company_name: r.company_name,
                active_jobs: r.active_jobs,
                this_week: r.this_week,
                prev_week: r.prev_week,
            })
            .collect())
    }

    pub async fn market_salary_trend(
        &self,
        seniority: &str,
    ) -> Result<Option<MarketSalaryTrend>, RepositoryError> {
        let Some(pool) = self.database.pool() else {
            return Err(RepositoryError::DatabaseDisabled);
        };

        #[derive(sqlx::FromRow)]
        struct MarketSalaryTrendRow {
            seniority: String,
            p25: i32,
            median: i32,
            p75: i32,
            sample_count: i64,
        }

        let row = sqlx::query_as::<_, MarketSalaryTrendRow>(
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
                  AND LOWER(TRIM(seniority)) = $1
            ),
            dominant_currency AS (
                SELECT salary_currency
                FROM filtered_jobs
                GROUP BY salary_currency
                ORDER BY COUNT(*) DESC, salary_currency ASC
                LIMIT 1
            )
            SELECT
                seniority,
                ROUND(PERCENTILE_CONT(0.25) WITHIN GROUP (ORDER BY salary_min))::integer AS p25,
                ROUND(PERCENTILE_CONT(0.50) WITHIN GROUP (ORDER BY salary_min))::integer AS median,
                ROUND(PERCENTILE_CONT(0.75) WITHIN GROUP (ORDER BY salary_min))::integer AS p75,
                COUNT(*)::bigint AS sample_count
            FROM filtered_jobs
            WHERE salary_currency = (SELECT salary_currency FROM dominant_currency)
            GROUP BY seniority
            "#,
        )
        .bind(seniority.trim().to_lowercase())
        .fetch_optional(pool)
        .await?;

        Ok(row.map(|row| MarketSalaryTrend {
            seniority: row.seniority,
            p25: row.p25,
            median: row.median,
            p75: row.p75,
            sample_count: row.sample_count,
        }))
    }

    pub async fn market_role_demand(
        &self,
        period_days: i32,
    ) -> Result<Vec<MarketRoleDemandEntry>, RepositoryError> {
        let Some(pool) = self.database.pool() else {
            return Err(RepositoryError::DatabaseDisabled);
        };

        #[derive(sqlx::FromRow)]
        struct MarketRoleDemandRow {
            role_group: String,
            this_period: i64,
            prev_period: i64,
        }

        let rows = sqlx::query_as::<_, MarketRoleDemandRow>(
            r#"
            WITH role_groups(role_group) AS (
                VALUES
                    ('Frontend'),
                    ('Backend'),
                    ('Fullstack'),
                    ('DevOps'),
                    ('Data/ML'),
                    ('QA'),
                    ('Design'),
                    ('Management')
            ),
            classified_jobs AS (
                SELECT
                    CASE
                        WHEN title ILIKE '%engineering manager%'
                          OR title ILIKE '%product manager%'
                          OR title ILIKE '%project manager%'
                          OR title ILIKE '%program manager%'
                          OR title ILIKE '%delivery manager%'
                          OR title ILIKE '%product owner%'
                          OR title ILIKE '%tech lead%'
                          OR title ILIKE '%technical lead%'
                          OR title ILIKE '%head of engineering%'
                          OR title ILIKE '%vp of engineering%'
                        THEN 'Management'
                        WHEN title ILIKE '%product designer%'
                          OR title ILIKE '%ui/ux designer%'
                          OR title ILIKE '%ux designer%'
                          OR title ILIKE '%ui designer%'
                          OR title ILIKE '%interaction designer%'
                          OR title ILIKE '%graphic designer%'
                        THEN 'Design'
                        WHEN title ~* '(^|[^a-z])(qa|sdet|tester)([^a-z]|$)'
                          OR title ILIKE '%quality assurance%'
                          OR title ILIKE '%test engineer%'
                          OR title ILIKE '%automation qa%'
                        THEN 'QA'
                        WHEN title ILIKE '%devops%'
                          OR title ILIKE '%site reliability%'
                          OR title ~* '(^|[^a-z])sre([^a-z]|$)'
                          OR title ILIKE '%cloud engineer%'
                          OR title ILIKE '%cloud architect%'
                          OR title ILIKE '%infrastructure%'
                          OR title ILIKE '%platform engineer%'
                        THEN 'DevOps'
                        WHEN title ILIKE '%machine learning%'
                          OR title ILIKE '%ml engineer%'
                          OR title ILIKE '%ai engineer%'
                          OR title ILIKE '%data scientist%'
                          OR title ILIKE '%data engineer%'
                          OR title ILIKE '%data analyst%'
                          OR title ILIKE '%analytics engineer%'
                          OR title ILIKE '%analyst%'
                        THEN 'Data/ML'
                        WHEN title ILIKE '%fullstack%'
                          OR title ILIKE '%full-stack%'
                          OR title ILIKE '%full stack%'
                        THEN 'Fullstack'
                        WHEN title ILIKE '%frontend%'
                          OR title ILIKE '%front-end%'
                          OR title ILIKE '%front end%'
                          OR title ILIKE '%react%'
                          OR title ILIKE '%vue%'
                          OR title ILIKE '%angular%'
                          OR title ILIKE '%next.js%'
                          OR title ILIKE '%nextjs%'
                          OR title ILIKE '%typescript%'
                          OR title ILIKE '%javascript%'
                        THEN 'Frontend'
                        WHEN title ILIKE '%backend%'
                          OR title ILIKE '%back-end%'
                          OR title ILIKE '%back end%'
                          OR title ILIKE '%server-side%'
                          OR title ILIKE '%api developer%'
                          OR title ILIKE '%rust engineer%'
                          OR title ILIKE '%rust developer%'
                          OR title ILIKE '%golang%'
                          OR title ILIKE '%go developer%'
                          OR title ILIKE '%java developer%'
                          OR title ILIKE '%python developer%'
                          OR title ILIKE '%php developer%'
                          OR title ILIKE '%node.js%'
                          OR title ILIKE '%nodejs%'
                        THEN 'Backend'
                        ELSE NULL
                    END AS role_group,
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
                ARRAY['Frontend', 'Backend', 'Fullstack', 'DevOps', 'Data/ML', 'QA', 'Design', 'Management']::text[],
                role_groups.role_group
            )
            "#,
        )
        .bind(period_days)
        .fetch_all(pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| MarketRoleDemandEntry {
                trend: compare_market_counts(row.this_period, row.prev_period),
                role_group: row.role_group,
                this_period: row.this_period,
                prev_period: row.prev_period,
            })
            .collect())
    }
}

fn compare_market_counts(this_period: i64, prev_period: i64) -> MarketTrendDirection {
    match this_period.cmp(&prev_period) {
        std::cmp::Ordering::Greater => MarketTrendDirection::Up,
        std::cmp::Ordering::Less => MarketTrendDirection::Down,
        std::cmp::Ordering::Equal => MarketTrendDirection::Stable,
    }
}

const JOB_VIEW_BASE_SELECT: &str = r#"
        SELECT
            jobs.id,
            jobs.title,
            jobs.company_name,
            jobs.location,
            jobs.remote_type,
            jobs.seniority,
            jobs.description_text,
            jobs.salary_min,
            jobs.salary_max,
            jobs.salary_currency,
            jobs.posted_at::text AS posted_at,
            jobs.first_seen_at::text AS first_seen_at,
            jobs.last_seen_at::text AS last_seen_at,
            jobs.is_active,
            jobs.inactivated_at::text AS inactivated_at,
            jobs.reactivated_at::text AS reactivated_at,
            variants.source AS variant_source,
            variants.source_job_id AS variant_source_job_id,
            variants.source_url AS variant_source_url,
            variants.raw_payload AS variant_raw_payload,
            variants.fetched_at::text AS variant_fetched_at,
            variants.last_seen_at::text AS variant_last_seen_at,
            variants.is_active AS variant_is_active,
            variants.inactivated_at::text AS variant_inactivated_at
        FROM jobs
        LEFT JOIN LATERAL (
            SELECT
                source,
                source_job_id,
                source_url,
                raw_payload,
                fetched_at,
                last_seen_at,
                is_active,
                inactivated_at
            FROM job_variants
            WHERE job_id = jobs.id
            ORDER BY fetched_at DESC, last_seen_at DESC, source ASC
            LIMIT 1
        ) AS variants ON TRUE"#;

fn job_view_query(where_clause: Option<&str>, limit_clause: Option<&str>) -> String {
    format!(
        r#"
        SELECT
            jobs.id,
            jobs.title,
            jobs.company_name,
            jobs.location,
            jobs.remote_type,
            jobs.seniority,
            jobs.description_text,
            jobs.salary_min,
            jobs.salary_max,
            jobs.salary_currency,
            jobs.posted_at::text AS posted_at,
            jobs.first_seen_at::text AS first_seen_at,
            jobs.last_seen_at::text AS last_seen_at,
            jobs.is_active,
            jobs.inactivated_at::text AS inactivated_at,
            jobs.reactivated_at::text AS reactivated_at,
            variants.source AS variant_source,
            variants.source_job_id AS variant_source_job_id,
            variants.source_url AS variant_source_url,
            variants.raw_payload AS variant_raw_payload,
            variants.fetched_at::text AS variant_fetched_at,
            variants.last_seen_at::text AS variant_last_seen_at,
            variants.is_active AS variant_is_active,
            variants.inactivated_at::text AS variant_inactivated_at
        FROM jobs
        LEFT JOIN LATERAL (
            SELECT
                source,
                source_job_id,
                source_url,
                raw_payload,
                fetched_at,
                last_seen_at,
                is_active,
                inactivated_at
            FROM job_variants
            WHERE job_id = jobs.id
            ORDER BY fetched_at DESC, last_seen_at DESC, source ASC
            LIMIT 1
        ) AS variants ON TRUE
        {}
        ORDER BY jobs.last_seen_at DESC, jobs.posted_at DESC NULLS LAST
        {}
        "#,
        where_clause.unwrap_or(""),
        limit_clause.unwrap_or("")
    )
}

impl From<JobRow> for Job {
    fn from(row: JobRow) -> Self {
        Self {
            id: row.id,
            title: row.title,
            company_name: row.company_name,
            location: row.location,
            remote_type: row.remote_type,
            seniority: row.seniority,
            description_text: row.description_text,
            salary_min: row.salary_min,
            salary_max: row.salary_max,
            salary_currency: row.salary_currency,
            posted_at: row.posted_at,
            last_seen_at: row.last_seen_at,
            is_active: row.is_active,
        }
    }
}

impl From<JobViewRow> for JobView {
    fn from(row: JobViewRow) -> Self {
        let lifecycle_stage = if !row.is_active {
            JobLifecycleStage::Inactive
        } else if row
            .reactivated_at
            .as_ref()
            .is_some_and(|value| value == &row.last_seen_at)
        {
            JobLifecycleStage::Reactivated
        } else {
            JobLifecycleStage::Active
        };

        let primary_variant = match (
            row.variant_source,
            row.variant_source_job_id,
            row.variant_source_url,
            row.variant_fetched_at,
            row.variant_last_seen_at,
            row.variant_is_active,
        ) {
            (
                Some(source),
                Some(source_job_id),
                Some(source_url),
                Some(fetched_at),
                Some(last_seen_at),
                Some(is_active),
            ) => Some(JobSourceVariant {
                source,
                source_job_id,
                source_url,
                raw_payload: row.variant_raw_payload,
                fetched_at,
                last_seen_at,
                is_active,
                inactivated_at: row.variant_inactivated_at,
            }),
            _ => None,
        };

        Self {
            job: Job {
                id: row.id,
                title: row.title,
                company_name: row.company_name,
                location: row.location,
                remote_type: row.remote_type,
                seniority: row.seniority,
                description_text: row.description_text,
                salary_min: row.salary_min,
                salary_max: row.salary_max,
                salary_currency: row.salary_currency,
                posted_at: row.posted_at,
                last_seen_at: row.last_seen_at,
                is_active: row.is_active,
            },
            first_seen_at: row.first_seen_at,
            inactivated_at: row.inactivated_at,
            reactivated_at: row.reactivated_at,
            lifecycle_stage,
            primary_variant,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::db::Database;
    use crate::db::repositories::{JobsRepository, RepositoryError};

    use super::job_view_query;

    #[tokio::test]
    async fn returns_disabled_error_without_database() {
        let repository = JobsRepository::new(Database::disabled());

        let error = repository
            .list_recent(10)
            .await
            .expect_err("repository should fail without configured database");

        assert!(matches!(error, RepositoryError::DatabaseDisabled));
    }

    #[test]
    fn job_view_query_appends_limit_after_sorting() {
        let query = job_view_query(None, Some("LIMIT $1"));

        assert!(query.contains("LIMIT $1"));
        assert!(query.contains("ORDER BY jobs.last_seen_at DESC"));
    }
}
