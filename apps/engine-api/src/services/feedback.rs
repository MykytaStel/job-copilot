#[cfg(test)]
use std::collections::HashMap;
#[cfg(test)]
use std::sync::{Arc, Mutex};

use crate::db::repositories::{FeedbackRepository, RepositoryError};
use crate::domain::feedback::model::{
    CompanyFeedbackRecord, CompanyFeedbackStatus, JobFeedbackFlags, JobFeedbackRecord,
};

#[derive(Clone)]
enum FeedbackServiceBackend {
    Repository(FeedbackRepository),
    #[cfg(test)]
    Stub(Arc<FeedbackServiceStub>),
}

#[derive(Clone)]
pub struct FeedbackService {
    backend: FeedbackServiceBackend,
}

impl FeedbackService {
    pub fn new(repository: FeedbackRepository) -> Self {
        Self {
            backend: FeedbackServiceBackend::Repository(repository),
        }
    }

    pub async fn upsert_job_feedback(
        &self,
        profile_id: &str,
        job_id: &str,
        flags: JobFeedbackFlags,
    ) -> Result<JobFeedbackRecord, RepositoryError> {
        match &self.backend {
            FeedbackServiceBackend::Repository(repository) => {
                repository
                    .upsert_job_feedback(profile_id, job_id, &flags)
                    .await
            }
            #[cfg(test)]
            FeedbackServiceBackend::Stub(stub) => {
                stub.upsert_job_feedback(profile_id, job_id, flags)
            }
        }
    }

    pub async fn list_job_feedback(
        &self,
        profile_id: &str,
    ) -> Result<Vec<JobFeedbackRecord>, RepositoryError> {
        match &self.backend {
            FeedbackServiceBackend::Repository(repository) => {
                repository.list_job_feedback(profile_id).await
            }
            #[cfg(test)]
            FeedbackServiceBackend::Stub(stub) => stub.list_job_feedback(profile_id),
        }
    }

    pub async fn list_job_feedback_for_jobs(
        &self,
        profile_id: &str,
        job_ids: &[String],
    ) -> Result<Vec<JobFeedbackRecord>, RepositoryError> {
        match &self.backend {
            FeedbackServiceBackend::Repository(repository) => {
                repository
                    .list_job_feedback_for_jobs(profile_id, job_ids)
                    .await
            }
            #[cfg(test)]
            FeedbackServiceBackend::Stub(stub) => {
                stub.list_job_feedback_for_jobs(profile_id, job_ids)
            }
        }
    }

    pub async fn upsert_company_feedback(
        &self,
        profile_id: &str,
        company_name: &str,
        normalized_company_name: &str,
        status: CompanyFeedbackStatus,
    ) -> Result<CompanyFeedbackRecord, RepositoryError> {
        match &self.backend {
            FeedbackServiceBackend::Repository(repository) => {
                repository
                    .upsert_company_feedback(
                        profile_id,
                        company_name,
                        normalized_company_name,
                        status,
                    )
                    .await
            }
            #[cfg(test)]
            FeedbackServiceBackend::Stub(stub) => stub.upsert_company_feedback(
                profile_id,
                company_name,
                normalized_company_name,
                status,
            ),
        }
    }

    pub async fn remove_company_feedback(
        &self,
        profile_id: &str,
        normalized_company_name: &str,
        status: CompanyFeedbackStatus,
    ) -> Result<bool, RepositoryError> {
        match &self.backend {
            FeedbackServiceBackend::Repository(repository) => {
                repository
                    .remove_company_feedback(profile_id, normalized_company_name, status)
                    .await
            }
            #[cfg(test)]
            FeedbackServiceBackend::Stub(stub) => {
                stub.remove_company_feedback(profile_id, normalized_company_name, status)
            }
        }
    }

    pub async fn list_company_feedback(
        &self,
        profile_id: &str,
    ) -> Result<Vec<CompanyFeedbackRecord>, RepositoryError> {
        match &self.backend {
            FeedbackServiceBackend::Repository(repository) => {
                repository.list_company_feedback(profile_id).await
            }
            #[cfg(test)]
            FeedbackServiceBackend::Stub(stub) => stub.list_company_feedback(profile_id),
        }
    }

