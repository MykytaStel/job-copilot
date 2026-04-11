use sqlx::FromRow;
use sqlx::types::Json;
use uuid::Uuid;

use crate::db::Database;
use crate::db::repositories::RepositoryError;
use crate::domain::matching::model::MatchResult;

#[derive(Clone)]
pub struct MatchResultsRepository {
    database: Database,
}

#[derive(FromRow)]
struct MatchResultRow {
    id: String,
    job_id: String,
    resume_id: String,
    score: i32,
    matched_skills_json: String,
    missing_skills_json: String,
    notes: String,
    created_at: String,
}

impl MatchResultsRepository {
    pub fn new(database: Database) -> Self {
        Self { database }
    }

    pub async fn get_for_job_and_resume(
        &self,
        job_id: &str,
        resume_id: &str,
    ) -> Result<Option<MatchResult>, RepositoryError> {
        let Some(pool) = self.database.pool() else {
            return Err(RepositoryError::DatabaseDisabled);
        };

        let row = sqlx::query_as::<_, MatchResultRow>(
            r#"
            SELECT
                id,
                job_id,
                resume_id,
                score,
                matched_skills::text AS matched_skills_json,
                missing_skills::text AS missing_skills_json,
                notes,
                created_at::text AS created_at
            FROM match_results
            WHERE job_id = $1 AND resume_id = $2
            ORDER BY created_at DESC
            LIMIT 1
            "#,
        )
        .bind(job_id)
        .bind(resume_id)
        .fetch_optional(pool)
        .await?;

        row.map(MatchResult::try_from).transpose()
    }

    pub async fn save(&self, result: &MatchResult) -> Result<MatchResult, RepositoryError> {
        let Some(pool) = self.database.pool() else {
            return Err(RepositoryError::DatabaseDisabled);
        };

        let row = sqlx::query_as::<_, MatchResultRow>(
            r#"
            INSERT INTO match_results (
                id,
                job_id,
                resume_id,
                score,
                matched_skills,
                missing_skills,
                notes,
                created_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, NOW())
            RETURNING
                id,
                job_id,
                resume_id,
                score,
                matched_skills::text AS matched_skills_json,
                missing_skills::text AS missing_skills_json,
                notes,
                created_at::text AS created_at
            "#,
        )
        .bind(Uuid::now_v7().to_string())
        .bind(&result.job_id)
        .bind(&result.resume_id)
        .bind(result.score)
        .bind(Json(result.matched_skills.clone()))
        .bind(Json(result.missing_skills.clone()))
        .bind(&result.notes)
        .fetch_one(pool)
        .await?;

        MatchResult::try_from(row)
    }
}

impl TryFrom<MatchResultRow> for MatchResult {
    type Error = RepositoryError;

    fn try_from(row: MatchResultRow) -> Result<Self, Self::Error> {
        Ok(Self {
            id: row.id,
            job_id: row.job_id,
            resume_id: row.resume_id,
            score: row.score,
            matched_skills: serde_json::from_str(&row.matched_skills_json)?,
            missing_skills: serde_json::from_str(&row.missing_skills_json)?,
            notes: row.notes,
            created_at: row.created_at,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::db::Database;
    use crate::db::repositories::{MatchResultsRepository, RepositoryError};

    #[tokio::test]
    async fn returns_disabled_error_without_database() {
        let repository = MatchResultsRepository::new(Database::disabled());

        let error = repository
            .get_for_job_and_resume("job-1", "resume-1")
            .await
            .expect_err("repository should fail without configured database");

        assert!(matches!(error, RepositoryError::DatabaseDisabled));
    }
}
