use sqlx::FromRow;
use uuid::Uuid;

use crate::db::Database;
use crate::db::repositories::RepositoryError;
use crate::domain::application::model::{
    Application, ApplicationDetail, CreateApplication, UpdateApplication,
};
use crate::domain::job::model::Job;
use crate::domain::resume::model::ResumeVersion;

#[derive(Clone)]
pub struct ApplicationsRepository {
    database: Database,
}

#[derive(FromRow)]
struct ApplicationRow {
    id: String,
    job_id: String,
    resume_id: Option<String>,
    status: String,
    applied_at: Option<String>,
    due_date: Option<String>,
    updated_at: String,
}

#[derive(FromRow)]
struct ApplicationDetailRow {
    application_id: String,
    application_job_id: String,
    application_resume_id: Option<String>,
    application_status: String,
    application_applied_at: Option<String>,
    application_due_date: Option<String>,
    application_updated_at: String,
    job_id: String,
    job_title: String,
    job_company_name: String,
    job_remote_type: Option<String>,
    job_seniority: Option<String>,
    job_description_text: String,
    job_salary_min: Option<i32>,
    job_salary_max: Option<i32>,
    job_salary_currency: Option<String>,
    job_posted_at: Option<String>,
    job_last_seen_at: String,
    job_is_active: bool,
    resume_version: Option<i32>,
    resume_filename: Option<String>,
    resume_raw_text: Option<String>,
    resume_is_active: Option<bool>,
    resume_uploaded_at: Option<String>,
}

impl ApplicationsRepository {
    pub fn new(database: Database) -> Self {
        Self { database }
    }

    pub async fn create(
        &self,
        application: &CreateApplication,
    ) -> Result<Application, RepositoryError> {
        let Some(pool) = self.database.pool() else {
            return Err(RepositoryError::DatabaseDisabled);
        };

        let row = sqlx::query_as::<_, ApplicationRow>(
            r#"
            INSERT INTO applications (
                id,
                job_id,
                resume_id,
                status,
                applied_at,
                due_date,
                updated_at
            )
            VALUES ($1, $2, NULL, $3, $4::timestamptz, NULL, NOW())
            RETURNING
                id,
                job_id,
                resume_id,
                status,
                applied_at::text AS applied_at,
                due_date::text AS due_date,
                updated_at::text AS updated_at
            "#,
        )
        .bind(Uuid::now_v7().to_string())
        .bind(&application.job_id)
        .bind(&application.status)
        .bind(&application.applied_at)
        .fetch_one(pool)
        .await?;

        Ok(Application::from(row))
    }

