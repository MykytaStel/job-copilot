use sqlx::FromRow;
use uuid::Uuid;

use crate::db::Database;
use crate::db::repositories::RepositoryError;
use crate::domain::application::model::{Activity, CreateActivity};

#[derive(Clone)]
pub struct ActivitiesRepository {
    database: Database,
}

#[derive(FromRow)]
struct ActivityRow {
    id: String,
    application_id: String,
    activity_type: String,
    description: String,
    happened_at: String,
    created_at: String,
}

impl ActivitiesRepository {
    pub fn new(database: Database) -> Self {
        Self { database }
    }

    pub async fn create(&self, activity: &CreateActivity) -> Result<Activity, RepositoryError> {
        let Some(pool) = self.database.pool() else {
            return Err(RepositoryError::DatabaseDisabled);
        };

        let row = sqlx::query_as::<_, ActivityRow>(
            r#"
            INSERT INTO activities (id, application_id, activity_type, description, happened_at, created_at)
            VALUES ($1, $2, $3, $4, $5::timestamptz, NOW())
            RETURNING
                id,
                application_id,
                activity_type,
                description,
                happened_at::text AS happened_at,
                created_at::text AS created_at
            "#,
        )
        .bind(Uuid::now_v7().to_string())
        .bind(&activity.application_id)
        .bind(&activity.activity_type)
        .bind(&activity.description)
        .bind(&activity.happened_at)
        .fetch_one(pool)
        .await?;

        Ok(Activity::from(row))
    }
}

impl From<ActivityRow> for Activity {
    fn from(row: ActivityRow) -> Self {
        Self {
            id: row.id,
            application_id: row.application_id,
            activity_type: row.activity_type,
            description: row.description,
            happened_at: row.happened_at,
            created_at: row.created_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::db::Database;
    use crate::db::repositories::RepositoryError;

    use super::ActivitiesRepository;

    #[tokio::test]
    async fn returns_disabled_error_without_database() {
        let repository = ActivitiesRepository::new(Database::disabled());

        let error = repository
            .create(&crate::domain::application::model::CreateActivity {
                application_id: "app-1".to_string(),
                activity_type: "interview".to_string(),
                description: "Phone screen".to_string(),
                happened_at: "2026-04-11T10:00:00Z".to_string(),
            })
            .await
            .expect_err("repository should fail without configured database");

        assert!(matches!(error, RepositoryError::DatabaseDisabled));
    }
}
