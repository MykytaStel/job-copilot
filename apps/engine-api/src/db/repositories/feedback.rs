use sqlx::FromRow;

use crate::db::Database;
use crate::db::repositories::RepositoryError;
use crate::domain::feedback::model::{
    CompanyFeedbackRecord, CompanyFeedbackStatus, JobFeedbackFlags, JobFeedbackReason,
    JobFeedbackRecord, JobFeedbackTagRecord, LegitimacySignal, SalaryFeedbackSignal,
    WorkModeFeedbackSignal,
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
    salary_signal: Option<String>,
    interest_rating: Option<i16>,
    work_mode_signal: Option<String>,
    legitimacy_signal: Option<String>,
    created_at: String,
    updated_at: String,
}

#[derive(FromRow)]
struct JobFeedbackTagRow {
    profile_id: String,
    job_id: String,
    tag: String,
    is_negative: bool,
    created_at: String,
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
                salary_signal,
                interest_rating,
                work_mode_signal,
                legitimacy_signal,
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

    pub async fn set_salary_signal(
        &self,
        profile_id: &str,
        job_id: &str,
        signal: SalaryFeedbackSignal,
    ) -> Result<JobFeedbackRecord, RepositoryError> {
        let Some(pool) = self.database.pool() else {
            return Err(RepositoryError::DatabaseDisabled);
        };

        let row = sqlx::query_as::<_, JobFeedbackRow>(
            r#"
            INSERT INTO profile_job_feedback (
                profile_id, job_id, saved, hidden, bad_fit, salary_signal, created_at, updated_at
            )
            VALUES ($1, $2, false, false, false, $3, NOW(), NOW())
            ON CONFLICT (profile_id, job_id)
            DO UPDATE SET salary_signal = EXCLUDED.salary_signal, updated_at = NOW()
            RETURNING
                profile_id, job_id, saved, hidden, bad_fit,
                salary_signal, interest_rating, work_mode_signal, legitimacy_signal,
                created_at::text AS created_at, updated_at::text AS updated_at
            "#,
        )
        .bind(profile_id)
        .bind(job_id)
        .bind(signal.as_str())
        .fetch_one(pool)
        .await?;

        JobFeedbackRecord::try_from(row)
    }

    pub async fn set_interest_rating(
        &self,
        profile_id: &str,
        job_id: &str,
        rating: i8,
    ) -> Result<JobFeedbackRecord, RepositoryError> {
        let Some(pool) = self.database.pool() else {
            return Err(RepositoryError::DatabaseDisabled);
        };

        let row = sqlx::query_as::<_, JobFeedbackRow>(
            r#"
            INSERT INTO profile_job_feedback (
                profile_id, job_id, saved, hidden, bad_fit, interest_rating, created_at, updated_at
            )
            VALUES ($1, $2, false, false, false, $3, NOW(), NOW())
            ON CONFLICT (profile_id, job_id)
            DO UPDATE SET interest_rating = EXCLUDED.interest_rating, updated_at = NOW()
            RETURNING
                profile_id, job_id, saved, hidden, bad_fit,
                salary_signal, interest_rating, work_mode_signal, legitimacy_signal,
                created_at::text AS created_at, updated_at::text AS updated_at
            "#,
        )
        .bind(profile_id)
        .bind(job_id)
        .bind(i16::from(rating))
        .fetch_one(pool)
        .await?;

        JobFeedbackRecord::try_from(row)
    }

    pub async fn set_work_mode_signal(
        &self,
        profile_id: &str,
        job_id: &str,
        signal: WorkModeFeedbackSignal,
    ) -> Result<JobFeedbackRecord, RepositoryError> {
        let Some(pool) = self.database.pool() else {
            return Err(RepositoryError::DatabaseDisabled);
        };

        let row = sqlx::query_as::<_, JobFeedbackRow>(
            r#"
            INSERT INTO profile_job_feedback (
                profile_id, job_id, saved, hidden, bad_fit, work_mode_signal, created_at, updated_at
            )
            VALUES ($1, $2, false, false, false, $3, NOW(), NOW())
            ON CONFLICT (profile_id, job_id)
            DO UPDATE SET work_mode_signal = EXCLUDED.work_mode_signal, updated_at = NOW()
            RETURNING
                profile_id, job_id, saved, hidden, bad_fit,
                salary_signal, interest_rating, work_mode_signal, legitimacy_signal,
                created_at::text AS created_at, updated_at::text AS updated_at
            "#,
        )
        .bind(profile_id)
        .bind(job_id)
        .bind(signal.as_str())
        .fetch_one(pool)
        .await?;

        JobFeedbackRecord::try_from(row)
    }

