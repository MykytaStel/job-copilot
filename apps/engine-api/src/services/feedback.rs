#[cfg(test)]
#[path = "feedback/stub.rs"]
mod stub;

#[cfg(test)]
use std::sync::Arc;

use crate::db::repositories::{FeedbackRepository, RepositoryError};
use crate::domain::feedback::model::{
    CompanyFeedbackRecord, CompanyFeedbackStatus, JobFeedbackFlags, JobFeedbackReason,
    JobFeedbackRecord, JobFeedbackTagRecord, LegitimacySignal, SalaryFeedbackSignal,
    WorkModeFeedbackSignal,
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

    pub async fn update_company_feedback_notes(
        &self,
        profile_id: &str,
        normalized_company_name: &str,
        notes: &str,
    ) -> Result<Option<CompanyFeedbackRecord>, RepositoryError> {
        match &self.backend {
            FeedbackServiceBackend::Repository(repository) => {
                repository
                    .update_company_feedback_notes(profile_id, normalized_company_name, notes)
                    .await
            }
            #[cfg(test)]
            FeedbackServiceBackend::Stub(stub) => {
                stub.update_company_feedback_notes(profile_id, normalized_company_name, notes)
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

    pub async fn set_salary_signal(
        &self,
        profile_id: &str,
        job_id: &str,
        signal: SalaryFeedbackSignal,
    ) -> Result<JobFeedbackRecord, RepositoryError> {
        match &self.backend {
            FeedbackServiceBackend::Repository(repository) => {
                repository
                    .set_salary_signal(profile_id, job_id, signal)
                    .await
            }
            #[cfg(test)]
            FeedbackServiceBackend::Stub(_) => Err(RepositoryError::DatabaseDisabled),
        }
    }

    pub async fn set_interest_rating(
        &self,
        profile_id: &str,
        job_id: &str,
        rating: i8,
    ) -> Result<JobFeedbackRecord, RepositoryError> {
        match &self.backend {
            FeedbackServiceBackend::Repository(repository) => {
                repository
                    .set_interest_rating(profile_id, job_id, rating)
                    .await
            }
            #[cfg(test)]
            FeedbackServiceBackend::Stub(_) => Err(RepositoryError::DatabaseDisabled),
        }
    }

    pub async fn set_work_mode_signal(
        &self,
        profile_id: &str,
        job_id: &str,
        signal: WorkModeFeedbackSignal,
    ) -> Result<JobFeedbackRecord, RepositoryError> {
        match &self.backend {
            FeedbackServiceBackend::Repository(repository) => {
                repository
                    .set_work_mode_signal(profile_id, job_id, signal)
                    .await
            }
            #[cfg(test)]
            FeedbackServiceBackend::Stub(_) => Err(RepositoryError::DatabaseDisabled),
        }
    }

    pub async fn set_legitimacy_signal(
        &self,
        profile_id: &str,
        job_id: &str,
        signal: LegitimacySignal,
    ) -> Result<JobFeedbackRecord, RepositoryError> {
        let also_bad_fit = matches!(
            signal,
            LegitimacySignal::Spam | LegitimacySignal::Suspicious
        );
        match &self.backend {
            FeedbackServiceBackend::Repository(repository) => {
                repository
                    .set_legitimacy_signal(profile_id, job_id, signal, also_bad_fit)
                    .await
            }
            #[cfg(test)]
            FeedbackServiceBackend::Stub(_) => Err(RepositoryError::DatabaseDisabled),
        }
    }

    pub async fn upsert_job_feedback_tags(
        &self,
        profile_id: &str,
        job_id: &str,
        tags: Vec<JobFeedbackReason>,
    ) -> Result<Vec<JobFeedbackTagRecord>, RepositoryError> {
        match &self.backend {
            FeedbackServiceBackend::Repository(repository) => {
                repository
                    .upsert_job_feedback_tags(profile_id, job_id, &tags)
                    .await
            }
            #[cfg(test)]
            FeedbackServiceBackend::Stub(_) => Err(RepositoryError::DatabaseDisabled),
        }
    }

    pub async fn list_feedback_tags_for_jobs(
        &self,
        profile_id: &str,
        job_ids: &[String],
    ) -> Result<Vec<JobFeedbackTagRecord>, RepositoryError> {
        match &self.backend {
            FeedbackServiceBackend::Repository(repository) => {
                repository
                    .list_feedback_tags_for_jobs(profile_id, job_ids)
                    .await
            }
            #[cfg(test)]
            FeedbackServiceBackend::Stub(_) => Ok(Vec::new()),
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

    pub async fn clear_all_hidden_jobs(&self, profile_id: &str) -> Result<u64, RepositoryError> {
        match &self.backend {
            FeedbackServiceBackend::Repository(repository) => {
                repository.clear_all_hidden_jobs(profile_id).await
            }
            #[cfg(test)]
            FeedbackServiceBackend::Stub(stub) => stub.clear_all_hidden_jobs(profile_id),
        }
    }

    pub async fn bulk_hide_jobs_by_company(
        &self,
        profile_id: &str,
        normalized_company_name: &str,
    ) -> Result<u64, RepositoryError> {
        match &self.backend {
            FeedbackServiceBackend::Repository(repository) => {
                repository
                    .bulk_hide_jobs_by_company(profile_id, normalized_company_name)
                    .await
            }
            #[cfg(test)]
            FeedbackServiceBackend::Stub(stub) => {
                stub.bulk_hide_jobs_by_company(profile_id, normalized_company_name)
            }
        }
    }

    pub async fn delete_all_for_profile(&self, profile_id: &str) -> Result<u64, RepositoryError> {
        match &self.backend {
            FeedbackServiceBackend::Repository(repository) => {
                repository.delete_all_for_profile(profile_id).await
            }
            #[cfg(test)]
            FeedbackServiceBackend::Stub(stub) => stub.delete_all_for_profile(profile_id),
        }
    }

    #[cfg(test)]
    pub fn for_tests(stub: FeedbackServiceStub) -> Self {
        Self {
            backend: FeedbackServiceBackend::Stub(Arc::new(stub)),
        }
    }
}
