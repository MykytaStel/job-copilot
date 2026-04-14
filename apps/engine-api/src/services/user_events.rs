#[cfg(test)]
use std::sync::{Arc, Mutex};

use crate::db::repositories::{RepositoryError, UserEventsRepository};
use crate::domain::user_event::model::{CreateUserEvent, UserEventRecord, UserEventSummary};

#[derive(Clone)]
enum UserEventsServiceBackend {
    Repository(UserEventsRepository),
    #[cfg(test)]
    Stub(Arc<UserEventsServiceStub>),
}

#[derive(Clone)]
pub struct UserEventsService {
    backend: UserEventsServiceBackend,
}

impl UserEventsService {
    pub fn new(repository: UserEventsRepository) -> Self {
        Self {
            backend: UserEventsServiceBackend::Repository(repository),
        }
    }

    pub async fn log_event(
        &self,
        event: CreateUserEvent,
    ) -> Result<UserEventRecord, RepositoryError> {
        match &self.backend {
            UserEventsServiceBackend::Repository(repository) => repository.create(&event).await,
            #[cfg(test)]
            UserEventsServiceBackend::Stub(stub) => stub.log_event(event),
        }
    }

    pub async fn list_by_profile(
        &self,
        profile_id: &str,
    ) -> Result<Vec<UserEventRecord>, RepositoryError> {
        match &self.backend {
            UserEventsServiceBackend::Repository(repository) => {
                repository.list_by_profile(profile_id).await
            }
            #[cfg(test)]
            UserEventsServiceBackend::Stub(stub) => stub.list_by_profile(profile_id),
        }
    }

    pub async fn summary_by_profile(
        &self,
        profile_id: &str,
    ) -> Result<UserEventSummary, RepositoryError> {
        match &self.backend {
            UserEventsServiceBackend::Repository(repository) => {
                repository.summary_by_profile(profile_id).await
            }
            #[cfg(test)]
            UserEventsServiceBackend::Stub(stub) => stub.summary_by_profile(profile_id),
        }
    }

    #[cfg(test)]
    pub fn for_tests(stub: UserEventsServiceStub) -> Self {
        Self {
            backend: UserEventsServiceBackend::Stub(Arc::new(stub)),
        }
    }
}

#[cfg(test)]
#[derive(Default)]
pub struct UserEventsServiceStub {
    records: Mutex<Vec<UserEventRecord>>,
    database_disabled: bool,
}

#[cfg(test)]
impl UserEventsServiceStub {
    pub fn with_event(self, record: UserEventRecord) -> Self {
        self.records
            .lock()
            .expect("user events stub mutex should not be poisoned")
            .push(record);
        self
    }

    pub fn with_database_disabled(mut self) -> Self {
        self.database_disabled = true;
        self
    }

    fn log_event(&self, event: CreateUserEvent) -> Result<UserEventRecord, RepositoryError> {
        if self.database_disabled {
            return Err(RepositoryError::DatabaseDisabled);
        }

        let mut records = self
            .records
            .lock()
            .expect("user events stub mutex should not be poisoned");
        let record = UserEventRecord {
            id: format!("evt-{}", records.len() + 1),
            profile_id: event.profile_id,
            event_type: event.event_type,
            job_id: event.job_id,
            company_name: event.company_name,
            source: event.source,
            role_family: event.role_family,
            payload_json: event.payload_json,
            created_at: format!("2026-04-15T00:00:{:02}Z", records.len()),
        };
        records.push(record.clone());

        Ok(record)
    }

    fn list_by_profile(&self, profile_id: &str) -> Result<Vec<UserEventRecord>, RepositoryError> {
        if self.database_disabled {
            return Err(RepositoryError::DatabaseDisabled);
        }

        Ok(self
            .records
            .lock()
            .expect("user events stub mutex should not be poisoned")
            .iter()
            .filter(|record| record.profile_id == profile_id)
            .cloned()
            .collect())
    }

    fn summary_by_profile(&self, profile_id: &str) -> Result<UserEventSummary, RepositoryError> {
        if self.database_disabled {
            return Err(RepositoryError::DatabaseDisabled);
        }

        let records = self
            .records
            .lock()
            .expect("user events stub mutex should not be poisoned");

        Ok(UserEventSummary::from_events(
            records
                .iter()
                .filter(|record| record.profile_id == profile_id),
        ))
    }
}