    pub async fn set_legitimacy_signal(
        &self,
        profile_id: &str,
        job_id: &str,
        signal: LegitimacySignal,
        also_bad_fit: bool,
    ) -> Result<JobFeedbackRecord, RepositoryError> {
        let Some(pool) = self.database.pool() else {
            return Err(RepositoryError::DatabaseDisabled);
        };

        let row = sqlx::query_as::<_, JobFeedbackRow>(
            r#"
            INSERT INTO profile_job_feedback (
                profile_id, job_id, saved, hidden, bad_fit, legitimacy_signal, created_at, updated_at
            )
            VALUES ($1, $2, false, false, $3, $4, NOW(), NOW())
            ON CONFLICT (profile_id, job_id)
            DO UPDATE SET
                legitimacy_signal = EXCLUDED.legitimacy_signal,
                bad_fit = profile_job_feedback.bad_fit OR EXCLUDED.bad_fit,
                updated_at = NOW()
            RETURNING
                profile_id, job_id, saved, hidden, bad_fit,
                salary_signal, interest_rating, work_mode_signal, legitimacy_signal,
                created_at::text AS created_at, updated_at::text AS updated_at
            "#,
        )
        .bind(profile_id)
        .bind(job_id)
        .bind(also_bad_fit)
        .bind(signal.as_str())
        .fetch_one(pool)
        .await?;

        JobFeedbackRecord::try_from(row)
    }

    pub async fn upsert_job_feedback_tags(
        &self,
        profile_id: &str,
        job_id: &str,
        tags: &[JobFeedbackReason],
    ) -> Result<Vec<JobFeedbackTagRecord>, RepositoryError> {
        if tags.is_empty() {
            return self.list_feedback_tags_for_job(profile_id, job_id).await;
        }

        let Some(pool) = self.database.pool() else {
            return Err(RepositoryError::DatabaseDisabled);
        };

        for tag in tags {
            sqlx::query(
                r#"
                INSERT INTO profile_job_feedback_tags (profile_id, job_id, tag, created_at)
                VALUES ($1, $2, $3, NOW())
                ON CONFLICT (profile_id, job_id, tag) DO NOTHING
                "#,
            )
            .bind(profile_id)
            .bind(job_id)
            .bind(tag.as_str())
            .execute(pool)
            .await?;
        }

        self.list_feedback_tags_for_job(profile_id, job_id).await
    }

    pub async fn remove_job_feedback_tag(
        &self,
        profile_id: &str,
        job_id: &str,
        tag: JobFeedbackReason,
    ) -> Result<(), RepositoryError> {
        let Some(pool) = self.database.pool() else {
            return Err(RepositoryError::DatabaseDisabled);
        };

        sqlx::query(
            r#"
            DELETE FROM profile_job_feedback_tags
            WHERE profile_id = $1 AND job_id = $2 AND tag = $3
            "#,
        )
        .bind(profile_id)
        .bind(job_id)
        .bind(tag.as_str())
        .execute(pool)
        .await?;

        Ok(())
    }

    async fn list_feedback_tags_for_job(
        &self,
        profile_id: &str,
        job_id: &str,
    ) -> Result<Vec<JobFeedbackTagRecord>, RepositoryError> {
        let Some(pool) = self.database.pool() else {
            return Err(RepositoryError::DatabaseDisabled);
        };

        let rows = sqlx::query_as::<_, JobFeedbackTagRow>(
            r#"
            SELECT profile_id, job_id, tag, is_negative, created_at::text AS created_at
            FROM profile_job_feedback_tags
            WHERE profile_id = $1 AND job_id = $2
            ORDER BY created_at ASC
            "#,
        )
        .bind(profile_id)
        .bind(job_id)
        .fetch_all(pool)
        .await?;

        rows.into_iter()
            .filter_map(|row| {
                JobFeedbackReason::parse(&row.tag).map(|tag| {
                    Ok(JobFeedbackTagRecord {
                        profile_id: row.profile_id,
                        job_id: row.job_id,
                        tag,
                        is_negative: row.is_negative,
                        created_at: row.created_at,
                    })
                })
            })
            .collect::<Result<Vec<_>, RepositoryError>>()
    }

