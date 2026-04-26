#[cfg(test)]
#[path = "jobs/stub.rs"]
mod stub;
#[cfg(test)]
#[path = "jobs/tests.rs"]
mod tests;

#[cfg(test)]
use std::sync::Arc;

use crate::db::repositories::{JobsRepository, RepositoryError};
use crate::domain::analytics::model::JobSourceCount;
use crate::domain::job::model::{Job, JobFeedSummary, JobView};
use crate::domain::market::model::{
    MarketCompanyEntry, MarketOverview, MarketRoleDemandEntry, MarketSalaryTrend, MarketSource,
};

#[cfg(test)]
pub use stub::JobsServiceStub;

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

    pub async fn get_view_by_id(&self, id: &str) -> Result<Option<JobView>, RepositoryError> {
        match &self.backend {
            JobsServiceBackend::Repository(repository) => repository.get_view_by_id(id).await,
            #[cfg(test)]
            JobsServiceBackend::Stub(stub) => stub.get_view_by_id(id),
        }
    }

    pub async fn list_filtered_views(
        &self,
        limit: i64,
        lifecycle: Option<&str>,
        source: Option<&str>,
    ) -> Result<Vec<JobView>, RepositoryError> {
        match &self.backend {
            JobsServiceBackend::Repository(repository) => {
                repository
                    .list_filtered_views(limit, lifecycle, source)
                    .await
            }
            #[cfg(test)]
            JobsServiceBackend::Stub(stub) => stub.list_filtered_views(limit, lifecycle, source),
        }
    }

    pub async fn feed_summary(&self) -> Result<JobFeedSummary, RepositoryError> {
        match &self.backend {
            JobsServiceBackend::Repository(repository) => repository.feed_summary().await,
            #[cfg(test)]
            JobsServiceBackend::Stub(stub) => stub.feed_summary(),
        }
    }

    pub async fn search_active(
        &self,
        query: &str,
        limit: i64,
    ) -> Result<Vec<Job>, RepositoryError> {
        match &self.backend {
            JobsServiceBackend::Repository(repository) => {
                repository.search_active(query, limit).await
            }
            #[cfg(test)]
            JobsServiceBackend::Stub(stub) => stub.search_active(query, limit),
        }
    }

    pub async fn jobs_by_source(&self) -> Result<Vec<JobSourceCount>, RepositoryError> {
        match &self.backend {
            JobsServiceBackend::Repository(repository) => repository.jobs_by_source().await,
            #[cfg(test)]
            JobsServiceBackend::Stub(stub) => stub.jobs_by_source(),
        }
    }

    pub async fn market_overview(
        &self,
    ) -> Result<(MarketOverview, MarketSource), RepositoryError> {
        match &self.backend {
            JobsServiceBackend::Repository(repository) => repository.market_overview().await,
            #[cfg(test)]
            JobsServiceBackend::Stub(stub) => stub.market_overview(),
        }
    }

    pub async fn market_companies(
        &self,
        limit: i64,
    ) -> Result<(Vec<MarketCompanyEntry>, MarketSource), RepositoryError> {
        match &self.backend {
            JobsServiceBackend::Repository(repository) => repository.market_companies(limit).await,
            #[cfg(test)]
            JobsServiceBackend::Stub(stub) => stub.market_companies(limit),
        }
    }

    pub async fn market_salary_trend(
        &self,
        seniority: &str,
    ) -> Result<(Option<MarketSalaryTrend>, MarketSource), RepositoryError> {
        match &self.backend {
            JobsServiceBackend::Repository(repository) => {
                repository.market_salary_trend(seniority).await
            }
            #[cfg(test)]
            JobsServiceBackend::Stub(stub) => stub.market_salary_trend(seniority),
        }
    }

    pub async fn market_salary_trends(
        &self,
    ) -> Result<(Vec<MarketSalaryTrend>, MarketSource), RepositoryError> {
        match &self.backend {
            JobsServiceBackend::Repository(repository) => repository.market_salary_trends().await,
            #[cfg(test)]
            JobsServiceBackend::Stub(stub) => stub.market_salary_trends(),
        }
    }

    pub async fn market_role_demand(
        &self,
        period_days: i32,
    ) -> Result<(Vec<MarketRoleDemandEntry>, MarketSource), RepositoryError> {
        match &self.backend {
            JobsServiceBackend::Repository(repository) => {
                repository.market_role_demand(period_days).await
            }
            #[cfg(test)]
            JobsServiceBackend::Stub(stub) => stub.market_role_demand(period_days),
        }
    }

    #[cfg(test)]
    pub fn for_tests(stub: JobsServiceStub) -> Self {
        Self {
            backend: JobsServiceBackend::Stub(Arc::new(stub)),
        }
    }
}
