use sqlx::FromRow;
use uuid::Uuid;

use crate::db::Database;
use crate::db::repositories::RepositoryError;
use crate::domain::application::model::{CreateTask, Task};

#[derive(Clone)]
pub struct TasksRepository {
    database: Database,
}

#[derive(FromRow)]
struct TaskRow {
    id: String,
    application_id: String,
    title: String,
    remind_at: Option<String>,
    done: bool,
    created_at: String,
}

impl TasksRepository {
    pub fn new(database: Database) -> Self {
        Self { database }
    }

    /// Insert a follow-up task. `remind_in_days` is applied as `NOW() + N days`.
    pub async fn create(&self, task: &CreateTask) -> Result<Task, RepositoryError> {
        let Some(pool) = self.database.pool() else {
            return Err(RepositoryError::DatabaseDisabled);
        };

        let row = sqlx::query_as::<_, TaskRow>(
            r#"
            INSERT INTO tasks (id, application_id, title, remind_at, done, created_at)
            VALUES ($1, $2, $3, NOW() + ($4 * INTERVAL '1 day'), false, NOW())
            RETURNING
                id,
                application_id,
                title,
                remind_at::text AS remind_at,
                done,
                created_at::text AS created_at
            "#,
        )
        .bind(Uuid::now_v7().to_string())
        .bind(&task.application_id)
        .bind(&task.title)
        .bind(task.remind_in_days)
        .fetch_one(pool)
        .await?;

        Ok(Task::from(row))
    }

    /// Returns true if a task with the given title already exists for the application.
    /// Used to make follow-up task creation idempotent.
    pub async fn has_followup_task(
        &self,
        application_id: &str,
        title: &str,
    ) -> Result<bool, RepositoryError> {
        let Some(pool) = self.database.pool() else {
            return Err(RepositoryError::DatabaseDisabled);
        };

        let row: (bool,) = sqlx::query_as(
            r#"
            SELECT EXISTS (
                SELECT 1 FROM tasks
                WHERE application_id = $1 AND title = $2
            )
            "#,
        )
        .bind(application_id)
        .bind(title)
        .fetch_one(pool)
        .await?;

        Ok(row.0)
    }
}

impl From<TaskRow> for Task {
    fn from(row: TaskRow) -> Self {
        Self {
            id: row.id,
            application_id: row.application_id,
            title: row.title,
            remind_at: row.remind_at,
            done: row.done,
            created_at: row.created_at,
        }
    }
}
