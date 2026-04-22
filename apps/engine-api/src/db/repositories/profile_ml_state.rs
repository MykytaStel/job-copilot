use sqlx::FromRow;

use crate::db::Database;
use crate::db::repositories::RepositoryError;
use crate::domain::profile::ml::{ProfileMlRetrainCandidate, ProfileMlState, UpdateProfileMlState};

#[derive(Clone)]
pub struct ProfileMlStateRepository {
    database: Database,
}

#[derive(FromRow)]
struct ProfileMlStateRow {
    id: String,
    ml_last_retrained_at: Option<String>,
    ml_examples_since_retrain: i32,
    ml_last_artifact_version: Option<String>,
    ml_last_training_status: Option<String>,
}

impl ProfileMlStateRepository {
    pub fn new(database: Database) -> Self {
        Self { database }
    }

    pub async fn get_by_profile_id(
        &self,
        profile_id: &str,
    ) -> Result<Option<ProfileMlState>, RepositoryError> {
        let Some(pool) = self.database.pool() else {
            return Err(RepositoryError::DatabaseDisabled);
        };

        let row = sqlx::query_as::<_, ProfileMlStateRow>(
            r#"
            SELECT
                id,
                ml_last_retrained_at::text AS ml_last_retrained_at,
                ml_examples_since_retrain,
                ml_last_artifact_version,
                ml_last_training_status
            FROM profiles
            WHERE id = $1
            "#,
        )
        .bind(profile_id)
        .fetch_optional(pool)
        .await?;

        Ok(row.map(ProfileMlState::from))
    }

    pub async fn record_labelable_job(
        &self,
        profile_id: &str,
        job_id: &str,
    ) -> Result<bool, RepositoryError> {
        let Some(pool) = self.database.pool() else {
            return Err(RepositoryError::DatabaseDisabled);
        };

        let mut tx = pool.begin().await?;
        let inserted = sqlx::query(
            r#"
            INSERT INTO profile_ml_labeled_jobs (profile_id, job_id, first_labeled_at)
            VALUES ($1, $2, NOW())
            ON CONFLICT (profile_id, job_id) DO NOTHING
            "#,
        )
        .bind(profile_id)
        .bind(job_id)
        .execute(&mut *tx)
        .await?
        .rows_affected()
            > 0;

        if inserted {
            sqlx::query(
                r#"
                UPDATE profiles
                SET ml_examples_since_retrain = ml_examples_since_retrain + 1
                WHERE id = $1
                "#,
            )
            .bind(profile_id)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(inserted)
    }

    pub async fn list_ready_for_retrain(
        &self,
        min_examples: usize,
    ) -> Result<Vec<ProfileMlRetrainCandidate>, RepositoryError> {
        let Some(pool) = self.database.pool() else {
            return Err(RepositoryError::DatabaseDisabled);
        };

        let rows = sqlx::query_as::<_, ProfileMlStateRow>(
            r#"
            SELECT
                id,
                ml_last_retrained_at::text AS ml_last_retrained_at,
                ml_examples_since_retrain,
                ml_last_artifact_version,
                ml_last_training_status
            FROM profiles
            WHERE ml_examples_since_retrain >= $1
            ORDER BY ml_examples_since_retrain DESC, updated_at DESC
            "#,
        )
        .bind(min_examples as i32)
        .fetch_all(pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| ProfileMlRetrainCandidate {
                profile_id: row.id,
                examples_since_retrain: row.ml_examples_since_retrain.max(0) as usize,
            })
            .collect())
    }

    pub async fn update_state(
        &self,
        profile_id: &str,
        update: &UpdateProfileMlState,
    ) -> Result<Option<ProfileMlState>, RepositoryError> {
        let Some(pool) = self.database.pool() else {
            return Err(RepositoryError::DatabaseDisabled);
        };

        let examples_since_retrain = update.examples_since_retrain.map(|value| value as i32);
        let row = sqlx::query_as::<_, ProfileMlStateRow>(
            r#"
            UPDATE profiles
            SET
                ml_last_retrained_at = CASE
                    WHEN $2 THEN $3::timestamptz
                    ELSE ml_last_retrained_at
                END,
                ml_examples_since_retrain = COALESCE($4, ml_examples_since_retrain),
                ml_last_artifact_version = CASE
                    WHEN $5 THEN $6
                    ELSE ml_last_artifact_version
                END,
                ml_last_training_status = CASE
                    WHEN $7 THEN $8
                    ELSE ml_last_training_status
                END
            WHERE id = $1
            RETURNING
                id,
                ml_last_retrained_at::text AS ml_last_retrained_at,
                ml_examples_since_retrain,
                ml_last_artifact_version,
                ml_last_training_status
            "#,
        )
        .bind(profile_id)
        .bind(update.last_retrained_at.is_some())
        .bind(update.last_retrained_at.as_ref().cloned().flatten())
        .bind(examples_since_retrain)
        .bind(update.last_artifact_version.is_some())
        .bind(update.last_artifact_version.as_ref().cloned().flatten())
        .bind(update.last_training_status.is_some())
        .bind(update.last_training_status.as_ref().cloned().flatten())
        .fetch_optional(pool)
        .await?;

        Ok(row.map(ProfileMlState::from))
    }
}

impl From<ProfileMlStateRow> for ProfileMlState {
    fn from(row: ProfileMlStateRow) -> Self {
        Self {
            profile_id: row.id,
            last_retrained_at: row.ml_last_retrained_at,
            examples_since_retrain: row.ml_examples_since_retrain.max(0) as usize,
            last_artifact_version: row.ml_last_artifact_version,
            last_training_status: row.ml_last_training_status,
        }
    }
}
