#[cfg(test)]
#[path = "profiles/stub.rs"]
mod stub;
#[cfg(test)]
#[path = "profiles/tests.rs"]
mod tests;

#[cfg(test)]
use std::sync::Arc;

use crate::db::repositories::{ProfilesRepository, RepositoryError};
use crate::domain::profile::model::{CreateProfile, Profile, ProfileAnalysis, UpdateProfile};

#[cfg(test)]
pub use stub::ProfilesServiceStub;

#[derive(Clone)]
enum ProfilesServiceBackend {
    Repository(ProfilesRepository),
    #[cfg(test)]
    Stub(Arc<ProfilesServiceStub>),
}

#[derive(Clone)]
pub struct ProfilesService {
    backend: ProfilesServiceBackend,
}

impl ProfilesService {
    pub fn new(repository: ProfilesRepository) -> Self {
        Self {
            backend: ProfilesServiceBackend::Repository(repository),
        }
    }

    pub async fn create(&self, input: CreateProfile) -> Result<Profile, RepositoryError> {
        match &self.backend {
            ProfilesServiceBackend::Repository(repository) => repository.create(&input).await,
            #[cfg(test)]
            ProfilesServiceBackend::Stub(stub) => stub.create(input),
        }
    }

    pub async fn get_by_id(&self, id: &str) -> Result<Option<Profile>, RepositoryError> {
        match &self.backend {
            ProfilesServiceBackend::Repository(repository) => repository.get_by_id(id).await,
            #[cfg(test)]
            ProfilesServiceBackend::Stub(stub) => stub.get_by_id(id),
        }
    }

    pub async fn update(
        &self,
        id: &str,
        input: UpdateProfile,
    ) -> Result<Option<Profile>, RepositoryError> {
        match &self.backend {
            ProfilesServiceBackend::Repository(repository) => repository.update(id, &input).await,
            #[cfg(test)]
            ProfilesServiceBackend::Stub(stub) => stub.update(id, input),
        }
    }

    pub async fn save_analysis(
        &self,
        id: &str,
        analysis: ProfileAnalysis,
    ) -> Result<Option<Profile>, RepositoryError> {
        match &self.backend {
            ProfilesServiceBackend::Repository(repository) => {
                repository.save_analysis(id, &analysis).await
            }
            #[cfg(test)]
            ProfilesServiceBackend::Stub(stub) => stub.save_analysis(id, analysis),
        }
    }

    pub async fn get_latest(&self) -> Result<Option<Profile>, RepositoryError> {
        match &self.backend {
            ProfilesServiceBackend::Repository(repository) => repository.get_latest().await,
            #[cfg(test)]
            ProfilesServiceBackend::Stub(stub) => stub.get_latest(),
        }
    }

    #[cfg(test)]
    pub fn for_tests(stub: ProfilesServiceStub) -> Self {
        Self {
            backend: ProfilesServiceBackend::Stub(Arc::new(stub)),
        }
    }
}
