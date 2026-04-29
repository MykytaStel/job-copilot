use serde::Serialize;

use crate::domain::market::model::{
    MarketCompanyEntry, MarketCompanyVelocityEntry, MarketFreezeSignalEntry, MarketOverview,
    MarketRegionDemandEntry, MarketRoleDemandEntry, MarketSalaryBySeniorityEntry,
    MarketSalaryTrend, MarketTechDemandEntry,
};

#[derive(Debug, Serialize)]
pub struct MarketOverviewResponse {
    pub new_jobs_this_week: i64,
    pub active_companies_count: i64,
    pub active_jobs_count: i64,
    pub remote_percentage: f64,
}

impl From<MarketOverview> for MarketOverviewResponse {
    fn from(m: MarketOverview) -> Self {
        Self {
            new_jobs_this_week: m.new_jobs_this_week,
            active_companies_count: m.active_companies_count,
            active_jobs_count: m.active_jobs_count,
            remote_percentage: m.remote_percentage,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct MarketCompanyEntryResponse {
    pub company_name: String,
    pub normalized_company_name: String,
    pub active_jobs: i64,
    pub this_week: i64,
    pub prev_week: i64,
    pub velocity: i64,
    pub sources: Vec<String>,
    pub top_role_groups: Vec<String>,
    pub latest_job_ids: Vec<String>,
    pub data_quality_flags: Vec<String>,
}

impl From<MarketCompanyEntry> for MarketCompanyEntryResponse {
    fn from(e: MarketCompanyEntry) -> Self {
        Self {
            velocity: e.this_week - e.prev_week,
            company_name: e.company_name,
            normalized_company_name: e.normalized_company_name,
            active_jobs: e.active_jobs,
            this_week: e.this_week,
            prev_week: e.prev_week,
            sources: e.sources,
            top_role_groups: e.top_role_groups,
            latest_job_ids: e.latest_job_ids,
            data_quality_flags: e.data_quality_flags,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct MarketCompaniesResponse {
    pub companies: Vec<MarketCompanyEntryResponse>,
}

#[derive(Debug, Serialize)]
pub struct MarketCompanyVelocityEntryResponse {
    pub company: String,
    pub job_count: i64,
    pub trend: String,
}

impl From<MarketCompanyVelocityEntry> for MarketCompanyVelocityEntryResponse {
    fn from(value: MarketCompanyVelocityEntry) -> Self {
        Self {
            company: value.company,
            job_count: value.job_count,
            trend: value.trend.as_str().to_string(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct MarketFreezeSignalEntryResponse {
    pub company: String,
    pub last_posted_at: String,
    pub days_since_last_post: u32,
    pub historical_count: u32,
}

impl From<MarketFreezeSignalEntry> for MarketFreezeSignalEntryResponse {
    fn from(value: MarketFreezeSignalEntry) -> Self {
        Self {
            company: value.company,
            last_posted_at: value.last_posted_at,
            days_since_last_post: value.days_since_last_post,
            historical_count: value.historical_count,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct MarketSalaryTrendResponse {
    pub seniority: String,
    pub currency: String,
    pub p25: i32,
    pub median: i32,
    pub p75: i32,
    pub sample_count: i64,
}

impl From<MarketSalaryTrend> for MarketSalaryTrendResponse {
    fn from(value: MarketSalaryTrend) -> Self {
        Self {
            seniority: value.seniority,
            currency: value.currency,
            p25: value.p25,
            median: value.median,
            p75: value.p75,
            sample_count: value.sample_count,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct MarketSalaryBySeniorityEntryResponse {
    pub seniority: String,
    pub median_min: u32,
    pub median_max: u32,
    pub sample_size: u32,
}

impl From<MarketSalaryBySeniorityEntry> for MarketSalaryBySeniorityEntryResponse {
    fn from(value: MarketSalaryBySeniorityEntry) -> Self {
        Self {
            seniority: value.seniority,
            median_min: value.median_min,
            median_max: value.median_max,
            sample_size: value.sample_size,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct MarketRoleDemandEntryResponse {
    pub role_group: String,
    pub this_period: i64,
    pub prev_period: i64,
    pub trend: String,
}

impl From<MarketRoleDemandEntry> for MarketRoleDemandEntryResponse {
    fn from(value: MarketRoleDemandEntry) -> Self {
        Self {
            role_group: value.role_group,
            this_period: value.this_period,
            prev_period: value.prev_period,
            trend: value.trend.as_str().to_string(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct MarketRegionDemandEntryResponse {
    pub region: String,
    pub job_count: u32,
    pub top_roles: Vec<String>,
}

impl From<MarketRegionDemandEntry> for MarketRegionDemandEntryResponse {
    fn from(value: MarketRegionDemandEntry) -> Self {
        Self {
            region: value.region,
            job_count: value.job_count,
            top_roles: value.top_roles,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct MarketTechDemandEntryResponse {
    pub skill: String,
    pub job_count: u32,
    pub percentage: f32,
}

impl From<MarketTechDemandEntry> for MarketTechDemandEntryResponse {
    fn from(value: MarketTechDemandEntry) -> Self {
        Self {
            skill: value.skill,
            job_count: value.job_count,
            percentage: value.percentage,
        }
    }
}