    pub async fn get_by_id(&self, id: &str) -> Result<Option<Application>, RepositoryError> {
        let Some(pool) = self.database.pool() else {
            return Err(RepositoryError::DatabaseDisabled);
        };

        let row = sqlx::query_as::<_, ApplicationRow>(
            r#"
            SELECT
                id,
                job_id,
                resume_id,
                status,
                applied_at::text AS applied_at,
                due_date::text AS due_date,
                updated_at::text AS updated_at
            FROM applications
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;

        Ok(row.map(Application::from))
    }

    pub async fn get_detail_by_id(
        &self,
        id: &str,
    ) -> Result<Option<ApplicationDetail>, RepositoryError> {
        let Some(pool) = self.database.pool() else {
            return Err(RepositoryError::DatabaseDisabled);
        };

        let row = sqlx::query_as::<_, ApplicationDetailRow>(
            r#"
            SELECT
                applications.id AS application_id,
                applications.job_id AS application_job_id,
                applications.resume_id AS application_resume_id,
                applications.status AS application_status,
                applications.applied_at::text AS application_applied_at,
                applications.due_date::text AS application_due_date,
                applications.updated_at::text AS application_updated_at,
                jobs.id AS job_id,
                jobs.title AS job_title,
                jobs.company_name AS job_company_name,
                jobs.remote_type AS job_remote_type,
                jobs.seniority AS job_seniority,
                jobs.description_text AS job_description_text,
                jobs.salary_min AS job_salary_min,
                jobs.salary_max AS job_salary_max,
                jobs.salary_currency AS job_salary_currency,
                jobs.posted_at::text AS job_posted_at,
                jobs.last_seen_at::text AS job_last_seen_at,
                jobs.is_active AS job_is_active,
                resumes.version AS resume_version,
                resumes.filename AS resume_filename,
                resumes.raw_text AS resume_raw_text,
                resumes.is_active AS resume_is_active,
                resumes.uploaded_at::text AS resume_uploaded_at
            FROM applications
            INNER JOIN jobs ON jobs.id = applications.job_id
            LEFT JOIN resumes ON resumes.id = applications.resume_id
            WHERE applications.id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;

        Ok(row.map(ApplicationDetail::from))
    }

    pub async fn list_recent(&self, limit: i64) -> Result<Vec<Application>, RepositoryError> {
        let Some(pool) = self.database.pool() else {
            return Err(RepositoryError::DatabaseDisabled);
        };

        let rows = sqlx::query_as::<_, ApplicationRow>(
            r#"
            SELECT
                id,
                job_id,
                resume_id,
                status,
                applied_at::text AS applied_at,
                due_date::text AS due_date,
                updated_at::text AS updated_at
            FROM applications
            ORDER BY updated_at DESC
            LIMIT $1
            "#,
        )
        .bind(limit)
        .fetch_all(pool)
        .await?;

        Ok(rows.into_iter().map(Application::from).collect())
    }

    pub async fn update(
        &self,
        id: &str,
        update: &UpdateApplication,
    ) -> Result<Option<Application>, RepositoryError> {
        let Some(pool) = self.database.pool() else {
            return Err(RepositoryError::DatabaseDisabled);
        };

        let row = sqlx::query_as::<_, ApplicationRow>(
            r#"
            UPDATE applications
            SET
                status = COALESCE($2, status),
                due_date = CASE
                    WHEN $3 THEN $4::timestamptz
                    ELSE due_date
                END,
                updated_at = NOW()
            WHERE id = $1
            RETURNING
                id,
                job_id,
                resume_id,
                status,
                applied_at::text AS applied_at,
                due_date::text AS due_date,
                updated_at::text AS updated_at
            "#,
        )
        .bind(id)
        .bind(&update.status)
        .bind(update.due_date.is_some())
        .bind(&update.due_date)
        .fetch_optional(pool)
        .await?;

        Ok(row.map(Application::from))
    }

    pub async fn attach_resume(
        &self,
        id: &str,
        resume_id: &str,
    ) -> Result<Option<Application>, RepositoryError> {
        let Some(pool) = self.database.pool() else {
            return Err(RepositoryError::DatabaseDisabled);
        };

        let row = sqlx::query_as::<_, ApplicationRow>(
            r#"
            UPDATE applications
            SET
                resume_id = $2,
                updated_at = NOW()
            WHERE id = $1
            RETURNING
                id,
                job_id,
                resume_id,
                status,
                applied_at::text AS applied_at,
                due_date::text AS due_date,
                updated_at::text AS updated_at
            "#,
        )
        .bind(id)
        .bind(resume_id)
        .fetch_optional(pool)
        .await?;

        Ok(row.map(Application::from))
    }
}

impl From<ApplicationRow> for Application {
    fn from(row: ApplicationRow) -> Self {
        Self {
            id: row.id,
            job_id: row.job_id,
            resume_id: row.resume_id,
            status: row.status,
            applied_at: row.applied_at,
            due_date: row.due_date,
            updated_at: row.updated_at,
        }
    }
}

impl From<ApplicationDetailRow> for ApplicationDetail {
    fn from(row: ApplicationDetailRow) -> Self {
        Self {
            application: Application {
                id: row.application_id,
                job_id: row.application_job_id,
                resume_id: row.application_resume_id.clone(),
                status: row.application_status,
                applied_at: row.application_applied_at,
                due_date: row.application_due_date,
                updated_at: row.application_updated_at,
            },
            job: Job {
                id: row.job_id,
                title: row.job_title,
                company_name: row.job_company_name,
                remote_type: row.job_remote_type,
                seniority: row.job_seniority,
                description_text: row.job_description_text,
                salary_min: row.job_salary_min,
                salary_max: row.job_salary_max,
                salary_currency: row.job_salary_currency,
                posted_at: row.job_posted_at,
                last_seen_at: row.job_last_seen_at,
                is_active: row.job_is_active,
            },
            resume: row.resume_version.map(|version| ResumeVersion {
                id: row
                    .application_resume_id
                    .expect("resume id should be present when resume is joined"),
                version,
                filename: row
                    .resume_filename
                    .expect("filename should be present when resume is joined"),
                raw_text: row
                    .resume_raw_text
                    .expect("raw_text should be present when resume is joined"),
                is_active: row
                    .resume_is_active
                    .expect("is_active should be present when resume is joined"),
                uploaded_at: row
                    .resume_uploaded_at
                    .expect("uploaded_at should be present when resume is joined"),
            }),
            notes: Vec::new(),
            contacts: Vec::new(),
            activities: Vec::new(),
            tasks: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::db::Database;
    use crate::db::repositories::{ApplicationsRepository, RepositoryError};
    use crate::domain::application::model::CreateApplication;

    #[tokio::test]
    async fn returns_disabled_error_without_database() {
        let repository = ApplicationsRepository::new(Database::disabled());

        let error = repository
            .get_by_id("application-1")
            .await
            .expect_err("repository should fail without configured database");

        assert!(matches!(error, RepositoryError::DatabaseDisabled));
    }

    #[tokio::test]
    async fn create_returns_disabled_without_database() {
        let repository = ApplicationsRepository::new(Database::disabled());

        let error = repository
            .create(&CreateApplication {
                job_id: "job-1".to_string(),
                status: "saved".to_string(),
                applied_at: None,
            })
            .await
            .expect_err("repository should fail without configured database");

        assert!(matches!(error, RepositoryError::DatabaseDisabled));
    }
}
