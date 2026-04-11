#[cfg(test)]
use std::collections::HashMap;
#[cfg(test)]
use std::sync::Arc;

use crate::db::repositories::{JobsRepository, RepositoryError};
use crate::domain::job::model::Job;

#[derive(Clone)]
enum JobsServiceBackend {
    Repository(JobsRepository),
    #[cfg(test)]
    Stub(Arc<JobsServiceStub>),
}

#[derive(Clone)]
pub struct JobsService {
    backend: JobsServiceBackend,
}

impl JobsService {
    pub fn new(repository: JobsRepository) -> Self {
        Self {
            backend: JobsServiceBackend::Repository(repository),
        }
    }

    pub async fn get_by_id(&self, id: &str) -> Result<Option<Job>, RepositoryError> {
        match &self.backend {
            JobsServiceBackend::Repository(repository) => repository.get_by_id(id).await,
            #[cfg(test)]
            JobsServiceBackend::Stub(stub) => stub.get_by_id(id),
        }
    }

    pub async fn list_recent(&self, limit: i64) -> Result<Vec<Job>, RepositoryError> {
        match &self.backend {
            JobsServiceBackend::Repository(repository) => repository.list_recent(limit).await,
            #[cfg(test)]
            JobsServiceBackend::Stub(stub) => stub.list_recent(limit),
        }
    }

    pub async fn search(&self, query: &str, limit: i64) -> Result<Vec<Job>, RepositoryError> {
        match &self.backend {
            JobsServiceBackend::Repository(repository) => repository.search(query, limit).await,
            #[cfg(test)]
            JobsServiceBackend::Stub(stub) => stub.search(query, limit),
        }
    }

    #[cfg(test)]
    pub fn for_tests(stub: JobsServiceStub) -> Self {
        Self {
            backend: JobsServiceBackend::Stub(Arc::new(stub)),
        }
    }
}

#[cfg(test)]
#[derive(Default)]
pub struct JobsServiceStub {
    jobs_by_id: HashMap<String, Job>,
    recent_jobs: Vec<Job>,
    search_jobs: Vec<Job>,
    database_disabled: bool,
}

#[cfg(test)]
impl JobsServiceStub {
    pub fn with_job(mut self, job: Job) -> Self {
        self.jobs_by_id.insert(job.id.clone(), job.clone());
        self.recent_jobs.push(job.clone());
        self.search_jobs.push(job);
        self
    }

    fn get_by_id(&self, id: &str) -> Result<Option<Job>, RepositoryError> {
        if self.database_disabled {
            return Err(RepositoryError::DatabaseDisabled);
        }

        Ok(self.jobs_by_id.get(id).cloned())
    }

    fn list_recent(&self, limit: i64) -> Result<Vec<Job>, RepositoryError> {
        if self.database_disabled {
            return Err(RepositoryError::DatabaseDisabled);
        }

        Ok(self
            .recent_jobs
            .iter()
            .take(limit as usize)
            .cloned()
            .collect())
    }

    fn search(&self, _query: &str, limit: i64) -> Result<Vec<Job>, RepositoryError> {
        if self.database_disabled {
            return Err(RepositoryError::DatabaseDisabled);
        }

        Ok(self
            .search_jobs
            .iter()
            .take(limit as usize)
            .cloned()
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use crate::db::Database;
    use crate::db::repositories::RepositoryError;

    use super::{JobsService, JobsServiceStub};

    #[tokio::test]
    async fn returns_disabled_error_without_database() {
        let service = JobsService::new(crate::db::repositories::JobsRepository::new(
            Database::disabled(),
        ));

        let error = service
            .get_by_id("job-1")
            .await
            .expect_err("service should fail without configured database");

        assert!(matches!(error, RepositoryError::DatabaseDisabled));
    }

    #[tokio::test]
    async fn returns_none_for_unknown_job_in_stub() {
        let service = JobsService::for_tests(JobsServiceStub::default());

        let job = service
            .get_by_id("missing-job")
            .await
            .expect("stub should not fail");

        assert!(job.is_none());
    }
}
