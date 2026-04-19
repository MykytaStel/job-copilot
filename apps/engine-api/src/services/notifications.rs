#[cfg(test)]
use std::collections::HashMap;
#[cfg(test)]
use std::sync::{Arc, Mutex};

use crate::db::repositories::{NotificationsRepository, RepositoryError};
use crate::domain::notification::model::Notification;

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

    pub async fn unread_count(&self, profile_id: &str) -> Result<i64, RepositoryError> {
        match &self.backend {
            NotificationsServiceBackend::Repository(repository) => {
                repository.unread_count(profile_id).await
            }
            #[cfg(test)]
            NotificationsServiceBackend::Stub(stub) => stub.unread_count(profile_id),
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
}
