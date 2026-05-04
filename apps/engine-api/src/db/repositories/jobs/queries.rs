use crate::db::repositories::RepositoryError;
use crate::domain::analytics::model::{JobSourceCount, SalaryBucket};
use crate::domain::job::model::{Job, JobFeedSummary, JobView};
use sqlx::{FromRow, Postgres, QueryBuilder};

use super::JobsRepository;
use super::rows::{JobFeedSummaryRow, JobRow, JobViewRow};

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
            jobs.language,
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

pub(super) fn job_view_query(where_clause: Option<&str>, limit_clause: Option<&str>) -> String {
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
            jobs.language,
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

impl JobsRepository {
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
                language,
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

    pub async fn get_views_by_ids(&self, ids: &[String]) -> Result<Vec<JobView>, RepositoryError> {
        if ids.is_empty() {
            return Ok(vec![]);
        }

        let Some(pool) = self.database.pool() else {
            return Err(RepositoryError::DatabaseDisabled);
        };

        let rows = sqlx::query_as::<_, JobViewRow>(&job_view_query(
            Some("WHERE jobs.id = ANY($1::text[])"),
            None,
        ))
        .bind(ids)
        .fetch_all(pool)
        .await?;

        Ok(rows.into_iter().map(JobView::from).collect())
    }

    pub async fn list_filtered_views(
        &self,
        limit: i64,
        lifecycle: Option<&str>,
        source: Option<&str>,
        quality_min: Option<i32>,
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

        let mut builder = QueryBuilder::<Postgres>::new(JOB_VIEW_BASE_SELECT);
        builder.push("\nWHERE jobs.duplicate_of IS NULL");
        let mut has_where = true;

        if let Some(cond) = lifecycle_cond {
            builder.push("\nAND ");
            builder.push(cond);
        }

        if let Some(src) = source {
            builder.push(if has_where { "\nAND " } else { "\nWHERE " });
            builder.push("EXISTS (SELECT 1 FROM job_variants WHERE job_id = jobs.id AND source = ");
            builder.push_bind(src);
            builder.push(")");
            has_where = true;
        }

        if let Some(min_score) = quality_min {
            builder.push(if has_where { "\nAND " } else { "\nWHERE " });
            builder.push("jobs.quality_score >= ");
            builder.push_bind(min_score);
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
                )::bigint AS reactivated_jobs,
                MAX(last_seen_at)::text AS last_ingested_at
            FROM jobs
            WHERE duplicate_of IS NULL
            "#,
        )
        .fetch_one(pool)
        .await?;

        Ok(JobFeedSummary {
            total_jobs: row.total_jobs,
            active_jobs: row.active_jobs,
            inactive_jobs: row.inactive_jobs,
            reactivated_jobs: row.reactivated_jobs,
            last_ingested_at: row.last_ingested_at,
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
            .map(|row| SalaryBucket {
                seniority: row.seniority,
                currency: row.salary_currency,
                min: row.salary_min_min,
                max: row.salary_max_max,
                avg: row.salary_avg,
                job_count: row.job_count,
            })
            .collect())
    }

    pub async fn jobs_by_source(&self) -> Result<Vec<JobSourceCount>, RepositoryError> {
        let Some(pool) = self.database.pool() else {
            return Err(RepositoryError::DatabaseDisabled);
        };

        #[derive(FromRow)]
        struct JobSourceCountRow {
            source: String,
            count: i64,
            last_seen: String,
        }

        let rows = sqlx::query_as::<_, JobSourceCountRow>(
            r#"
            SELECT
                COALESCE(source, 'unknown') AS source,
                COUNT(DISTINCT job_id)::bigint AS count,
                MAX(last_seen_at)::text AS last_seen
            FROM job_variants
            GROUP BY source
            ORDER BY count DESC
            "#,
        )
        .fetch_all(pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| JobSourceCount {
                source: row.source,
                count: row.count,
                last_seen: row.last_seen,
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
                language,
                posted_at::text AS posted_at,
                last_seen_at::text AS last_seen_at,
                is_active
            FROM jobs
            WHERE
                is_active = TRUE
                AND search_vector @@ plainto_tsquery('simple', $1)
            ORDER BY
                ts_rank_cd(search_vector, plainto_tsquery('simple', $1)) DESC,
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
}
