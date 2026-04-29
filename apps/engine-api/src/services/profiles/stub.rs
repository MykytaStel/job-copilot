use std::collections::HashMap;
use std::sync::Mutex;

use crate::db::repositories::RepositoryError;
use crate::domain::profile::model::{CreateProfile, Profile, ProfileAnalysis, UpdateProfile};

#[derive(Default)]
pub struct ProfilesServiceStub {
    profiles_by_id: Mutex<HashMap<String, Profile>>,
    database_disabled: bool,
}

impl ProfilesServiceStub {
    pub fn with_profile(self, profile: Profile) -> Self {
        self.profiles_by_id
            .lock()
            .expect("profiles stub mutex should not be poisoned")
            .insert(profile.id.clone(), profile);
        self
    }

    pub(crate) fn create(&self, input: CreateProfile) -> Result<Profile, RepositoryError> {
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
            preferred_locations: input.preferred_locations,
            experience: input.experience,
            work_mode_preference: input.work_mode_preference,
            preferred_language: None,
            search_preferences: input.search_preferences,
            created_at: "2026-04-11T00:00:00+00:00".to_string(),
            updated_at: "2026-04-11T00:00:00+00:00".to_string(),
            skills_updated_at: None,
            portfolio_url: None,
            github_url: None,
            linkedin_url: None,
        };

        self.profiles_by_id
            .lock()
            .expect("profiles stub mutex should not be poisoned")
            .insert(profile.id.clone(), profile.clone());

        Ok(profile)
    }

    pub(crate) fn get_by_id(&self, id: &str) -> Result<Option<Profile>, RepositoryError> {
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

    pub(crate) fn update(
        &self,
        id: &str,
        input: UpdateProfile,
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
        if let Some(preferred_locations) = input.preferred_locations {
            profile.preferred_locations = preferred_locations;
        }
        if let Some(experience) = input.experience {
            profile.experience = experience;
        }
        if let Some(work_mode_preference) = input.work_mode_preference {
            profile.work_mode_preference = work_mode_preference;
        }
        if let Some(skills) = input.skills
            && let Some(analysis) = profile.analysis.as_mut()
        {
            analysis.skills = skills;
            profile.skills_updated_at = Some("2026-04-11T00:00:02+00:00".to_string());
        }
        if let Some(search_preferences) = input.search_preferences {
            profile.search_preferences = search_preferences;
        }

        profile.updated_at = "2026-04-11T00:00:01+00:00".to_string();

        Ok(Some(profile.clone()))
    }

    pub(crate) fn save_analysis(
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

    pub(crate) fn get_by_email(&self, email: &str) -> Result<Option<Profile>, RepositoryError> {
        if self.database_disabled {
            return Err(RepositoryError::DatabaseDisabled);
        }

        Ok(self
            .profiles_by_id
            .lock()
            .expect("profiles stub mutex should not be poisoned")
            .values()
            .find(|p| p.email == email)
            .cloned())
    }

    pub(crate) fn get_latest(&self) -> Result<Option<Profile>, RepositoryError> {
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