    /// Clear specific feedback flags for a job. `flags` with `true` values are cleared.
    /// Returns the updated record, or `None` if no feedback row existed.
    pub async fn clear_job_feedback(
        &self,
        profile_id: &str,
        job_id: &str,
        flags: JobFeedbackFlags,
    ) -> Result<Option<JobFeedbackRecord>, RepositoryError> {
        match &self.backend {
            FeedbackServiceBackend::Repository(repository) => {
                repository
                    .clear_job_feedback(profile_id, job_id, &flags)
                    .await
            }
            #[cfg(test)]
            FeedbackServiceBackend::Stub(stub) => {
                stub.clear_job_feedback(profile_id, job_id, flags)
            }
        }
    }

    pub async fn list_company_feedback_for_names(
        &self,
        profile_id: &str,
        normalized_company_names: &[String],
    ) -> Result<Vec<CompanyFeedbackRecord>, RepositoryError> {
        match &self.backend {
            FeedbackServiceBackend::Repository(repository) => {
                repository
                    .list_company_feedback_for_names(profile_id, normalized_company_names)
                    .await
            }
            #[cfg(test)]
            FeedbackServiceBackend::Stub(stub) => {
                stub.list_company_feedback_for_names(profile_id, normalized_company_names)
            }
        }
    }

    pub fn normalize_company_name(company_name: &str) -> String {
        company_name
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
            .trim()
            .to_lowercase()
    }

    #[cfg(test)]
    pub fn for_tests(stub: FeedbackServiceStub) -> Self {
        Self {
            backend: FeedbackServiceBackend::Stub(Arc::new(stub)),
        }
    }
}

#[cfg(test)]
#[derive(Default)]
pub struct FeedbackServiceStub {
    job_feedback_by_key: Mutex<HashMap<(String, String), JobFeedbackRecord>>,
    company_feedback_by_key: Mutex<HashMap<(String, String), CompanyFeedbackRecord>>,
    database_disabled: bool,
}

#[cfg(test)]
impl FeedbackServiceStub {
    pub fn with_job_feedback(self, record: JobFeedbackRecord) -> Self {
        self.job_feedback_by_key
            .lock()
            .expect("feedback job stub mutex should not be poisoned")
            .insert((record.profile_id.clone(), record.job_id.clone()), record);
        self
    }

    pub fn with_company_feedback(self, record: CompanyFeedbackRecord) -> Self {
        self.company_feedback_by_key
            .lock()
            .expect("feedback company stub mutex should not be poisoned")
            .insert(
                (
                    record.profile_id.clone(),
                    record.normalized_company_name.clone(),
                ),
                record,
            );
        self
    }

    fn upsert_job_feedback(
        &self,
        profile_id: &str,
        job_id: &str,
        flags: JobFeedbackFlags,
    ) -> Result<JobFeedbackRecord, RepositoryError> {
        if self.database_disabled {
            return Err(RepositoryError::DatabaseDisabled);
        }

        let key = (profile_id.to_string(), job_id.to_string());
        let mut records = self
            .job_feedback_by_key
            .lock()
            .expect("feedback job stub mutex should not be poisoned");
        let entry = records.entry(key).or_insert_with(|| JobFeedbackRecord {
            profile_id: profile_id.to_string(),
            job_id: job_id.to_string(),
            saved: false,
            hidden: false,
            bad_fit: false,
            created_at: "2026-04-14T00:00:00+00:00".to_string(),
            updated_at: "2026-04-14T00:00:00+00:00".to_string(),
        });

        entry.saved |= flags.saved;
        entry.hidden |= flags.hidden;
        entry.bad_fit |= flags.bad_fit;
        entry.updated_at = "2026-04-14T00:00:01+00:00".to_string();

        Ok(entry.clone())
    }

    fn list_job_feedback(
        &self,
        profile_id: &str,
    ) -> Result<Vec<JobFeedbackRecord>, RepositoryError> {
        if self.database_disabled {
            return Err(RepositoryError::DatabaseDisabled);
        }

        Ok(self
            .job_feedback_by_key
            .lock()
            .expect("feedback job stub mutex should not be poisoned")
            .values()
            .filter(|record| record.profile_id == profile_id)
            .cloned()
            .collect())
    }

