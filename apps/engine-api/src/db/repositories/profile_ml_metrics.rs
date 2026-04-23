use sqlx::FromRow;
use sqlx::types::Json;
use uuid::Uuid;

use crate::db::Database;
use crate::db::repositories::RepositoryError;
use crate::domain::profile::ml::{CreateProfileMlMetric, ProfileMlMetricRecord};

#[derive(Clone)]
pub struct ProfileMlMetricsRepository {
    database: Database,
}

#[derive(FromRow)]
struct ProfileMlMetricRow {
    id: String,
    profile_id: String,
    retrained_at: String,
    status: String,
    artifact_version: Option<String>,
    model_type: Option<String>,
    reason: Option<String>,
    metrics_json: Option<Json<serde_json::Value>>,
    training_json: Option<Json<serde_json::Value>>,
    feature_importances_json: Option<Json<serde_json::Value>>,
    benchmark_json: Option<Json<serde_json::Value>>,
}

impl ProfileMlMetricsRepository {
    pub fn new(database: Database) -> Self {
        Self { database }
    }

    pub async fn create(
        &self,
        input: &CreateProfileMlMetric,
    ) -> Result<ProfileMlMetricRecord, RepositoryError> {
        let Some(pool) = self.database.pool() else {
            return Err(RepositoryError::DatabaseDisabled);
        };

        let row = sqlx::query_as::<_, ProfileMlMetricRow>(
            r#"
            INSERT INTO profile_ml_metrics (
                id,
                profile_id,
                retrained_at,
                status,
                artifact_version,
                model_type,
                reason,
                metrics_json,
                training_json,
                feature_importances_json,
                benchmark_json
            )
            VALUES ($1, $2, NOW(), $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING
                id,
                profile_id,
                retrained_at::text AS retrained_at,
                status,
                artifact_version,
                model_type,
                reason,
                metrics_json,
                training_json,
                feature_importances_json,
                benchmark_json
            "#,
        )
        .bind(Uuid::now_v7().to_string())
        .bind(&input.profile_id)
        .bind(&input.status)
        .bind(&input.artifact_version)
        .bind(&input.model_type)
        .bind(&input.reason)
        .bind(input.metrics_json.as_ref().cloned().map(Json))
        .bind(input.training_json.as_ref().cloned().map(Json))
        .bind(input.feature_importances_json.as_ref().cloned().map(Json))
        .bind(input.benchmark_json.as_ref().cloned().map(Json))
        .fetch_one(pool)
        .await?;

        Ok(ProfileMlMetricRecord::from(row))
    }

    pub async fn list_recent(
        &self,
        profile_id: &str,
        limit: usize,
    ) -> Result<Vec<ProfileMlMetricRecord>, RepositoryError> {
        let Some(pool) = self.database.pool() else {
            return Err(RepositoryError::DatabaseDisabled);
        };

        let rows = sqlx::query_as::<_, ProfileMlMetricRow>(
            r#"
            SELECT
                id,
                profile_id,
                retrained_at::text AS retrained_at,
                status,
                artifact_version,
                model_type,
                reason,
                metrics_json,
                training_json,
                feature_importances_json,
                benchmark_json
            FROM profile_ml_metrics
            WHERE profile_id = $1
            ORDER BY retrained_at DESC, id DESC
            LIMIT $2
            "#,
        )
        .bind(profile_id)
        .bind(limit as i64)
        .fetch_all(pool)
        .await?;

        Ok(rows.into_iter().map(ProfileMlMetricRecord::from).collect())
    }
}

impl From<ProfileMlMetricRow> for ProfileMlMetricRecord {
    fn from(row: ProfileMlMetricRow) -> Self {
        Self {
            id: row.id,
            profile_id: row.profile_id,
            retrained_at: row.retrained_at,
            status: row.status,
            artifact_version: row.artifact_version,
            model_type: row.model_type,
            reason: row.reason,
            metrics_json: row.metrics_json.map(|value| value.0),
            training_json: row.training_json.map(|value| value.0),
            feature_importances_json: row.feature_importances_json.map(|value| value.0),
            benchmark_json: row.benchmark_json.map(|value| value.0),
        }
    }
}
