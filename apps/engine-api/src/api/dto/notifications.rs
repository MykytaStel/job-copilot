use serde::Serialize;

use crate::domain::notification::model::Notification;

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
