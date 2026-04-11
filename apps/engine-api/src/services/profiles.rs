#[cfg(test)]
use std::collections::HashMap;
#[cfg(test)]
use std::sync::{Arc, Mutex};

use crate::db::repositories::{ProfilesRepository, RepositoryError};
use crate::domain::profile::model::{CreateProfile, Profile, ProfileAnalysis, UpdateProfile};

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

    #[cfg(test)]
    pub fn for_tests(stub: ProfilesServiceStub) -> Self {
        Self {
            backend: ProfilesServiceBackend::Stub(Arc::new(stub)),
        }
    }
}

#[cfg(test)]
#[derive(Default)]
pub struct ProfilesServiceStub {
    profiles_by_id: Mutex<HashMap<String, Profile>>,
    database_disabled: bool,
}

#[cfg(test)]
impl ProfilesServiceStub {
    fn create(&self, input: CreateProfile) -> Result<Profile, RepositoryError> {
        if self.database_disabled {
            return Err(RepositoryError::DatabaseDisabled);
        }

        let profile = Profile {
            id: "profile_test_001".to_string(),
            name: input.name,
            email: input.email,
            location: input.location,
            raw_text: input.raw_text,
            analysis: None,
            created_at: "2026-04-11T00:00:00+00:00".to_string(),
            updated_at: "2026-04-11T00:00:00+00:00".to_string(),
        };

        self.profiles_by_id
            .lock()
            .expect("profiles stub mutex should not be poisoned")
            .insert(profile.id.clone(), profile.clone());

        Ok(profile)
    }

    fn get_by_id(&self, id: &str) -> Result<Option<Profile>, RepositoryError> {
        if self.database_disabled {
            return Err(RepositoryError::DatabaseDisabled);
        }

        Ok(self
            .profiles_by_id
            .lock()
            .expect("profiles stub mutex should not be poisoned")
            .get(id)
            .cloned())
    }

    fn update(&self, id: &str, input: UpdateProfile) -> Result<Option<Profile>, RepositoryError> {
        if self.database_disabled {
            return Err(RepositoryError::DatabaseDisabled);
        }

        let mut profiles = self
            .profiles_by_id
            .lock()
            .expect("profiles stub mutex should not be poisoned");
        let Some(profile) = profiles.get_mut(id) else {
            return Ok(None);
        };

        if let Some(name) = input.name {
            profile.name = name;
        }
        if let Some(email) = input.email {
            profile.email = email;
        }
        if let Some(location) = input.location {
            profile.location = location;
        }
        if let Some(raw_text) = input.raw_text {
            profile.raw_text = raw_text;
            profile.analysis = None;
        }

        profile.updated_at = "2026-04-11T00:00:01+00:00".to_string();

        Ok(Some(profile.clone()))
    }

    fn save_analysis(
        &self,
        id: &str,
        analysis: ProfileAnalysis,
    ) -> Result<Option<Profile>, RepositoryError> {
        if self.database_disabled {
            return Err(RepositoryError::DatabaseDisabled);
        }

        let mut profiles = self
            .profiles_by_id
            .lock()
            .expect("profiles stub mutex should not be poisoned");
        let Some(profile) = profiles.get_mut(id) else {
            return Ok(None);
        };

        profile.analysis = Some(analysis);
        profile.updated_at = "2026-04-11T00:00:02+00:00".to_string();

        Ok(Some(profile.clone()))
    }
}

#[cfg(test)]
mod tests {
    use crate::db::Database;
    use crate::db::repositories::RepositoryError;
    use crate::domain::profile::model::CreateProfile;

    use super::{ProfilesService, ProfilesServiceStub};

    #[tokio::test]
    async fn returns_disabled_error_without_database() {
        let service = ProfilesService::new(crate::db::repositories::ProfilesRepository::new(
            Database::disabled(),
        ));

        let error = service
            .get_by_id("profile-1")
            .await
            .expect_err("service should fail without configured database");

        assert!(matches!(error, RepositoryError::DatabaseDisabled));
    }

    #[tokio::test]
    async fn creates_profile_in_stub() {
        let service = ProfilesService::for_tests(ProfilesServiceStub::default());

        let profile = service
            .create(CreateProfile {
                name: "Jane Doe".to_string(),
                email: "jane@example.com".to_string(),
                location: Some("Kyiv".to_string()),
                raw_text: "Senior frontend engineer".to_string(),
            })
            .await
            .expect("stub should create a profile");

        assert_eq!(profile.id, "profile_test_001");
        assert_eq!(profile.name, "Jane Doe");
    }
}
