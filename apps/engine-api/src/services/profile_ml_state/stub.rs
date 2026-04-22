use std::collections::{HashMap, HashSet};
use std::sync::Mutex;

use crate::db::repositories::RepositoryError;
use crate::domain::profile::ml::{ProfileMlRetrainCandidate, ProfileMlState, UpdateProfileMlState};

#[derive(Default)]
pub struct ProfileMlStateServiceStub {
    states_by_profile_id: Mutex<HashMap<String, ProfileMlState>>,
    labeled_jobs_by_profile_id: Mutex<HashMap<String, HashSet<String>>>,
    database_disabled: bool,
}

impl ProfileMlStateServiceStub {
    pub fn with_state(self, state: ProfileMlState) -> Self {
        self.states_by_profile_id
            .lock()
            .expect("profile ml state stub mutex should not be poisoned")
            .insert(state.profile_id.clone(), state);
        self
    }

    #[allow(dead_code)]
    pub fn with_database_disabled(mut self) -> Self {
        self.database_disabled = true;
        self
    }

    pub(crate) fn get_by_profile_id(
        &self,
        profile_id: &str,
    ) -> Result<Option<ProfileMlState>, RepositoryError> {
        if self.database_disabled {
            return Err(RepositoryError::DatabaseDisabled);
        }

        Ok(self
            .states_by_profile_id
            .lock()
            .expect("profile ml state stub mutex should not be poisoned")
            .get(profile_id)
            .cloned())
    }

    pub(crate) fn record_labelable_job(
        &self,
        profile_id: &str,
        job_id: &str,
    ) -> Result<bool, RepositoryError> {
        if self.database_disabled {
            return Err(RepositoryError::DatabaseDisabled);
        }

        let mut labeled_jobs_by_profile_id = self
            .labeled_jobs_by_profile_id
            .lock()
            .expect("profile ml state stub mutex should not be poisoned");
        let jobs = labeled_jobs_by_profile_id
            .entry(profile_id.to_string())
            .or_default();
        let inserted = jobs.insert(job_id.to_string());

        if inserted {
            let mut states = self
                .states_by_profile_id
                .lock()
                .expect("profile ml state stub mutex should not be poisoned");
            let state = states
                .entry(profile_id.to_string())
                .or_insert_with(|| ProfileMlState {
                    profile_id: profile_id.to_string(),
                    ..ProfileMlState::default()
                });
            state.examples_since_retrain += 1;
        }

        Ok(inserted)
    }

    pub(crate) fn list_ready_for_retrain(
        &self,
        min_examples: usize,
    ) -> Result<Vec<ProfileMlRetrainCandidate>, RepositoryError> {
        if self.database_disabled {
            return Err(RepositoryError::DatabaseDisabled);
        }

        Ok(self
            .states_by_profile_id
            .lock()
            .expect("profile ml state stub mutex should not be poisoned")
            .values()
            .filter(|state| state.examples_since_retrain >= min_examples)
            .map(|state| ProfileMlRetrainCandidate {
                profile_id: state.profile_id.clone(),
                examples_since_retrain: state.examples_since_retrain,
            })
            .collect())
    }

    pub(crate) fn update_state(
        &self,
        profile_id: &str,
        update: UpdateProfileMlState,
    ) -> Result<Option<ProfileMlState>, RepositoryError> {
        if self.database_disabled {
            return Err(RepositoryError::DatabaseDisabled);
        }

        let mut states = self
            .states_by_profile_id
            .lock()
            .expect("profile ml state stub mutex should not be poisoned");
        let state = states
            .entry(profile_id.to_string())
            .or_insert_with(|| ProfileMlState {
                profile_id: profile_id.to_string(),
                ..ProfileMlState::default()
            });

        if let Some(last_retrained_at) = update.last_retrained_at {
            state.last_retrained_at = last_retrained_at;
        }
        if let Some(examples_since_retrain) = update.examples_since_retrain {
            state.examples_since_retrain = examples_since_retrain;
        }
        if let Some(last_artifact_version) = update.last_artifact_version {
            state.last_artifact_version = last_artifact_version;
        }
        if let Some(last_training_status) = update.last_training_status {
            state.last_training_status = last_training_status;
        }

        Ok(Some(state.clone()))
    }
}
