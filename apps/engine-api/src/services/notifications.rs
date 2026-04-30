#[cfg(test)]
use std::collections::HashMap;
#[cfg(test)]
use std::sync::{Arc, Mutex};

use crate::db::repositories::{NotificationsRepository, RepositoryError};
use crate::domain::notification::model::DueTaskNotification;
use crate::domain::notification::model::Notification;
use crate::domain::notification::model::NotificationPreferences;

#[allow(dead_code)]
pub mod job_alert_scoring {
    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../shared/rust/job_alert_scoring.rs"
    ));
}

#[derive(Clone)]
enum NotificationsServiceBackend {
    Repository(NotificationsRepository),
    #[cfg(test)]
    Stub(Arc<NotificationsServiceStub>),
}

#[derive(Clone)]
pub struct NotificationsService {
    backend: NotificationsServiceBackend,
}

impl NotificationsService {
    pub fn new(repository: NotificationsRepository) -> Self {
        Self {
            backend: NotificationsServiceBackend::Repository(repository),
        }
    }

    pub async fn list_by_profile(
        &self,
        profile_id: &str,
        limit: i64,
    ) -> Result<Vec<Notification>, RepositoryError> {
        match &self.backend {
            NotificationsServiceBackend::Repository(repository) => {
                repository.list_by_profile(profile_id, limit).await
            }
            #[cfg(test)]
            NotificationsServiceBackend::Stub(stub) => stub.list_by_profile(profile_id, limit),
        }
    }

    pub async fn mark_read(&self, id: &str) -> Result<Option<Notification>, RepositoryError> {
        match &self.backend {
            NotificationsServiceBackend::Repository(repository) => repository.mark_read(id).await,
            #[cfg(test)]
            NotificationsServiceBackend::Stub(stub) => stub.mark_read(id),
        }
    }

    pub async fn get_by_id(&self, id: &str) -> Result<Option<Notification>, RepositoryError> {
        match &self.backend {
            NotificationsServiceBackend::Repository(repository) => repository.get_by_id(id).await,
            #[cfg(test)]
            NotificationsServiceBackend::Stub(stub) => stub.get_by_id(id),
        }
    }

    pub async fn unread_count(&self, profile_id: &str) -> Result<i64, RepositoryError> {
        match &self.backend {
            NotificationsServiceBackend::Repository(repository) => {
                repository.unread_count(profile_id).await
            }
            #[cfg(test)]
            NotificationsServiceBackend::Stub(stub) => stub.unread_count(profile_id),
        }
    }

    pub async fn materialize_due_task_notifications(
        &self,
        profile_id: &str,
    ) -> Result<Vec<DueTaskNotification>, RepositoryError> {
        match &self.backend {
            NotificationsServiceBackend::Repository(repository) => {
                repository.insert_due_task_notifications(profile_id).await
            }
            #[cfg(test)]
            NotificationsServiceBackend::Stub(stub) => {
                stub.materialize_due_task_notifications(profile_id)
            }
        }
    }

    pub async fn get_preferences(
        &self,
        profile_id: &str,
    ) -> Result<NotificationPreferences, RepositoryError> {
        match &self.backend {
            NotificationsServiceBackend::Repository(repository) => {
                repository.get_preferences(profile_id).await
            }
            #[cfg(test)]
            NotificationsServiceBackend::Stub(stub) => stub.get_preferences(profile_id),
        }
    }

    pub async fn update_preferences(
        &self,
        profile_id: &str,
        new_jobs_matching_profile: Option<bool>,
        application_status_reminders: Option<bool>,
        weekly_digest: Option<bool>,
        market_intelligence_updates: Option<bool>,
    ) -> Result<NotificationPreferences, RepositoryError> {
        match &self.backend {
            NotificationsServiceBackend::Repository(repository) => {
                repository
                    .update_preferences(
                        profile_id,
                        new_jobs_matching_profile,
                        application_status_reminders,
                        weekly_digest,
                        market_intelligence_updates,
                    )
                    .await
            }
            #[cfg(test)]
            NotificationsServiceBackend::Stub(stub) => stub.update_preferences(
                profile_id,
                new_jobs_matching_profile,
                application_status_reminders,
                weekly_digest,
                market_intelligence_updates,
            ),
        }
    }

    #[cfg(test)]
    pub fn for_tests(stub: NotificationsServiceStub) -> Self {
        Self {
            backend: NotificationsServiceBackend::Stub(Arc::new(stub)),
        }
    }
}

#[cfg(test)]
#[derive(Default)]
pub struct NotificationsServiceStub {
    notifications_by_id: Mutex<HashMap<String, Notification>>,
    database_disabled: bool,
    preferences_by_profile_id: Mutex<HashMap<String, NotificationPreferences>>,
}

