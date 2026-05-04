use std::collections::HashMap;

use crate::db::repositories::RepositoryError;
use crate::domain::analytics::model::JobSourceCount;
use crate::domain::job::model::{Job, JobFeedSummary, JobView};
use crate::domain::market::model::{
    MarketCompanyDetail, MarketCompanyEntry, MarketCompanyVelocityEntry, MarketFreezeSignalEntry,
    MarketOverview, MarketRegionDemandEntry, MarketRoleDemandEntry, MarketSalaryBySeniorityEntry,
    MarketSalaryTrend, MarketSource, MarketTechDemandEntry,
};

pub struct JobsServiceStub {
    jobs_by_id: HashMap<String, Job>,
    job_views_by_id: HashMap<String, JobView>,
    recent_job_views: Vec<JobView>,
    search_jobs: Vec<Job>,
    feed_summary: JobFeedSummary,
    source_counts: Vec<JobSourceCount>,
    market_overview: MarketOverview,
    market_companies: Vec<MarketCompanyEntry>,
    market_company_details: HashMap<String, MarketCompanyDetail>,
    market_company_velocity: Vec<MarketCompanyVelocityEntry>,
    market_freeze_signals: Vec<MarketFreezeSignalEntry>,
    market_salary_trends: HashMap<String, MarketSalaryTrend>,
    market_salary_by_seniority: Vec<MarketSalaryBySeniorityEntry>,
    market_role_demand: Vec<MarketRoleDemandEntry>,
    market_region_breakdown: Vec<MarketRegionDemandEntry>,
    market_tech_demand: Vec<MarketTechDemandEntry>,
    database_disabled: bool,
}

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

    pub fn with_market_company_detail(mut self, slug: &str, detail: MarketCompanyDetail) -> Self {
        self.market_company_details.insert(slug.to_string(), detail);
        self
    }

    pub fn with_market_company_velocity(
        mut self,
        entries: Vec<MarketCompanyVelocityEntry>,
    ) -> Self {
        self.market_company_velocity = entries;
        self
    }

    pub fn with_market_freeze_signals(mut self, entries: Vec<MarketFreezeSignalEntry>) -> Self {
        self.market_freeze_signals = entries;
        self
    }

    pub fn with_market_salary_trend(mut self, trend: MarketSalaryTrend) -> Self {
        self.market_salary_trends
            .insert(trend.seniority.clone(), trend);
        self
    }

    pub fn with_market_salary_by_seniority(
        mut self,
        entries: Vec<MarketSalaryBySeniorityEntry>,
    ) -> Self {
        self.market_salary_by_seniority = entries;
        self
    }

    pub fn with_market_role_demand(mut self, entries: Vec<MarketRoleDemandEntry>) -> Self {
        self.market_role_demand = entries;
        self
    }

    pub fn with_market_region_breakdown(mut self, entries: Vec<MarketRegionDemandEntry>) -> Self {
        self.market_region_breakdown = entries;
        self
    }

    pub fn with_market_tech_demand(mut self, entries: Vec<MarketTechDemandEntry>) -> Self {
        self.market_tech_demand = entries;
        self
    }

    pub(crate) fn get_by_id(&self, id: &str) -> Result<Option<Job>, RepositoryError> {
        if self.database_disabled {
            return Err(RepositoryError::DatabaseDisabled);
        }

        Ok(self.jobs_by_id.get(id).cloned())
    }

    pub(crate) fn get_view_by_id(&self, id: &str) -> Result<Option<JobView>, RepositoryError> {
        if self.database_disabled {
            return Err(RepositoryError::DatabaseDisabled);
        }

        Ok(self.job_views_by_id.get(id).cloned())
    }

    pub(crate) fn get_views_by_ids(&self, ids: &[String]) -> Result<Vec<JobView>, RepositoryError> {
        if self.database_disabled {
            return Err(RepositoryError::DatabaseDisabled);
        }

        Ok(ids
            .iter()
            .filter_map(|id| self.job_views_by_id.get(id).cloned())
            .collect())
    }

    pub(crate) fn list_filtered_views(
        &self,
        limit: i64,
        lifecycle: Option<&str>,
        source: Option<&str>,
        _quality_min: Option<i32>,
    ) -> Result<Vec<JobView>, RepositoryError> {
        if self.database_disabled {
            return Err(RepositoryError::DatabaseDisabled);
        }

        let iter = self.recent_job_views.iter().filter(|view| {
            let lifecycle_ok =
                match lifecycle {
                    Some("inactive") => !view.job.is_active,
                    Some("reactivated") => {
                        view.job.is_active
                            && view.reactivated_at.as_ref().is_some_and(|reactivated_at| {
                                reactivated_at == &view.job.last_seen_at
                            })
                    }
                    Some("active") => {
                        view.job.is_active
                            && !view.reactivated_at.as_ref().is_some_and(|reactivated_at| {
                                reactivated_at == &view.job.last_seen_at
                            })
                    }
                    _ => true,
                };
            let source_ok = match source {
                Some(expected_source) => view
                    .primary_variant
                    .as_ref()
                    .is_some_and(|variant| variant.source == expected_source),
                None => true,
            };
            lifecycle_ok && source_ok
        });

        Ok(iter.take(limit as usize).cloned().collect())
    }

    pub(crate) fn search_active(
        &self,
        _query: &str,
        limit: i64,
    ) -> Result<Vec<Job>, RepositoryError> {
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

    pub(crate) fn feed_summary(&self) -> Result<JobFeedSummary, RepositoryError> {
        if self.database_disabled {
            return Err(RepositoryError::DatabaseDisabled);
        }

        Ok(self.feed_summary.clone())
    }

    pub(crate) fn jobs_by_source(&self) -> Result<Vec<JobSourceCount>, RepositoryError> {
        if self.database_disabled {
            return Err(RepositoryError::DatabaseDisabled);
        }

        Ok(self.source_counts.clone())
    }

    pub(crate) fn market_overview(
        &self,
    ) -> Result<(MarketOverview, MarketSource), RepositoryError> {
        if self.database_disabled {
            return Err(RepositoryError::DatabaseDisabled);
        }

        Ok((self.market_overview.clone(), MarketSource::Snapshot))
    }

    pub(crate) fn market_companies(
        &self,
        limit: i64,
    ) -> Result<(Vec<MarketCompanyEntry>, MarketSource), RepositoryError> {
        if self.database_disabled {
            return Err(RepositoryError::DatabaseDisabled);
        }

        Ok((
            self.market_companies
                .iter()
                .take(limit as usize)
                .cloned()
                .collect(),
            MarketSource::Snapshot,
        ))
    }

    pub(crate) fn market_company_detail(
        &self,
        company_slug: &str,
    ) -> Result<Option<MarketCompanyDetail>, RepositoryError> {
        if self.database_disabled {
            return Err(RepositoryError::DatabaseDisabled);
        }

        Ok(self.market_company_details.get(company_slug).cloned())
    }

    pub(crate) fn market_salary_trend(
        &self,
        seniority: &str,
    ) -> Result<(Option<MarketSalaryTrend>, MarketSource), RepositoryError> {
        if self.database_disabled {
            return Err(RepositoryError::DatabaseDisabled);
        }

        Ok((
            self.market_salary_trends.get(seniority).cloned(),
            MarketSource::Snapshot,
        ))
    }

    pub(crate) fn market_company_velocity(
        &self,
    ) -> Result<(Vec<MarketCompanyVelocityEntry>, MarketSource), RepositoryError> {
        if self.database_disabled {
            return Err(RepositoryError::DatabaseDisabled);
        }

        Ok((self.market_company_velocity.clone(), MarketSource::Snapshot))
    }

    pub(crate) fn market_freeze_signals(
        &self,
    ) -> Result<(Vec<MarketFreezeSignalEntry>, MarketSource), RepositoryError> {
        if self.database_disabled {
            return Err(RepositoryError::DatabaseDisabled);
        }

        Ok((self.market_freeze_signals.clone(), MarketSource::Snapshot))
    }

    pub(crate) fn market_salary_trends(
        &self,
    ) -> Result<(Vec<MarketSalaryTrend>, MarketSource), RepositoryError> {
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

        Ok((trends, MarketSource::Snapshot))
    }

    pub(crate) fn market_salary_by_seniority(
        &self,
    ) -> Result<(Vec<MarketSalaryBySeniorityEntry>, MarketSource), RepositoryError> {
        if self.database_disabled {
            return Err(RepositoryError::DatabaseDisabled);
        }

        Ok((
            self.market_salary_by_seniority.clone(),
            MarketSource::Snapshot,
        ))
    }

    pub(crate) fn market_role_demand(
        &self,
        _period_days: i32,
    ) -> Result<(Vec<MarketRoleDemandEntry>, MarketSource), RepositoryError> {
        if self.database_disabled {
            return Err(RepositoryError::DatabaseDisabled);
        }

        Ok((self.market_role_demand.clone(), MarketSource::Snapshot))
    }

    pub(crate) fn market_region_breakdown(
        &self,
    ) -> Result<(Vec<MarketRegionDemandEntry>, MarketSource), RepositoryError> {
        if self.database_disabled {
            return Err(RepositoryError::DatabaseDisabled);
        }

        Ok((self.market_region_breakdown.clone(), MarketSource::Snapshot))
    }

    pub(crate) fn market_tech_demand(
        &self,
    ) -> Result<(Vec<MarketTechDemandEntry>, MarketSource), RepositoryError> {
        if self.database_disabled {
            return Err(RepositoryError::DatabaseDisabled);
        }

        Ok((self.market_tech_demand.clone(), MarketSource::Snapshot))
    }
}

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
                last_ingested_at: None,
            },
            source_counts: Vec::new(),
            market_overview: MarketOverview {
                new_jobs_this_week: 0,
                active_companies_count: 0,
                active_jobs_count: 0,
                remote_percentage: 0.0,
            },
            market_companies: Vec::new(),
            market_company_details: HashMap::new(),
            market_company_velocity: Vec::new(),
            market_freeze_signals: Vec::new(),
            market_salary_trends: HashMap::new(),
            market_salary_by_seniority: Vec::new(),
            market_role_demand: Vec::new(),
            market_region_breakdown: Vec::new(),
            market_tech_demand: Vec::new(),
            database_disabled: false,
        }
    }
}
