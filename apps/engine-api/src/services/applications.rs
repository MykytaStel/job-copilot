#[cfg(test)]
use std::collections::HashMap;
#[cfg(test)]
use std::sync::{Arc, Mutex};

use crate::db::repositories::{ApplicationsRepository, RepositoryError};
use crate::domain::application::model::{
    Application, ApplicationDetail, CreateApplication, UpdateApplication,
};

#[derive(Clone)]
enum ApplicationsServiceBackend {
    Repository(ApplicationsRepository),
    #[cfg(test)]
    Stub(Arc<ApplicationsServiceStub>),
}

#[derive(Clone)]
pub struct ApplicationsService {
    backend: ApplicationsServiceBackend,
}

impl ApplicationsService {
    pub fn new(repository: ApplicationsRepository) -> Self {
        Self {
            backend: ApplicationsServiceBackend::Repository(repository),
        }
    }

    pub async fn create(
        &self,
        application: CreateApplication,
    ) -> Result<Application, RepositoryError> {
        match &self.backend {
            ApplicationsServiceBackend::Repository(repository) => {
                repository.create(&application).await
            }
            #[cfg(test)]
            ApplicationsServiceBackend::Stub(stub) => stub.create(application),
        }
    }

    pub async fn get_by_id(&self, id: &str) -> Result<Option<Application>, RepositoryError> {
        match &self.backend {
            ApplicationsServiceBackend::Repository(repository) => repository.get_by_id(id).await,
            #[cfg(test)]
            ApplicationsServiceBackend::Stub(stub) => stub.get_by_id(id),
        }
    }

    pub async fn get_detail_by_id(
        &self,
        id: &str,
    ) -> Result<Option<ApplicationDetail>, RepositoryError> {
        match &self.backend {
            ApplicationsServiceBackend::Repository(repository) => {
                repository.get_detail_by_id(id).await
            }
            #[cfg(test)]
            ApplicationsServiceBackend::Stub(stub) => stub.get_detail_by_id(id),
        }
    }

    pub async fn list_recent(&self, limit: i64) -> Result<Vec<Application>, RepositoryError> {
        match &self.backend {
            ApplicationsServiceBackend::Repository(repository) => {
                repository.list_recent(limit).await
            }
            #[cfg(test)]
            ApplicationsServiceBackend::Stub(stub) => stub.list_recent(limit),
        }
    }

    pub async fn update(
        &self,
        id: &str,
        patch: UpdateApplication,
    ) -> Result<Option<Application>, RepositoryError> {
        match &self.backend {
            ApplicationsServiceBackend::Repository(repository) => {
                repository.update(id, &patch).await
            }
            #[cfg(test)]
            ApplicationsServiceBackend::Stub(stub) => stub.update(id, patch),
        }
    }

    pub async fn attach_resume(
        &self,
        id: &str,
        resume_id: &str,
    ) -> Result<Option<Application>, RepositoryError> {
        match &self.backend {
            ApplicationsServiceBackend::Repository(repository) => {
                repository.attach_resume(id, resume_id).await
            }
            #[cfg(test)]
            ApplicationsServiceBackend::Stub(stub) => stub.attach_resume(id, resume_id),
        }
    }

    #[cfg(test)]
    pub fn for_tests(stub: ApplicationsServiceStub) -> Self {
        Self {
            backend: ApplicationsServiceBackend::Stub(Arc::new(stub)),
        }
    }
}

#[cfg(test)]
#[derive(Default)]
pub struct ApplicationsServiceStub {
    applications_by_id: Mutex<HashMap<String, Application>>,
    recent_applications: Vec<Application>,
    details_by_id: Mutex<HashMap<String, ApplicationDetail>>,
    database_disabled: bool,
}

#[cfg(test)]
impl ApplicationsServiceStub {
    fn create(&self, application: CreateApplication) -> Result<Application, RepositoryError> {
        if self.database_disabled {
            return Err(RepositoryError::DatabaseDisabled);
        }

        let created = Application {
            id: "application_test_001".to_string(),
            job_id: application.job_id,
            resume_id: None,
            status: application.status,
            applied_at: application.applied_at,
            due_date: None,
            updated_at: "2026-04-11T00:00:00+00:00".to_string(),
        };

        self.applications_by_id
            .lock()
            .expect("applications stub mutex should not be poisoned")
            .insert(created.id.clone(), created.clone());

        Ok(created)
    }

    fn get_by_id(&self, id: &str) -> Result<Option<Application>, RepositoryError> {
        if self.database_disabled {
            return Err(RepositoryError::DatabaseDisabled);
        }

        Ok(self
            .applications_by_id
            .lock()
            .expect("applications stub mutex should not be poisoned")
            .get(id)
            .cloned())
    }

    fn get_detail_by_id(&self, id: &str) -> Result<Option<ApplicationDetail>, RepositoryError> {
        if self.database_disabled {
            return Err(RepositoryError::DatabaseDisabled);
        }

        Ok(self
            .details_by_id
            .lock()
            .expect("application details stub mutex should not be poisoned")
            .get(id)
            .cloned())
    }

    fn list_recent(&self, limit: i64) -> Result<Vec<Application>, RepositoryError> {
        if self.database_disabled {
            return Err(RepositoryError::DatabaseDisabled);
        }

        Ok(self
            .recent_applications
            .iter()
            .take(limit as usize)
            .cloned()
            .collect())
    }

    fn update(
        &self,
        id: &str,
        patch: UpdateApplication,
    ) -> Result<Option<Application>, RepositoryError> {
        if self.database_disabled {
            return Err(RepositoryError::DatabaseDisabled);
        }

        let mut applications = self
            .applications_by_id
            .lock()
            .expect("applications stub mutex should not be poisoned");
        let Some(application) = applications.get_mut(id) else {
            return Ok(None);
        };

        if let Some(status) = patch.status {
            application.status = status;
        }
        if let Some(due_date) = patch.due_date {
            application.due_date = due_date;
        }
        application.updated_at = "2026-04-11T00:00:01+00:00".to_string();

        Ok(Some(application.clone()))
    }

    fn attach_resume(
        &self,
        id: &str,
        resume_id: &str,
    ) -> Result<Option<Application>, RepositoryError> {
        if self.database_disabled {
            return Err(RepositoryError::DatabaseDisabled);
        }

        let mut applications = self
            .applications_by_id
            .lock()
            .expect("applications stub mutex should not be poisoned");
        let Some(application) = applications.get_mut(id) else {
            return Ok(None);
        };

        application.resume_id = Some(resume_id.to_string());
        Ok(Some(application.clone()))
    }
}

#[cfg(test)]
mod tests {
    use crate::db::Database;
    use crate::db::repositories::RepositoryError;

    use super::{ApplicationsService, ApplicationsServiceStub};

    #[tokio::test]
    async fn returns_disabled_error_without_database() {
        let service = ApplicationsService::new(
            crate::db::repositories::ApplicationsRepository::new(Database::disabled()),
        );

        let error = service
            .get_by_id("application-1")
            .await
            .expect_err("service should fail without configured database");

        assert!(matches!(error, RepositoryError::DatabaseDisabled));
    }

    #[tokio::test]
    async fn returns_none_for_unknown_application_in_stub() {
        let service = ApplicationsService::for_tests(ApplicationsServiceStub::default());

        let application = service
            .get_by_id("missing-application")
            .await
            .expect("stub should not fail");

        assert!(application.is_none());
    }
}