#[cfg(test)]
impl NotificationsServiceStub {
    pub fn with_notification(self, notification: Notification) -> Self {
        self.notifications_by_id
            .lock()
            .expect("notifications stub mutex should not be poisoned")
            .insert(notification.id.clone(), notification);
        self
    }

    fn list_by_profile(
        &self,
        profile_id: &str,
        limit: i64,
    ) -> Result<Vec<Notification>, RepositoryError> {
        if self.database_disabled {
            return Err(RepositoryError::DatabaseDisabled);
        }

        let mut notifications = self
            .notifications_by_id
            .lock()
            .expect("notifications stub mutex should not be poisoned")
            .values()
            .filter(|notification| notification.profile_id == profile_id)
            .cloned()
            .collect::<Vec<_>>();

        notifications.sort_by(|left, right| right.created_at.cmp(&left.created_at));
        notifications.truncate(limit as usize);

        Ok(notifications)
    }

    fn mark_read(&self, id: &str) -> Result<Option<Notification>, RepositoryError> {
        if self.database_disabled {
            return Err(RepositoryError::DatabaseDisabled);
        }

        let mut notifications = self
            .notifications_by_id
            .lock()
            .expect("notifications stub mutex should not be poisoned");
        let Some(notification) = notifications.get_mut(id) else {
            return Ok(None);
        };

        if notification.read_at.is_none() {
            notification.read_at = Some("2026-04-19T00:00:00Z".to_string());
        }

        Ok(Some(notification.clone()))
    }

    fn get_by_id(&self, id: &str) -> Result<Option<Notification>, RepositoryError> {
        if self.database_disabled {
            return Err(RepositoryError::DatabaseDisabled);
        }

        Ok(self
            .notifications_by_id
            .lock()
            .expect("notifications stub mutex should not be poisoned")
            .get(id)
            .cloned())
    }

    fn unread_count(&self, profile_id: &str) -> Result<i64, RepositoryError> {
        if self.database_disabled {
            return Err(RepositoryError::DatabaseDisabled);
        }

        Ok(self
            .notifications_by_id
            .lock()
            .expect("notifications stub mutex should not be poisoned")
            .values()
            .filter(|notification| {
                notification.profile_id == profile_id && notification.read_at.is_none()
            })
            .count() as i64)
    }

    fn materialize_due_task_notifications(
        &self,
        _profile_id: &str,
    ) -> Result<Vec<DueTaskNotification>, RepositoryError> {
        if self.database_disabled {
            return Err(RepositoryError::DatabaseDisabled);
        }

        Ok(Vec::new())
    }

    fn default_preferences(profile_id: &str) -> NotificationPreferences {
        NotificationPreferences {
            profile_id: profile_id.to_string(),
            new_jobs_matching_profile: true,
            application_status_reminders: true,
            weekly_digest: true,
            market_intelligence_updates: true,
            created_at: "2026-04-27T00:00:00Z".to_string(),
            updated_at: "2026-04-27T00:00:00Z".to_string(),
        }
    }

    fn get_preferences(
        &self,
        profile_id: &str,
    ) -> Result<NotificationPreferences, RepositoryError> {
        if self.database_disabled {
            return Err(RepositoryError::DatabaseDisabled);
        }

        let mut preferences = self
            .preferences_by_profile_id
            .lock()
            .expect("notification preferences stub mutex should not be poisoned");

        let entry = preferences
            .entry(profile_id.to_string())
            .or_insert_with(|| Self::default_preferences(profile_id));

        Ok(entry.clone())
    }

    fn update_preferences(
        &self,
        profile_id: &str,
        new_jobs_matching_profile: Option<bool>,
        application_status_reminders: Option<bool>,
        weekly_digest: Option<bool>,
        market_intelligence_updates: Option<bool>,
    ) -> Result<NotificationPreferences, RepositoryError> {
        if self.database_disabled {
            return Err(RepositoryError::DatabaseDisabled);
        }

        let mut preferences = self
            .preferences_by_profile_id
            .lock()
            .expect("notification preferences stub mutex should not be poisoned");

        let entry = preferences
            .entry(profile_id.to_string())
            .or_insert_with(|| Self::default_preferences(profile_id));

        if let Some(value) = new_jobs_matching_profile {
            entry.new_jobs_matching_profile = value;
        }

        if let Some(value) = application_status_reminders {
            entry.application_status_reminders = value;
        }

        if let Some(value) = weekly_digest {
            entry.weekly_digest = value;
        }

        if let Some(value) = market_intelligence_updates {
            entry.market_intelligence_updates = value;
        }

        entry.updated_at = "2026-04-27T01:00:00Z".to_string();

        Ok(entry.clone())
    }
}
