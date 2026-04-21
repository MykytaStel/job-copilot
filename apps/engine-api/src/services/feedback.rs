#[cfg(test)]
#[path = "feedback/stub.rs"]
mod stub;

#[cfg(test)]
use std::sync::Arc;

use crate::db::repositories::{FeedbackRepository, RepositoryError};
use crate::domain::feedback::model::{
    CompanyFeedbackRecord, CompanyFeedbackStatus, JobFeedbackFlags, JobFeedbackRecord,
};

#[cfg(test)]
pub use stub::FeedbackServiceStub;

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
