use crate::db::Database;
use crate::db::repositories::RepositoryError;
use crate::domain::job::model::Job;
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

    pub async fn search(&self, query: &str, limit: i64) -> Result<Vec<Job>, RepositoryError> {
        let Some(pool) = self.database.pool() else {
            return Err(RepositoryError::DatabaseDisabled);
        };

        let pattern = format!("%{}%", query);
        let rows = sqlx::query_as::<_, JobRow>(
            r#"
            SELECT
                id,
                title,
                company_name,
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
                title ILIKE $1
                OR company_name ILIKE $1
                OR description_text ILIKE $1
            ORDER BY last_seen_at DESC, posted_at DESC NULLS LAST
            LIMIT $2
            "#,
        )
        .bind(pattern)
        .bind(limit)
        .fetch_all(pool)
        .await?;

        Ok(rows.into_iter().map(Job::from).collect())
    }
}

impl From<JobRow> for Job {
    fn from(row: JobRow) -> Self {
        Self {
            id: row.id,
            title: row.title,
            company_name: row.company_name,
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

#[cfg(test)]
mod tests {
    use crate::db::Database;
    use crate::db::repositories::{JobsRepository, RepositoryError};

    #[tokio::test]
    async fn returns_disabled_error_without_database() {
        let repository = JobsRepository::new(Database::disabled());

        let error = repository
            .list_recent(10)
            .await
            .expect_err("repository should fail without configured database");

        assert!(matches!(error, RepositoryError::DatabaseDisabled));
    }
}