    fn list_job_feedback_for_jobs(
        &self,
        profile_id: &str,
        job_ids: &[String],
    ) -> Result<Vec<JobFeedbackRecord>, RepositoryError> {
        if self.database_disabled {
            return Err(RepositoryError::DatabaseDisabled);
        }

        Ok(self
            .job_feedback_by_key
            .lock()
            .expect("feedback job stub mutex should not be poisoned")
            .values()
            .filter(|record| {
                record.profile_id == profile_id
                    && job_ids.iter().any(|job_id| job_id == &record.job_id)
            })
            .cloned()
            .collect())
    }

    fn upsert_company_feedback(
        &self,
        profile_id: &str,
        company_name: &str,
        normalized_company_name: &str,
        status: CompanyFeedbackStatus,
    ) -> Result<CompanyFeedbackRecord, RepositoryError> {
        if self.database_disabled {
            return Err(RepositoryError::DatabaseDisabled);
        }

        let record = CompanyFeedbackRecord {
            profile_id: profile_id.to_string(),
            company_name: company_name.to_string(),
            normalized_company_name: normalized_company_name.to_string(),
            status,
            created_at: "2026-04-14T00:00:00+00:00".to_string(),
            updated_at: "2026-04-14T00:00:01+00:00".to_string(),
        };

        self.company_feedback_by_key
            .lock()
            .expect("feedback company stub mutex should not be poisoned")
            .insert(
                (profile_id.to_string(), normalized_company_name.to_string()),
                record.clone(),
            );

        Ok(record)
    }

    fn remove_company_feedback(
        &self,
        profile_id: &str,
        normalized_company_name: &str,
        status: CompanyFeedbackStatus,
    ) -> Result<bool, RepositoryError> {
        if self.database_disabled {
            return Err(RepositoryError::DatabaseDisabled);
        }

        let mut records = self
            .company_feedback_by_key
            .lock()
            .expect("feedback company stub mutex should not be poisoned");
        let key = (profile_id.to_string(), normalized_company_name.to_string());
        let should_remove = records
            .get(&key)
            .is_some_and(|record| record.status == status);

        if should_remove {
            records.remove(&key);
        }

        Ok(should_remove)
    }

    fn clear_job_feedback(
        &self,
        profile_id: &str,
        job_id: &str,
        flags: JobFeedbackFlags,
    ) -> Result<Option<JobFeedbackRecord>, RepositoryError> {
        if self.database_disabled {
            return Err(RepositoryError::DatabaseDisabled);
        }

        let key = (profile_id.to_string(), job_id.to_string());
        let mut records = self
            .job_feedback_by_key
            .lock()
            .expect("feedback job stub mutex should not be poisoned");

        let Some(entry) = records.get_mut(&key) else {
            return Ok(None);
        };

        if flags.saved {
            entry.saved = false;
        }
        if flags.hidden {
            entry.hidden = false;
        }
        if flags.bad_fit {
            entry.bad_fit = false;
        }
        entry.updated_at = "2026-04-14T00:00:01+00:00".to_string();

        Ok(Some(entry.clone()))
    }

    fn list_company_feedback(
        &self,
        profile_id: &str,
    ) -> Result<Vec<CompanyFeedbackRecord>, RepositoryError> {
        if self.database_disabled {
            return Err(RepositoryError::DatabaseDisabled);
        }

        Ok(self
            .company_feedback_by_key
            .lock()
            .expect("feedback company stub mutex should not be poisoned")
            .values()
            .filter(|record| record.profile_id == profile_id)
            .cloned()
            .collect())
    }

    fn list_company_feedback_for_names(
        &self,
        profile_id: &str,
        normalized_company_names: &[String],
    ) -> Result<Vec<CompanyFeedbackRecord>, RepositoryError> {
        if self.database_disabled {
            return Err(RepositoryError::DatabaseDisabled);
        }

        Ok(self
            .company_feedback_by_key
            .lock()
            .expect("feedback company stub mutex should not be poisoned")
            .values()
            .filter(|record| {
                record.profile_id == profile_id
                    && normalized_company_names
                        .iter()
                        .any(|name| name == &record.normalized_company_name)
            })
            .cloned()
            .collect())
    }
}
