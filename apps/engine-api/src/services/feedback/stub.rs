use std::collections::HashMap;
use std::sync::Mutex;

use crate::db::repositories::RepositoryError;
use crate::domain::feedback::model::{
    CompanyFeedbackRecord, CompanyFeedbackStatus, JobFeedbackFlags, JobFeedbackRecord,
};

#[derive(Default)]
pub struct FeedbackServiceStub {
    job_feedback_by_key: Mutex<HashMap<(String, String), JobFeedbackRecord>>,
    company_feedback_by_key: Mutex<HashMap<(String, String), CompanyFeedbackRecord>>,
    database_disabled: bool,
}

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

    pub(crate) fn upsert_job_feedback(
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

    pub(crate) fn list_job_feedback(
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

    pub(crate) fn list_job_feedback_for_jobs(
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

    pub(crate) fn upsert_company_feedback(
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

    pub(crate) fn remove_company_feedback(
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

    pub(crate) fn clear_job_feedback(
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

    pub(crate) fn list_company_feedback(
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

    pub(crate) fn list_company_feedback_for_names(
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
