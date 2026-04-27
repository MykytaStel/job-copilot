use std::fmt;

use serde_json::Value;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NotificationType {
    NewJobsFound,
    JobReactivated,
    ApplicationDueSoon,
}

impl NotificationType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::NewJobsFound => "new_jobs_found",
            Self::JobReactivated => "job_reactivated",
            Self::ApplicationDueSoon => "application_due_soon",
        }
    }
}

impl TryFrom<&str> for NotificationType {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.trim() {
            "new_jobs_found" => Ok(Self::NewJobsFound),
            "job_reactivated" => Ok(Self::JobReactivated),
            "application_due_soon" => Ok(Self::ApplicationDueSoon),
            other => Err(format!("unsupported notification type '{other}'")),
        }
    }
}

impl fmt::Display for NotificationType {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Notification {
    pub id: String,
    pub profile_id: String,
    pub notification_type: NotificationType,
    pub title: String,
    pub body: Option<String>,
    pub payload: Option<Value>,
    pub read_at: Option<String>,
    pub created_at: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NotificationPreferences {
    pub profile_id: String,
    pub new_jobs_matching_profile: bool,
    pub application_status_reminders: bool,
    pub weekly_digest: bool,
    pub market_intelligence_updates: bool,
    pub created_at: String,
    pub updated_at: String,
}
