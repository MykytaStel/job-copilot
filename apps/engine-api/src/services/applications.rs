#[cfg(test)]
#[path = "applications/stub.rs"]
mod stub;
#[cfg(test)]
#[path = "applications/tests.rs"]
mod tests;

#[cfg(test)]
use std::sync::Arc;

use crate::db::repositories::{ApplicationsRepository, RepositoryError};
use crate::domain::application::model::{
    Application, ApplicationContact, ApplicationDetail, ApplicationNote, Contact,
    CreateApplication, CreateApplicationContact, CreateContact, CreateNote, Offer,
    UpdateApplication, UpsertOffer,
};
use crate::domain::search::global::ApplicationSearchHit;

#[cfg(test)]
pub use stub::ApplicationsServiceStub;

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

    pub async fn list_recent(
        &self,
        limit: i64,
        profile_id: Option<&str>,
    ) -> Result<Vec<Application>, RepositoryError> {
        match &self.backend {
            ApplicationsServiceBackend::Repository(repository) => {
                repository.list_recent(limit, profile_id).await
            }
            #[cfg(test)]
            ApplicationsServiceBackend::Stub(stub) => stub.list_recent(limit, profile_id),
        }
    }

    pub async fn search_by_job_title(
        &self,
        query: &str,
        limit: i64,
    ) -> Result<Vec<ApplicationSearchHit>, RepositoryError> {
        match &self.backend {
            ApplicationsServiceBackend::Repository(repository) => {
                repository.search_by_job_title(query, limit).await
            }
            #[cfg(test)]
            ApplicationsServiceBackend::Stub(stub) => stub.search_by_job_title(query, limit),
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

    pub async fn create_note(&self, note: CreateNote) -> Result<ApplicationNote, RepositoryError> {
        match &self.backend {
            ApplicationsServiceBackend::Repository(repository) => {
                repository.create_note(&note).await
            }
            #[cfg(test)]
            ApplicationsServiceBackend::Stub(stub) => stub.create_note(note),
        }
    }

    pub async fn create_contact(&self, contact: CreateContact) -> Result<Contact, RepositoryError> {
        match &self.backend {
            ApplicationsServiceBackend::Repository(repository) => {
                repository.create_contact(&contact).await
            }
            #[cfg(test)]
            ApplicationsServiceBackend::Stub(stub) => stub.create_contact(contact),
        }
    }

    pub async fn list_contacts(&self, offset: i64) -> Result<(Vec<Contact>, i64), RepositoryError> {
        match &self.backend {
            ApplicationsServiceBackend::Repository(repository) => {
                repository.list_contacts(offset).await
            }
            #[cfg(test)]
            ApplicationsServiceBackend::Stub(stub) => stub.list_contacts(offset),
        }
    }

    pub async fn get_contact_by_id(&self, id: &str) -> Result<Option<Contact>, RepositoryError> {
        match &self.backend {
            ApplicationsServiceBackend::Repository(repository) => {
                repository.get_contact_by_id(id).await
            }
            #[cfg(test)]
            ApplicationsServiceBackend::Stub(stub) => stub.get_contact_by_id(id),
        }
    }

    pub async fn attach_contact(
        &self,
        contact: CreateApplicationContact,
    ) -> Result<ApplicationContact, RepositoryError> {
        match &self.backend {
            ApplicationsServiceBackend::Repository(repository) => {
                repository.attach_contact(&contact).await
            }
            #[cfg(test)]
            ApplicationsServiceBackend::Stub(stub) => stub.attach_contact(contact),
        }
    }

    pub async fn upsert_offer(&self, offer: UpsertOffer) -> Result<Offer, RepositoryError> {
        match &self.backend {
            ApplicationsServiceBackend::Repository(repository) => {
                repository.upsert_offer(&offer).await
            }
            #[cfg(test)]
            ApplicationsServiceBackend::Stub(stub) => stub.upsert_offer(offer),
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
