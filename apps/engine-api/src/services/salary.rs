use crate::db::repositories::{JobsRepository, RepositoryError};
use crate::domain::analytics::model::SalaryBucket;

#[derive(Clone)]
pub struct SalaryService {
    jobs_repository: JobsRepository,
}

impl SalaryService {
    pub fn new(jobs_repository: JobsRepository) -> Self {
        Self { jobs_repository }
    }

    pub async fn salary_intelligence(&self) -> Result<Vec<SalaryBucket>, RepositoryError> {
        self.jobs_repository.salary_intelligence().await
    }
}
