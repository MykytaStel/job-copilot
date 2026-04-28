use sqlx::FromRow;
use uuid::Uuid;

use crate::db::Database;
use crate::db::repositories::RepositoryError;
use crate::domain::resume::model::{ResumeVersion, UploadResume};

#[derive(Clone)]
pub struct ResumesRepository {
    database: Database,
}

#[derive(FromRow)]
struct ResumeRow {
    id: String,
    version: i32,
    filename: String,
    raw_text: String,
    is_active: bool,
    uploaded_at: String,
}

impl ResumesRepository {
    pub fn new(database: Database) -> Self {
        Self { database }
    }

    pub async fn list(&self) -> Result<Vec<ResumeVersion>, RepositoryError> {
        let Some(pool) = self.database.pool() else {
            return Err(RepositoryError::DatabaseDisabled);
        };

        let rows = sqlx::query_as::<_, ResumeRow>(
            r#"
            SELECT
                id,
                version,
                filename,
                raw_text,
                is_active,
                uploaded_at::text AS uploaded_at
            FROM resumes
            ORDER BY version DESC
            "#,
        )
        .fetch_all(pool)
        .await?;

        Ok(rows.into_iter().map(ResumeVersion::from).collect())
    }

    pub async fn get_active(&self) -> Result<Option<ResumeVersion>, RepositoryError> {
        let Some(pool) = self.database.pool() else {
            return Err(RepositoryError::DatabaseDisabled);
        };

        let row = sqlx::query_as::<_, ResumeRow>(
            r#"
            SELECT
                id,
                version,
                filename,
                raw_text,
                is_active,
                uploaded_at::text AS uploaded_at
            FROM resumes
            WHERE is_active = TRUE
            ORDER BY version DESC
            LIMIT 1
            "#,
        )
        .fetch_optional(pool)
        .await?;

        Ok(row.map(ResumeVersion::from))
    }

    pub async fn upload(&self, input: &UploadResume) -> Result<ResumeVersion, RepositoryError> {
        let Some(pool) = self.database.pool() else {
            return Err(RepositoryError::DatabaseDisabled);
        };

        sqlx::query("UPDATE resumes SET is_active = FALSE WHERE is_active = TRUE")
            .execute(pool)
            .await?;

        let row = sqlx::query_as::<_, ResumeRow>(
            r#"
            INSERT INTO resumes (
                id,
                version,
                filename,
                raw_text,
                is_active,
                uploaded_at
            )
            VALUES (
                $1,
                (SELECT COALESCE(MAX(version), 0) + 1 FROM resumes),
                $2,
                $3,
                TRUE,
                NOW()
            )
            RETURNING
                id,
                version,
                filename,
                raw_text,
                is_active,
                uploaded_at::text AS uploaded_at
            "#,
        )
        .bind(Uuid::now_v7().to_string())
        .bind(&input.filename)
        .bind(&input.raw_text)
        .fetch_one(pool)
        .await?;

        Ok(ResumeVersion::from(row))
    }

    pub async fn activate(&self, id: &str) -> Result<Option<ResumeVersion>, RepositoryError> {
        let Some(pool) = self.database.pool() else {
            return Err(RepositoryError::DatabaseDisabled);
        };

        sqlx::query("UPDATE resumes SET is_active = FALSE WHERE is_active = TRUE")
            .execute(pool)
            .await?;

        let row = sqlx::query_as::<_, ResumeRow>(
            r#"
            UPDATE resumes
            SET is_active = TRUE
            WHERE id = $1
            RETURNING
                id,
                version,
                filename,
                raw_text,
                is_active,
                uploaded_at::text AS uploaded_at
            "#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;

        Ok(row.map(ResumeVersion::from))
    }

    pub async fn delete(&self, id: &str) -> Result<bool, RepositoryError> {
        let Some(pool) = self.database.pool() else {
            return Err(RepositoryError::DatabaseDisabled);
        };

        let result = sqlx::query("DELETE FROM resumes WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }
}

impl From<ResumeRow> for ResumeVersion {
    fn from(row: ResumeRow) -> Self {
        Self {
            id: row.id,
            version: row.version,
            filename: row.filename,
            raw_text: row.raw_text,
            is_active: row.is_active,
            uploaded_at: row.uploaded_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::db::Database;
    use crate::db::repositories::{RepositoryError, ResumesRepository};

    #[tokio::test]
    async fn returns_disabled_error_without_database() {
        let repository = ResumesRepository::new(Database::disabled());

        let error = repository
            .list()
            .await
            .expect_err("repository should fail without configured database");

        assert!(matches!(error, RepositoryError::DatabaseDisabled));
    }
}
