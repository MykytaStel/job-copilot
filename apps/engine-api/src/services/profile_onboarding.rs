#[cfg(test)]
use std::collections::HashMap;
#[cfg(test)]
use std::sync::{Arc, Mutex};

use crate::db::repositories::{ProfileOnboardingRepository, RepositoryError};
use crate::domain::profile::onboarding::{OnboardingOutcome, OnboardingStep, ProfileOnboarding};

#[derive(Clone)]
enum ProfileOnboardingBackend {
    Repository(ProfileOnboardingRepository),
    #[cfg(test)]
    Stub(Arc<ProfileOnboardingServiceStub>),
}

#[derive(Clone)]
pub struct ProfileOnboardingService {
    backend: ProfileOnboardingBackend,
}

impl ProfileOnboardingService {
    pub fn new(repository: ProfileOnboardingRepository) -> Self {
        Self {
            backend: ProfileOnboardingBackend::Repository(repository),
        }
    }

    pub async fn get_or_create(
        &self,
        profile_id: &str,
    ) -> Result<ProfileOnboarding, RepositoryError> {
        match &self.backend {
            ProfileOnboardingBackend::Repository(repository) => {
                repository.get_or_create(profile_id).await
            }
            #[cfg(test)]
            ProfileOnboardingBackend::Stub(stub) => stub.get_or_create(profile_id),
        }
    }

    pub async fn advance(
        &self,
        profile_id: &str,
        step: OnboardingStep,
        outcome: OnboardingOutcome,
    ) -> Result<ProfileOnboarding, RepositoryError> {
        match &self.backend {
            ProfileOnboardingBackend::Repository(repository) => {
                repository.advance(profile_id, step, outcome).await
            }
            #[cfg(test)]
            ProfileOnboardingBackend::Stub(stub) => stub.advance(profile_id, step, outcome),
        }
    }

    #[cfg(test)]
    pub fn for_tests(stub: ProfileOnboardingServiceStub) -> Self {
        Self {
            backend: ProfileOnboardingBackend::Stub(Arc::new(stub)),
        }
    }
}

#[cfg(test)]
#[derive(Default)]
pub struct ProfileOnboardingServiceStub {
    states: Mutex<HashMap<String, ProfileOnboarding>>,
}

#[cfg(test)]
impl ProfileOnboardingServiceStub {
    fn get_or_create(&self, profile_id: &str) -> Result<ProfileOnboarding, RepositoryError> {
        let mut states = self
            .states
            .lock()
            .expect("onboarding stub mutex should not be poisoned");
        Ok(states
            .entry(profile_id.to_string())
            .or_insert_with(|| ProfileOnboarding::new(profile_id))
            .clone())
    }

    fn advance(
        &self,
        profile_id: &str,
        step: OnboardingStep,
        outcome: OnboardingOutcome,
    ) -> Result<ProfileOnboarding, RepositoryError> {
        let mut states = self
            .states
            .lock()
            .expect("onboarding stub mutex should not be poisoned");
        let state = states
            .entry(profile_id.to_string())
            .or_insert_with(|| ProfileOnboarding::new(profile_id));
        state
            .advance(step, outcome)
            .map_err(|message| RepositoryError::Conflict { message })?;
        Ok(state.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::{ProfileOnboardingService, ProfileOnboardingServiceStub};
    use crate::domain::profile::onboarding::{OnboardingOutcome, OnboardingStep};

    #[tokio::test]
    async fn stub_persists_progress_between_reads() {
        let service = ProfileOnboardingService::for_tests(ProfileOnboardingServiceStub::default());

        service
            .advance(
                "profile-1",
                OnboardingStep::Cv,
                OnboardingOutcome::Completed,
            )
            .await
            .expect("step should advance");
        let state = service
            .get_or_create("profile-1")
            .await
            .expect("state should load");

        assert_eq!(state.current_step, OnboardingStep::Profile);
    }
}
