#[cfg(test)]
#[path = "profile_ml_state/stub.rs"]
mod stub;

#[cfg(test)]
use std::sync::Arc;

use crate::db::repositories::{ProfileMlStateRepository, RepositoryError};
use crate::domain::profile::ml::{ProfileMlRetrainCandidate, ProfileMlState, UpdateProfileMlState};

#[cfg(test)]
pub use stub::ProfileMlStateServiceStub;

#[derive(Clone)]
enum ProfileMlStateServiceBackend {
    Repository(ProfileMlStateRepository),
    #[cfg(test)]
    Stub(Arc<ProfileMlStateServiceStub>),
}

#[derive(Clone)]
pub struct ProfileMlStateService {
    backend: ProfileMlStateServiceBackend,
}

impl ProfileMlStateService {
    pub fn new(repository: ProfileMlStateRepository) -> Self {
        Self {
            backend: ProfileMlStateServiceBackend::Repository(repository),
        }
    }

    pub async fn get_by_profile_id(
        &self,
        profile_id: &str,
    ) -> Result<Option<ProfileMlState>, RepositoryError> {
        match &self.backend {
            ProfileMlStateServiceBackend::Repository(repository) => {
                repository.get_by_profile_id(profile_id).await
            }
            #[cfg(test)]
            ProfileMlStateServiceBackend::Stub(stub) => stub.get_by_profile_id(profile_id),
        }
    }

    pub async fn record_labelable_job(
        &self,
        profile_id: &str,
        job_id: &str,
    ) -> Result<bool, RepositoryError> {
        match &self.backend {
            ProfileMlStateServiceBackend::Repository(repository) => {
                repository.record_labelable_job(profile_id, job_id).await
            }
            #[cfg(test)]
            ProfileMlStateServiceBackend::Stub(stub) => {
                stub.record_labelable_job(profile_id, job_id)
            }
        }
    }

    pub async fn list_ready_for_retrain(
        &self,
        min_examples: usize,
    ) -> Result<Vec<ProfileMlRetrainCandidate>, RepositoryError> {
        match &self.backend {
            ProfileMlStateServiceBackend::Repository(repository) => {
                repository.list_ready_for_retrain(min_examples).await
            }
            #[cfg(test)]
            ProfileMlStateServiceBackend::Stub(stub) => stub.list_ready_for_retrain(min_examples),
        }
    }

    pub async fn update_state(
        &self,
        profile_id: &str,
        update: UpdateProfileMlState,
    ) -> Result<Option<ProfileMlState>, RepositoryError> {
        match &self.backend {
            ProfileMlStateServiceBackend::Repository(repository) => {
                repository.update_state(profile_id, &update).await
            }
            #[cfg(test)]
            ProfileMlStateServiceBackend::Stub(stub) => stub.update_state(profile_id, update),
        }
    }

    #[cfg(test)]
    pub fn for_tests(stub: ProfileMlStateServiceStub) -> Self {
        Self {
            backend: ProfileMlStateServiceBackend::Stub(Arc::new(stub)),
        }
    }
}
