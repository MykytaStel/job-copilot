use sqlx::types::Json;
use uuid::Uuid;

use crate::db::Database;
use crate::db::repositories::RepositoryError;
use crate::domain::ranking::FitScore;

#[derive(Clone)]
pub struct FitScoresRepository {
    database: Database,
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
