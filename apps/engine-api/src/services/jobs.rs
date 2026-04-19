#[cfg(test)]
use std::collections::HashMap;
#[cfg(test)]
use std::sync::Arc;

use crate::db::repositories::{JobsRepository, RepositoryError};
use crate::domain::analytics::model::JobSourceCount;
use crate::domain::job::model::{Job, JobFeedSummary, JobView};
use crate::domain::market::model::{
    MarketCompanyEntry, MarketOverview, MarketRoleDemandEntry, MarketSalaryTrend,
};

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

    pub async fn list_recent_views(&self, limit: i64) -> Result<Vec<JobView>, RepositoryError> {
        match &self.backend {
            JobsServiceBackend::Repository(repository) => repository.list_recent_views(limit).await,
            #[cfg(test)]
            JobsServiceBackend::Stub(stub) => stub.list_recent_views(limit),
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

    pub async fn market_overview(&self) -> Result<MarketOverview, RepositoryError> {
        match &self.backend {
            JobsServiceBackend::Repository(repository) => repository.market_overview().await,
            #[cfg(test)]
            JobsServiceBackend::Stub(stub) => stub.market_overview(),
        }
    }

    pub async fn market_companies(
        &self,
        limit: i64,
    ) -> Result<Vec<MarketCompanyEntry>, RepositoryError> {
        match &self.backend {
            JobsServiceBackend::Repository(repository) => repository.market_companies(limit).await,
            #[cfg(test)]
            JobsServiceBackend::Stub(stub) => stub.market_companies(limit),
        }
    }

    pub async fn market_salary_trend(
        &self,
        seniority: &str,
    ) -> Result<Option<MarketSalaryTrend>, RepositoryError> {
        match &self.backend {
            JobsServiceBackend::Repository(repository) => {
                repository.market_salary_trend(seniority).await
            }
            #[cfg(test)]
            JobsServiceBackend::Stub(stub) => stub.market_salary_trend(seniority),
        }
    }

    pub async fn market_salary_trends(&self) -> Result<Vec<MarketSalaryTrend>, RepositoryError> {
        match &self.backend {
            JobsServiceBackend::Repository(repository) => repository.market_salary_trends().await,
            #[cfg(test)]
            JobsServiceBackend::Stub(stub) => stub.market_salary_trends(),
        }
    }

    pub async fn market_role_demand(
        &self,
        period_days: i32,
    ) -> Result<Vec<MarketRoleDemandEntry>, RepositoryError> {
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

#[cfg(test)]
pub struct JobsServiceStub {
    jobs_by_id: HashMap<String, Job>,
    job_views_by_id: HashMap<String, JobView>,
    recent_job_views: Vec<JobView>,
    search_jobs: Vec<Job>,
    feed_summary: JobFeedSummary,
    source_counts: Vec<JobSourceCount>,
    market_overview: MarketOverview,
    market_companies: Vec<MarketCompanyEntry>,
    market_salary_trends: HashMap<String, MarketSalaryTrend>,
    market_role_demand: Vec<MarketRoleDemandEntry>,
    database_disabled: bool,
}

#[cfg(test)]
impl JobsServiceStub {
    pub fn with_job(mut self, job: Job) -> Self {
        self.jobs_by_id.insert(job.id.clone(), job.clone());
        self.search_jobs.push(job);
        self
    }

    pub fn with_job_view(mut self, job_view: JobView) -> Self {
        self.job_views_by_id
            .insert(job_view.job.id.clone(), job_view.clone());
        self.recent_job_views.push(job_view);
        self
    }

    pub fn with_feed_summary(mut self, summary: JobFeedSummary) -> Self {
        self.feed_summary = summary;
        self
    }

    pub fn with_jobs_by_source(mut self, counts: Vec<JobSourceCount>) -> Self {
        self.source_counts = counts;
        self
    }

    pub fn with_market_overview(mut self, overview: MarketOverview) -> Self {
        self.market_overview = overview;
        self
    }

    pub fn with_market_companies(mut self, companies: Vec<MarketCompanyEntry>) -> Self {
        self.market_companies = companies;
        self
    }

    pub fn with_market_salary_trend(mut self, trend: MarketSalaryTrend) -> Self {
        self.market_salary_trends
            .insert(trend.seniority.clone(), trend);
        self
    }

    pub fn with_market_role_demand(mut self, entries: Vec<MarketRoleDemandEntry>) -> Self {
        self.market_role_demand = entries;
        self
    }

    fn get_by_id(&self, id: &str) -> Result<Option<Job>, RepositoryError> {
        if self.database_disabled {
            return Err(RepositoryError::DatabaseDisabled);
        }

        Ok(self.jobs_by_id.get(id).cloned())
    }

    fn get_view_by_id(&self, id: &str) -> Result<Option<JobView>, RepositoryError> {
        if self.database_disabled {
            return Err(RepositoryError::DatabaseDisabled);
        }

        Ok(self.job_views_by_id.get(id).cloned())
    }

    fn list_recent_views(&self, limit: i64) -> Result<Vec<JobView>, RepositoryError> {
        if self.database_disabled {
            return Err(RepositoryError::DatabaseDisabled);
        }

        Ok(self
            .recent_job_views
            .iter()
            .take(limit as usize)
            .cloned()
            .collect())
    }

    fn list_filtered_views(
        &self,
        limit: i64,
        lifecycle: Option<&str>,
        source: Option<&str>,
    ) -> Result<Vec<JobView>, RepositoryError> {
        if self.database_disabled {
            return Err(RepositoryError::DatabaseDisabled);
        }

        let iter = self.recent_job_views.iter().filter(|v| {
            let lifecycle_ok = match lifecycle {
                Some("inactive") => !v.job.is_active,
                Some("reactivated") => {
                    v.job.is_active
                        && v.reactivated_at
                            .as_ref()
                            .is_some_and(|r| r == &v.job.last_seen_at)
                }
                Some("active") => {
                    v.job.is_active
                        && !v
                            .reactivated_at
                            .as_ref()
                            .is_some_and(|r| r == &v.job.last_seen_at)
                }
                _ => true,
            };
            let source_ok = match source {
                Some(s) => v.primary_variant.as_ref().is_some_and(|pv| pv.source == s),
                None => true,
            };
            lifecycle_ok && source_ok
        });

        Ok(iter.take(limit as usize).cloned().collect())
    }

    fn search_active(&self, _query: &str, limit: i64) -> Result<Vec<Job>, RepositoryError> {
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

    fn feed_summary(&self) -> Result<JobFeedSummary, RepositoryError> {
        if self.database_disabled {
            return Err(RepositoryError::DatabaseDisabled);
        }

        Ok(self.feed_summary.clone())
    }

    fn jobs_by_source(&self) -> Result<Vec<JobSourceCount>, RepositoryError> {
        if self.database_disabled {
            return Err(RepositoryError::DatabaseDisabled);
        }

        Ok(self.source_counts.clone())
    }

    fn market_overview(&self) -> Result<MarketOverview, RepositoryError> {
        if self.database_disabled {
            return Err(RepositoryError::DatabaseDisabled);
        }

        Ok(self.market_overview.clone())
    }

    fn market_companies(&self, limit: i64) -> Result<Vec<MarketCompanyEntry>, RepositoryError> {
        if self.database_disabled {
            return Err(RepositoryError::DatabaseDisabled);
        }

        Ok(self
            .market_companies
            .iter()
            .take(limit as usize)
            .cloned()
            .collect())
    }

    fn market_salary_trend(
        &self,
        seniority: &str,
    ) -> Result<Option<MarketSalaryTrend>, RepositoryError> {
        if self.database_disabled {
            return Err(RepositoryError::DatabaseDisabled);
        }

        Ok(self.market_salary_trends.get(seniority).cloned())
    }

    fn market_salary_trends(&self) -> Result<Vec<MarketSalaryTrend>, RepositoryError> {
        if self.database_disabled {
            return Err(RepositoryError::DatabaseDisabled);
        }

        let mut trends = self
            .market_salary_trends
            .values()
            .cloned()
            .collect::<Vec<_>>();
        trends.sort_by_key(|trend| match trend.seniority.as_str() {
            "intern" => (0, trend.seniority.clone()),
            "junior" => (1, trend.seniority.clone()),
            "middle" | "mid" => (2, trend.seniority.clone()),
            "senior" => (3, trend.seniority.clone()),
            "lead" => (4, trend.seniority.clone()),
            _ => (5, trend.seniority.clone()),
        });

        Ok(trends)
    }

    fn market_role_demand(
        &self,
        _period_days: i32,
    ) -> Result<Vec<MarketRoleDemandEntry>, RepositoryError> {
        if self.database_disabled {
            return Err(RepositoryError::DatabaseDisabled);
        }

        Ok(self.market_role_demand.clone())
    }
}

#[cfg(test)]
impl Default for JobsServiceStub {
    fn default() -> Self {
        Self {
            jobs_by_id: HashMap::new(),
            job_views_by_id: HashMap::new(),
            recent_job_views: Vec::new(),
            search_jobs: Vec::new(),
            feed_summary: JobFeedSummary {
                total_jobs: 0,
                active_jobs: 0,
                inactive_jobs: 0,
                reactivated_jobs: 0,
            },
            source_counts: Vec::new(),
            market_overview: MarketOverview {
                new_jobs_this_week: 0,
                active_companies_count: 0,
                active_jobs_count: 0,
                remote_percentage: 0.0,
            },
            market_companies: Vec::new(),
            market_salary_trends: HashMap::new(),
            market_role_demand: Vec::new(),
            database_disabled: false,
        }
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
