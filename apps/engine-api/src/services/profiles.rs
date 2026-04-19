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

#[cfg(test)]
#[derive(Default)]
pub struct ProfilesServiceStub {
    profiles_by_id: Mutex<HashMap<String, Profile>>,
    database_disabled: bool,
}

#[cfg(test)]
impl ProfilesServiceStub {
    pub fn with_profile(self, profile: Profile) -> Self {
        self.profiles_by_id
            .lock()
            .expect("profiles stub mutex should not be poisoned")
            .insert(profile.id.clone(), profile);
        self
    }

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
            years_of_experience: input.years_of_experience,
            salary_min: input.salary_min,
            salary_max: input.salary_max,
            salary_currency: input.salary_currency,
            languages: input.languages,
            preferred_work_mode: None,
            created_at: "2026-04-11T00:00:00+00:00".to_string(),
            updated_at: "2026-04-11T00:00:00+00:00".to_string(),
            skills_updated_at: None,
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
            profile.skills_updated_at = None;
        }
        if let Some(years_of_experience) = input.years_of_experience {
            profile.years_of_experience = years_of_experience;
        }
        if let Some(salary_min) = input.salary_min {
            profile.salary_min = salary_min;
        }
        if let Some(salary_max) = input.salary_max {
            profile.salary_max = salary_max;
        }
        if let Some(salary_currency) = input.salary_currency {
            profile.salary_currency = salary_currency;
        }
        if let Some(languages) = input.languages {
            profile.languages = languages;
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
        profile.skills_updated_at = Some("2026-04-11T00:00:02+00:00".to_string());

        Ok(Some(profile.clone()))
    }

    fn get_latest(&self) -> Result<Option<Profile>, RepositoryError> {
        if self.database_disabled {
            return Err(RepositoryError::DatabaseDisabled);
        }

        Ok(self
            .profiles_by_id
            .lock()
            .expect("profiles stub mutex should not be poisoned")
            .values()
            .next()
            .cloned())
    }
}

#[cfg(test)]
mod tests {
    use crate::db::Database;
    use crate::db::repositories::RepositoryError;
    use crate::domain::profile::model::{CreateProfile, ProfileAnalysis};
    use crate::domain::role::RoleId;

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
                years_of_experience: None,
                salary_min: None,
                salary_max: None,
                salary_currency: "USD".to_string(),
                languages: vec![],
            })
            .await
            .expect("stub should create a profile");

        assert_eq!(profile.id, "profile_test_001");
        assert_eq!(profile.name, "Jane Doe");
    }

    #[tokio::test]
    async fn unrelated_profile_updates_keep_skills_timestamp() {
        let service = ProfilesService::for_tests(ProfilesServiceStub::default());

        let created = service
            .create(CreateProfile {
                name: "Jane Doe".to_string(),
                email: "jane@example.com".to_string(),
                location: Some("Kyiv".to_string()),
                raw_text: "Senior frontend engineer".to_string(),
                years_of_experience: None,
                salary_min: None,
                salary_max: None,
                salary_currency: "USD".to_string(),
                languages: vec![],
            })
            .await
            .expect("stub should create a profile");

        let analyzed = service
            .save_analysis(
                &created.id,
                ProfileAnalysis {
                    summary: "Experienced frontend engineer".to_string(),
                    primary_role: RoleId::FrontendEngineer,
                    seniority: "senior".to_string(),
                    skills: vec!["react".to_string()],
                    keywords: vec!["frontend".to_string()],
                },
            )
            .await
            .expect("analysis save should succeed")
            .expect("profile should exist");

        let updated = service
            .update(
                &created.id,
                crate::domain::profile::model::UpdateProfile {
                    name: Some("Jane Smith".to_string()),
                    ..Default::default()
                },
            )
            .await
            .expect("profile update should succeed")
            .expect("profile should exist");

        assert_eq!(updated.name, "Jane Smith");
        assert_eq!(updated.skills_updated_at, analyzed.skills_updated_at);
    }
}
