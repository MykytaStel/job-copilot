use sqlx::FromRow;
use sqlx::types::Json;

use crate::db::Database;
use crate::db::repositories::RepositoryError;
use crate::domain::notification::model::NotificationPreferences;
use crate::domain::notification::model::{Notification, NotificationType};

#[derive(Clone)]
pub struct NotificationsRepository {
    database: Database,
}

#[derive(FromRow)]
struct NotificationRow {
    id: String,
    profile_id: String,
    r#type: String,
    title: String,
    body: Option<String>,
    payload: Option<Json<serde_json::Value>>,
    read_at: Option<String>,
    created_at: String,
}
#[derive(FromRow)]
struct NotificationPreferencesRow {
    profile_id: String,
    new_jobs_matching_profile: bool,
    application_status_reminders: bool,
    weekly_digest: bool,
    market_intelligence_updates: bool,
    created_at: String,
    updated_at: String,
}

impl NotificationsRepository {
    pub fn new(database: Database) -> Self {
        Self { database }
    }

    pub async fn list_by_profile(
        &self,
        profile_id: &str,
        limit: i64,
    ) -> Result<Vec<Notification>, RepositoryError> {
        let Some(pool) = self.database.pool() else {
            return Err(RepositoryError::DatabaseDisabled);
        };

        let rows = sqlx::query_as::<_, NotificationRow>(
            r#"
            SELECT
                id,
                profile_id,
                type,
                title,
                body,
                payload,
                read_at::text AS read_at,
                created_at::text AS created_at
            FROM notifications
            WHERE profile_id = $1
            ORDER BY created_at DESC
            LIMIT $2
            "#,
        )
        .bind(profile_id)
        .bind(limit)
        .fetch_all(pool)
        .await?;

        rows.into_iter().map(Notification::try_from).collect()
    }

    pub async fn mark_read(&self, id: &str) -> Result<Option<Notification>, RepositoryError> {
        let Some(pool) = self.database.pool() else {
            return Err(RepositoryError::DatabaseDisabled);
        };

        let row = sqlx::query_as::<_, NotificationRow>(
            r#"
            UPDATE notifications
            SET read_at = COALESCE(read_at, NOW())
            WHERE id = $1
            RETURNING
                id,
                profile_id,
                type,
                title,
                body,
                payload,
                read_at::text AS read_at,
                created_at::text AS created_at
            "#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;

        row.map(Notification::try_from).transpose()
    }

    pub async fn unread_count(&self, profile_id: &str) -> Result<i64, RepositoryError> {
        let Some(pool) = self.database.pool() else {
            return Err(RepositoryError::DatabaseDisabled);
        };

        sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COUNT(*)::bigint
            FROM notifications
            WHERE profile_id = $1
              AND read_at IS NULL
            "#,
        )
        .bind(profile_id)
        .fetch_one(pool)
        .await
        .map_err(RepositoryError::from)
    }
    pub async fn get_preferences(
        &self,
        profile_id: &str,
    ) -> Result<NotificationPreferences, RepositoryError> {
        let Some(pool) = self.database.pool() else {
            return Err(RepositoryError::DatabaseDisabled);
        };

        let row = sqlx::query_as::<_, NotificationPreferencesRow>(
            r#"
        INSERT INTO notification_preferences (profile_id)
        VALUES ($1)
        ON CONFLICT (profile_id) DO UPDATE
        SET profile_id = EXCLUDED.profile_id
        RETURNING
          profile_id,
          new_jobs_matching_profile,
          application_status_reminders,
          weekly_digest,
          market_intelligence_updates,
          created_at::text AS created_at,
          updated_at::text AS updated_at
        "#,
        )
        .bind(profile_id)
        .fetch_one(pool)
        .await?;

        Ok(row.into())
    }

    pub async fn update_preferences(
        &self,
        profile_id: &str,
        new_jobs_matching_profile: Option<bool>,
        application_status_reminders: Option<bool>,
        weekly_digest: Option<bool>,
        market_intelligence_updates: Option<bool>,
    ) -> Result<NotificationPreferences, RepositoryError> {
        let Some(pool) = self.database.pool() else {
            return Err(RepositoryError::DatabaseDisabled);
        };

        let row = sqlx::query_as::<_, NotificationPreferencesRow>(
        r#"
        INSERT INTO notification_preferences (
          profile_id,
          new_jobs_matching_profile,
          application_status_reminders,
          weekly_digest,
          market_intelligence_updates
        )
        VALUES (
          $1,
          COALESCE($2, TRUE),
          COALESCE($3, TRUE),
          COALESCE($4, TRUE),
          COALESCE($5, TRUE)
        )
        ON CONFLICT (profile_id) DO UPDATE
        SET
          new_jobs_matching_profile = COALESCE($2, notification_preferences.new_jobs_matching_profile),
          application_status_reminders = COALESCE($3, notification_preferences.application_status_reminders),
          weekly_digest = COALESCE($4, notification_preferences.weekly_digest),
          market_intelligence_updates = COALESCE($5, notification_preferences.market_intelligence_updates),
          updated_at = NOW()
        RETURNING
          profile_id,
          new_jobs_matching_profile,
          application_status_reminders,
          weekly_digest,
          market_intelligence_updates,
          created_at::text AS created_at,
          updated_at::text AS updated_at
        "#,
    )
    .bind(profile_id)
    .bind(new_jobs_matching_profile)
    .bind(application_status_reminders)
    .bind(weekly_digest)
    .bind(market_intelligence_updates)
    .fetch_one(pool)
    .await?;

        Ok(row.into())
    }
}

impl TryFrom<NotificationRow> for Notification {
    type Error = RepositoryError;

    fn try_from(row: NotificationRow) -> Result<Self, Self::Error> {
        let notification_type = NotificationType::try_from(row.r#type.as_str())
            .map_err(|message| RepositoryError::InvalidData { message })?;

        Ok(Self {
            id: row.id,
            profile_id: row.profile_id,
            notification_type,
            title: row.title,
            body: row.body,
            payload: row.payload.map(|payload| payload.0),
            read_at: row.read_at,
            created_at: row.created_at,
        })
    }
}

impl From<NotificationPreferencesRow> for NotificationPreferences {
    fn from(row: NotificationPreferencesRow) -> Self {
        Self {
            profile_id: row.profile_id,
            new_jobs_matching_profile: row.new_jobs_matching_profile,
            application_status_reminders: row.application_status_reminders,
            weekly_digest: row.weekly_digest,
            market_intelligence_updates: row.market_intelligence_updates,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}
