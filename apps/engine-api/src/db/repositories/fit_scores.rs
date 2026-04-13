use sqlx::types::Json;
use sqlx::FromRow;
use uuid::Uuid;

use crate::db::Database;
use crate::db::repositories::RepositoryError;
use crate::domain::ranking::{FitScore, FitScoreComponents};

#[derive(Clone)]
pub struct FitScoresRepository {
    database: Database,
}

#[derive(FromRow)]
struct FitScoreRow {
    job_id: String,
    total: i32,
    skill_overlap: f32,
    seniority_alignment: f32,
    salary_overlap: f32,
    work_mode_match: f32,
    matched_skills_json: String,
    missing_skills_json: String,
}

impl TryFrom<FitScoreRow> for FitScore {
    type Error = RepositoryError;

    fn try_from(row: FitScoreRow) -> Result<Self, Self::Error> {
        Ok(Self {
            job_id: row.job_id,
            total: row.total.clamp(0, 100) as u8,
            components: FitScoreComponents {
                skill_overlap: row.skill_overlap,
                seniority_alignment: row.seniority_alignment,
                salary_overlap: row.salary_overlap,
                work_mode_match: row.work_mode_match,
                recency_bonus: 0.5, // not persisted; default to neutral
            },
            matched_skills: serde_json::from_str(&row.matched_skills_json)?,
            missing_skills: serde_json::from_str(&row.missing_skills_json)?,
        })
    }
}

impl FitScoresRepository {
    pub fn new(database: Database) -> Self {
        Self { database }
    }

    /// Upsert a computed fit score for a (job, resume) pair.
    /// Safe to call on every GET /jobs/{id}/fit — the ON CONFLICT clause refreshes in place.
    pub async fn upsert(&self, score: &FitScore, resume_id: &str) -> Result<(), RepositoryError> {
        let Some(pool) = self.database.pool() else {
            return Err(RepositoryError::DatabaseDisabled);
        };

        sqlx::query(
            r#"
            INSERT INTO fit_scores (
                id, job_id, resume_id, total,
                skill_overlap, seniority_alignment, salary_overlap, work_mode_match,
                matched_skills, missing_skills, computed_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, NOW())
            ON CONFLICT (job_id, resume_id) DO UPDATE SET
                total                = EXCLUDED.total,
                skill_overlap        = EXCLUDED.skill_overlap,
                seniority_alignment  = EXCLUDED.seniority_alignment,
                salary_overlap       = EXCLUDED.salary_overlap,
                work_mode_match      = EXCLUDED.work_mode_match,
                matched_skills       = EXCLUDED.matched_skills,
                missing_skills       = EXCLUDED.missing_skills,
                computed_at          = NOW()
            "#,
        )
        .bind(Uuid::now_v7().to_string())
        .bind(&score.job_id)
        .bind(resume_id)
        .bind(score.total as i32)
        .bind(score.components.skill_overlap)
        .bind(score.components.seniority_alignment)
        .bind(score.components.salary_overlap)
        .bind(score.components.work_mode_match)
        .bind(Json(score.matched_skills.clone()))
        .bind(Json(score.missing_skills.clone()))
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn get_for_job_and_resume(
        &self,
        job_id: &str,
        resume_id: &str,
    ) -> Result<Option<FitScore>, RepositoryError> {
        let Some(pool) = self.database.pool() else {
            return Err(RepositoryError::DatabaseDisabled);
        };

        let row = sqlx::query_as::<_, FitScoreRow>(
            r#"
            SELECT
                job_id,
                total,
                skill_overlap,
                seniority_alignment,
                salary_overlap,
                work_mode_match,
                matched_skills::text AS matched_skills_json,
                missing_skills::text AS missing_skills_json
            FROM fit_scores
            WHERE job_id = $1 AND resume_id = $2
            "#,
        )
        .bind(job_id)
        .bind(resume_id)
        .fetch_optional(pool)
        .await?;

        row.map(FitScore::try_from).transpose()
    }
}

#[cfg(test)]
mod tests {
    use crate::db::Database;
    use crate::db::repositories::{FitScoresRepository, RepositoryError};
    use crate::domain::ranking::{FitScore, FitScoreComponents};

    #[tokio::test]
    async fn returns_disabled_error_without_database() {
        let repository = FitScoresRepository::new(Database::disabled());

        let score = FitScore {
            job_id: "job-1".to_string(),
            total: 75,
            components: FitScoreComponents {
                skill_overlap: 0.8,
                seniority_alignment: 0.7,
                salary_overlap: 0.5,
                work_mode_match: 0.5,
                recency_bonus: 0.5,
            },
            matched_skills: vec![],
            missing_skills: vec![],
        };

        let error = repository
            .upsert(&score, "resume-1")
            .await
            .expect_err("repository should fail without configured database");

        assert!(matches!(error, RepositoryError::DatabaseDisabled));
    }
}
