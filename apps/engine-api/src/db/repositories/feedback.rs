use sqlx::FromRow;

use crate::db::Database;
use crate::db::repositories::RepositoryError;
use crate::domain::feedback::model::{
    CompanyFeedbackRecord, CompanyFeedbackStatus, JobFeedbackFlags, JobFeedbackRecord,
};

#[derive(Clone)]
pub struct FeedbackRepository {
    database: Database,
}

#[derive(FromRow)]
struct JobFeedbackRow {
    profile_id: String,
    job_id: String,
    saved: bool,
    hidden: bool,
    bad_fit: bool,
    created_at: String,
    updated_at: String,
}

#[derive(FromRow)]
struct CompanyFeedbackRow {
    profile_id: String,
    company_name: String,
    normalized_company_name: String,
    status: String,
    created_at: String,
    updated_at: String,
}

impl FeedbackRepository {
    pub fn new(database: Database) -> Self {
        Self { database }
    }

    pub async fn upsert_job_feedback(
        &self,
        profile_id: &str,
        job_id: &str,
        flags: &JobFeedbackFlags,
    ) -> Result<JobFeedbackRecord, RepositoryError> {
        let Some(pool) = self.database.pool() else {
            return Err(RepositoryError::DatabaseDisabled);
        };

        let row = sqlx::query_as::<_, JobFeedbackRow>(
            r#"
            INSERT INTO profile_job_feedback (
                profile_id,
                job_id,
                saved,
                hidden,
                bad_fit,
                created_at,
                updated_at
            )
            VALUES ($1, $2, $3, $4, $5, NOW(), NOW())
            ON CONFLICT (profile_id, job_id)
            DO UPDATE SET
                saved = profile_job_feedback.saved OR EXCLUDED.saved,
                hidden = profile_job_feedback.hidden OR EXCLUDED.hidden,
                bad_fit = profile_job_feedback.bad_fit OR EXCLUDED.bad_fit,
                updated_at = NOW()
            RETURNING
                profile_id,
                job_id,
                saved,
                hidden,
                bad_fit,
                created_at::text AS created_at,
                updated_at::text AS updated_at
            "#,
        )
        .bind(profile_id)
        .bind(job_id)
        .bind(flags.saved)
        .bind(flags.hidden)
        .bind(flags.bad_fit)
        .fetch_one(pool)
        .await?;

        JobFeedbackRecord::try_from(row)
    }

    pub async fn list_job_feedback(
        &self,
        profile_id: &str,
    ) -> Result<Vec<JobFeedbackRecord>, RepositoryError> {
        let Some(pool) = self.database.pool() else {
            return Err(RepositoryError::DatabaseDisabled);
        };

        let rows = sqlx::query_as::<_, JobFeedbackRow>(
            r#"
            SELECT
                profile_id,
                job_id,
                saved,
                hidden,
                bad_fit,
                created_at::text AS created_at,
                updated_at::text AS updated_at
            FROM profile_job_feedback
            WHERE profile_id = $1
            ORDER BY updated_at DESC, job_id ASC
            "#,
        )
        .bind(profile_id)
        .fetch_all(pool)
        .await?;

        rows.into_iter()
            .map(JobFeedbackRecord::try_from)
            .collect::<Result<Vec<_>, _>>()
    }

    pub async fn list_job_feedback_for_jobs(
        &self,
        profile_id: &str,
        job_ids: &[String],
    ) -> Result<Vec<JobFeedbackRecord>, RepositoryError> {
        if job_ids.is_empty() {
            return Ok(Vec::new());
        }

        let Some(pool) = self.database.pool() else {
            return Err(RepositoryError::DatabaseDisabled);
        };

        let rows = sqlx::query_as::<_, JobFeedbackRow>(
            r#"
            SELECT
                profile_id,
                job_id,
                saved,
                hidden,
                bad_fit,
                created_at::text AS created_at,
                updated_at::text AS updated_at
            FROM profile_job_feedback
            WHERE profile_id = $1
              AND job_id = ANY($2)
            "#,
        )
        .bind(profile_id)
        .bind(job_ids)
        .fetch_all(pool)
        .await?;

        rows.into_iter()
            .map(JobFeedbackRecord::try_from)
            .collect::<Result<Vec<_>, _>>()
    }

    pub async fn upsert_company_feedback(
        &self,
        profile_id: &str,
        company_name: &str,
        normalized_company_name: &str,
        status: CompanyFeedbackStatus,
    ) -> Result<CompanyFeedbackRecord, RepositoryError> {
        let Some(pool) = self.database.pool() else {
            return Err(RepositoryError::DatabaseDisabled);
        };

        let row = sqlx::query_as::<_, CompanyFeedbackRow>(
            r#"
            INSERT INTO profile_company_feedback (
                profile_id,
                normalized_company_name,
                company_name,
                status,
                created_at,
                updated_at
            )
            VALUES ($1, $2, $3, $4, NOW(), NOW())
            ON CONFLICT (profile_id, normalized_company_name)
            DO UPDATE SET
                company_name = EXCLUDED.company_name,
                status = EXCLUDED.status,
                updated_at = NOW()
            RETURNING
                profile_id,
                company_name,
                normalized_company_name,
                status,
                created_at::text AS created_at,
                updated_at::text AS updated_at
            "#,
        )
        .bind(profile_id)
        .bind(normalized_company_name)
        .bind(company_name)
        .bind(status.as_str())
        .fetch_one(pool)
        .await?;

        CompanyFeedbackRecord::try_from(row)
    }