    pub async fn list_feedback_tags_for_jobs(
        &self,
        profile_id: &str,
        job_ids: &[String],
    ) -> Result<Vec<JobFeedbackTagRecord>, RepositoryError> {
        if job_ids.is_empty() {
            return Ok(Vec::new());
        }

        let Some(pool) = self.database.pool() else {
            return Err(RepositoryError::DatabaseDisabled);
        };

        let rows = sqlx::query_as::<_, JobFeedbackTagRow>(
            r#"
            SELECT profile_id, job_id, tag, is_negative, created_at::text AS created_at
            FROM profile_job_feedback_tags
            WHERE profile_id = $1 AND job_id = ANY($2)
            ORDER BY job_id ASC, created_at ASC
            "#,
        )
        .bind(profile_id)
        .bind(job_ids)
        .fetch_all(pool)
        .await?;

        rows.into_iter()
            .filter_map(|row| {
                JobFeedbackReason::parse(&row.tag).map(|tag| {
                    Ok(JobFeedbackTagRecord {
                        profile_id: row.profile_id,
                        job_id: row.job_id,
                        tag,
                        is_negative: row.is_negative,
                        created_at: row.created_at,
                    })
                })
            })
            .collect::<Result<Vec<_>, RepositoryError>>()
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
                salary_signal,
                interest_rating,
                work_mode_signal,
                legitimacy_signal,
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
                salary_signal,
                interest_rating,
                work_mode_signal,
                legitimacy_signal,
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

    pub async fn clear_all_hidden_jobs(&self, profile_id: &str) -> Result<u64, RepositoryError> {
        let Some(pool) = self.database.pool() else {
            return Err(RepositoryError::DatabaseDisabled);
        };

        let result = sqlx::query(
            r#"
        UPDATE profile_job_feedback
        SET
            hidden = false,
            updated_at = NOW()
        WHERE profile_id = $1
          AND hidden = true
        "#,
        )
        .bind(profile_id)
        .execute(pool)
        .await?;

        Ok(result.rows_affected())
    }

    pub async fn delete_all_for_profile(&self, profile_id: &str) -> Result<u64, RepositoryError> {
        let Some(pool) = self.database.pool() else {
            return Err(RepositoryError::DatabaseDisabled);
        };

        let tag_result = sqlx::query(
            r#"
            DELETE FROM profile_job_feedback_tags
            WHERE profile_id = $1
            "#,
        )
        .bind(profile_id)
        .execute(pool)
        .await?;

        let job_result = sqlx::query(
            r#"
            DELETE FROM profile_job_feedback
            WHERE profile_id = $1
            "#,
        )
        .bind(profile_id)
        .execute(pool)
        .await?;

        let company_result = sqlx::query(
            r#"
            DELETE FROM profile_company_feedback
            WHERE profile_id = $1
            "#,
        )
        .bind(profile_id)
        .execute(pool)
        .await?;

        Ok(
            tag_result.rows_affected()
                + job_result.rows_affected()
                + company_result.rows_affected(),
        )
    }

    /// Clear specific job-level feedback flags. Only updates flags that are `true` in `flags`.
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
                salary_signal,
                interest_rating,
                work_mode_signal,
                legitimacy_signal,
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
        let interest_rating = row
            .interest_rating
            .map(i8::try_from)
            .transpose()
            .map_err(|_| RepositoryError::InvalidData {
                message: format!(
                    "interest_rating '{}' is outside the supported i8 range",
                    row.interest_rating.unwrap_or_default()
                ),
            })?;
        let salary_signal = row
            .salary_signal
            .as_deref()
            .map(SalaryFeedbackSignal::parse)
            .flatten();
        let work_mode_signal = row
            .work_mode_signal
            .as_deref()
            .map(WorkModeFeedbackSignal::parse)
            .flatten();
        let legitimacy_signal = row
            .legitimacy_signal
            .as_deref()
            .map(LegitimacySignal::parse)
            .flatten();

        Ok(Self {
            profile_id: row.profile_id,
            job_id: row.job_id,
            saved: row.saved,
            hidden: row.hidden,
            bad_fit: row.bad_fit,
            salary_signal,
            interest_rating,
            work_mode_signal,
            legitimacy_signal,
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
