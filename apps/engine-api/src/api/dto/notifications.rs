use crate::domain::notification::model::Notification;
use crate::domain::notification::model::NotificationPreferences;
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct NotificationResponse {
    pub id: String,
    pub profile_id: String,
    pub r#type: String,
    pub title: String,
    pub body: Option<String>,
    pub payload: Option<serde_json::Value>,
    pub read_at: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
pub struct NotificationsResponse {
    pub notifications: Vec<NotificationResponse>,
}

#[derive(Debug, Serialize)]
pub struct UnreadNotificationsCountResponse {
    pub profile_id: String,
    pub unread_count: i64,
}

impl From<Notification> for NotificationResponse {
    fn from(notification: Notification) -> Self {
        Self {
            id: notification.id,
            profile_id: notification.profile_id,
            r#type: notification.notification_type.as_str().to_string(),
            title: notification.title,
            body: notification.body,
            payload: notification.payload,
            read_at: notification.read_at,
            created_at: notification.created_at,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct NotificationPreferencesResponse {
    pub profile_id: String,
    pub new_jobs_matching_profile: bool,
    pub application_status_reminders: bool,
    pub weekly_digest: bool,
    pub market_intelligence_updates: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateNotificationPreferencesRequest {
    pub new_jobs_matching_profile: Option<bool>,
    pub application_status_reminders: Option<bool>,
    pub weekly_digest: Option<bool>,
    pub market_intelligence_updates: Option<bool>,
}

impl From<NotificationPreferences> for NotificationPreferencesResponse {
    fn from(preferences: NotificationPreferences) -> Self {
        Self {
            profile_id: preferences.profile_id,
            new_jobs_matching_profile: preferences.new_jobs_matching_profile,
            application_status_reminders: preferences.application_status_reminders,
            weekly_digest: preferences.weekly_digest,
            market_intelligence_updates: preferences.market_intelligence_updates,
            created_at: preferences.created_at,
            updated_at: preferences.updated_at,
        }
    }
}
