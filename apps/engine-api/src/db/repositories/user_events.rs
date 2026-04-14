use serde_json::Value;
use sqlx::FromRow;
use uuid::Uuid;

use crate::db::Database;
use crate::db::repositories::RepositoryError;
use crate::domain::user_event::model::{
    CreateUserEvent, UserEventRecord, UserEventSummary, UserEventType,
};

#[derive(Clone)]
pub struct UserEventsRepository {
    database: Database,
}

#[derive(FromRow)]
struct UserEventRow {
    id: String,
    profile_id: String,
    event_type: String,
    job_id: Option<String>,
    company_name: Option<String>,
    source: Option<String>,
    role_family: Option<String>,
    payload_json: Option<Value>,
    created_at: String,
}

#[derive(FromRow)]
struct UserEventSummaryRow {
    save_count: i64,
    hide_count: i64,
    bad_fit_count: i64,
    search_run_count: i64,
    fit_explanation_requested_count: i64,
    application_coach_requested_count: i64,
    cover_letter_draft_requested_count: i64,
    interview_prep_requested_count: i64,
}

impl UserEventsRepository {
    pub fn new(database: Database) -> Self {
        Self { database }
    }

    pub async fn create(
        &self,
        event: &CreateUserEvent,
    ) -> Result<UserEventRecord, RepositoryError> {
        let Some(pool) = self.database.pool() else {
            return Err(RepositoryError::DatabaseDisabled);
        };

        let row = sqlx::query_as::<_, UserEventRow>(
            r#"
            INSERT INTO user_events (
                id,
                profile_id,
                event_type,
                job_id,
                company_name,
                source,
                role_family,
                payload_json,
                created_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, NOW())
            RETURNING
                id,
                profile_id,
                event_type,
                job_id,
                company_name,
                source,
                role_family,
                payload_json,
                created_at::text AS created_at
            "#,
        )
        .bind(Uuid::now_v7().to_string())
        .bind(&event.profile_id)
        .bind(event.event_type.as_str())
        .bind(&event.job_id)
        .bind(&event.company_name)
        .bind(&event.source)
        .bind(&event.role_family)
        .bind(&event.payload_json)
        .fetch_one(pool)
        .await?;

        UserEventRecord::try_from(row)
    }

    pub async fn list_by_profile(
        &self,
        profile_id: &str,
    ) -> Result<Vec<UserEventRecord>, RepositoryError> {
        let Some(pool) = self.database.pool() else {
            return Err(RepositoryError::DatabaseDisabled);
        };

        let rows = sqlx::query_as::<_, UserEventRow>(
            r#"
            SELECT
                id,
                profile_id,
                event_type,
                job_id,
                company_name,
                source,
                role_family,
                payload_json,
                created_at::text AS created_at
            FROM user_events
            WHERE profile_id = $1
            ORDER BY created_at DESC, id DESC
            "#,
        )
        .bind(profile_id)
        .fetch_all(pool)
        .await?;

        rows.into_iter()
            .map(UserEventRecord::try_from)
            .collect::<Result<Vec<_>, _>>()
    }

    pub async fn summary_by_profile(
        &self,
        profile_id: &str,
    ) -> Result<UserEventSummary, RepositoryError> {
        let Some(pool) = self.database.pool() else {
            return Err(RepositoryError::DatabaseDisabled);
        };

        let row = sqlx::query_as::<_, UserEventSummaryRow>(
            r#"
            SELECT
                COUNT(*) FILTER (WHERE event_type = 'job_saved')::bigint AS save_count,
                COUNT(*) FILTER (WHERE event_type = 'job_hidden')::bigint AS hide_count,
                COUNT(*) FILTER (WHERE event_type = 'job_bad_fit')::bigint AS bad_fit_count,
                COUNT(*) FILTER (WHERE event_type = 'search_run')::bigint AS search_run_count,
                COUNT(*) FILTER (WHERE event_type = 'fit_explanation_requested')::bigint AS fit_explanation_requested_count,
                COUNT(*) FILTER (WHERE event_type = 'application_coach_requested')::bigint AS application_coach_requested_count,
                COUNT(*) FILTER (WHERE event_type = 'cover_letter_draft_requested')::bigint AS cover_letter_draft_requested_count,
                COUNT(*) FILTER (WHERE event_type = 'interview_prep_requested')::bigint AS interview_prep_requested_count
            FROM user_events
            WHERE profile_id = $1
            "#,
        )
        .bind(profile_id)
        .fetch_one(pool)
        .await?;

        Ok(UserEventSummary {
            save_count: row.save_count as usize,
            hide_count: row.hide_count as usize,
            bad_fit_count: row.bad_fit_count as usize,
            search_run_count: row.search_run_count as usize,
            fit_explanation_requested_count: row.fit_explanation_requested_count as usize,
            application_coach_requested_count: row.application_coach_requested_count as usize,
            cover_letter_draft_requested_count: row.cover_letter_draft_requested_count as usize,
            interview_prep_requested_count: row.interview_prep_requested_count as usize,
        })
    }
}

impl TryFrom<UserEventRow> for UserEventRecord {
    type Error = RepositoryError;

    fn try_from(row: UserEventRow) -> Result<Self, Self::Error> {
        let event_type =
            UserEventType::parse(&row.event_type).ok_or_else(|| RepositoryError::InvalidData {
                message: format!("unknown user event type '{}'", row.event_type),
            })?;

        Ok(Self {
            id: row.id,
            profile_id: row.profile_id,
            event_type,
            job_id: row.job_id,
            company_name: row.company_name,
            source: row.source,
            role_family: row.role_family,
            payload_json: row.payload_json,
            created_at: row.created_at,
        })
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::db::Database;
    use crate::db::repositories::{RepositoryError, UserEventsRepository};
    use crate::domain::user_event::model::{CreateUserEvent, UserEventType};

    #[tokio::test]
    async fn create_returns_disabled_without_database() {
        let repository = UserEventsRepository::new(Database::disabled());

        let error = repository
            .create(&CreateUserEvent {
                profile_id: "profile-1".to_string(),
                event_type: UserEventType::JobSaved,
                job_id: Some("job-1".to_string()),
                company_name: Some("NovaLedger".to_string()),
                source: Some("djinni".to_string()),
                role_family: None,
                payload_json: Some(json!({ "surface": "dashboard" })),
            })
            .await
            .expect_err("repository should fail without configured database");

        assert!(matches!(error, RepositoryError::DatabaseDisabled));
    }

    #[tokio::test]
    async fn list_returns_disabled_without_database() {
        let repository = UserEventsRepository::new(Database::disabled());

        let error = repository
            .list_by_profile("profile-1")
            .await
            .expect_err("repository should fail without configured database");

        assert!(matches!(error, RepositoryError::DatabaseDisabled));
    }
}