    pub async fn remove_company_feedback(
        &self,
        profile_id: &str,
        normalized_company_name: &str,
        status: CompanyFeedbackStatus,
    ) -> Result<bool, RepositoryError> {
        let Some(pool) = self.database.pool() else {
            return Err(RepositoryError::DatabaseDisabled);
        };

        let deleted = sqlx::query(
            r#"
            DELETE FROM profile_company_feedback
            WHERE profile_id = $1
              AND normalized_company_name = $2
              AND status = $3
            "#,
        )
        .bind(profile_id)
        .bind(normalized_company_name)
        .bind(status.as_str())
        .execute(pool)
        .await?
        .rows_affected();

        Ok(deleted > 0)
    }

    pub async fn list_company_feedback(
        &self,
        profile_id: &str,
    ) -> Result<Vec<CompanyFeedbackRecord>, RepositoryError> {
        let Some(pool) = self.database.pool() else {
            return Err(RepositoryError::DatabaseDisabled);
        };

        let rows = sqlx::query_as::<_, CompanyFeedbackRow>(
            r#"
            SELECT
                profile_id,
                company_name,
                normalized_company_name,
                status,
                created_at::text AS created_at,
                updated_at::text AS updated_at
            FROM profile_company_feedback
            WHERE profile_id = $1
            ORDER BY updated_at DESC, normalized_company_name ASC
            "#,
        )
        .bind(profile_id)
        .fetch_all(pool)
        .await?;

        rows.into_iter()
            .map(CompanyFeedbackRecord::try_from)
            .collect::<Result<Vec<_>, _>>()
    }

    /// Clear specific job-level feedback flags. Only updates flags that are `true` in `flags`.
    /// Returns the updated record, or `None` if no row exists for this (profile, job) pair.
    pub async fn clear_job_feedback(
        &self,
        profile_id: &str,
        job_id: &str,
        flags: &JobFeedbackFlags,
    ) -> Result<Option<JobFeedbackRecord>, RepositoryError> {
        let Some(pool) = self.database.pool() else {
            return Err(RepositoryError::DatabaseDisabled);
        };

        let row = sqlx::query_as::<_, JobFeedbackRow>(
            r#"
            UPDATE profile_job_feedback
            SET
                saved    = CASE WHEN $3 THEN false ELSE saved    END,
                hidden   = CASE WHEN $4 THEN false ELSE hidden   END,
                bad_fit  = CASE WHEN $5 THEN false ELSE bad_fit  END,
                updated_at = NOW()
            WHERE profile_id = $1 AND job_id = $2
            RETURNING
                profile_id,
                job_id,
                saved,
                hidden,
                bad_fit,
                created_at::text AS created_at,
                updated_at::text AS updated_at
            "#,
        )
        .bind(profile_id)
        .bind(job_id)
        .bind(flags.saved)
        .bind(flags.hidden)
        .bind(flags.bad_fit)
        .fetch_optional(pool)
        .await?;

        row.map(JobFeedbackRecord::try_from).transpose()
    }

    pub async fn list_company_feedback_for_names(
        &self,
        profile_id: &str,
        normalized_company_names: &[String],
    ) -> Result<Vec<CompanyFeedbackRecord>, RepositoryError> {
        if normalized_company_names.is_empty() {
            return Ok(Vec::new());
        }

        let Some(pool) = self.database.pool() else {
            return Err(RepositoryError::DatabaseDisabled);
        };

        let rows = sqlx::query_as::<_, CompanyFeedbackRow>(
            r#"
            SELECT
                profile_id,
                company_name,
                normalized_company_name,
                status,
                created_at::text AS created_at,
                updated_at::text AS updated_at
            FROM profile_company_feedback
            WHERE profile_id = $1
              AND normalized_company_name = ANY($2)
            "#,
        )
        .bind(profile_id)
        .bind(normalized_company_names)
        .fetch_all(pool)
        .await?;

        rows.into_iter()
            .map(CompanyFeedbackRecord::try_from)
            .collect::<Result<Vec<_>, _>>()
    }
}

impl TryFrom<JobFeedbackRow> for JobFeedbackRecord {
    type Error = RepositoryError;

    fn try_from(row: JobFeedbackRow) -> Result<Self, Self::Error> {
        Ok(Self {
            profile_id: row.profile_id,
            job_id: row.job_id,
            saved: row.saved,
            hidden: row.hidden,
            bad_fit: row.bad_fit,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

impl TryFrom<CompanyFeedbackRow> for CompanyFeedbackRecord {
    type Error = RepositoryError;

    fn try_from(row: CompanyFeedbackRow) -> Result<Self, Self::Error> {
        let Some(status) = CompanyFeedbackStatus::parse(&row.status) else {
            return Err(RepositoryError::InvalidData {
                message: format!("unsupported company feedback status '{}'", row.status),
            });
        };

        Ok(Self {
            profile_id: row.profile_id,
            company_name: row.company_name,
            normalized_company_name: row.normalized_company_name,
            status,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}
