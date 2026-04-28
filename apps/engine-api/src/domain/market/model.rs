use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MarketSource {
    Snapshot,
    Live,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MarketOverview {
    pub new_jobs_this_week: i64,
    pub active_companies_count: i64,
    pub active_jobs_count: i64,
    pub remote_percentage: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MarketCompanyEntry {
    pub company_name: String,
    #[serde(default)]
    pub normalized_company_name: String,
    pub active_jobs: i64,
    pub this_week: i64,
    pub prev_week: i64,
    #[serde(default)]
    pub sources: Vec<String>,
    #[serde(default)]
    pub top_role_groups: Vec<String>,
    #[serde(default)]
    pub latest_job_ids: Vec<String>,
    #[serde(default)]
    pub data_quality_flags: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MarketSalaryTrend {
    pub seniority: String,
    #[serde(default = "default_salary_currency")]
    pub currency: String,
    pub p25: i32,
    pub median: i32,
    pub p75: i32,
    pub sample_count: i64,
}

fn default_salary_currency() -> String {
    "UNKNOWN".to_string()
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MarketTrendDirection {
    Up,
    Down,
    Stable,
}

impl MarketTrendDirection {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Up => "up",
            Self::Down => "down",
            Self::Stable => "stable",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MarketRoleDemandEntry {
    pub role_group: String,
    pub this_period: i64,
    pub prev_period: i64,
    pub trend: MarketTrendDirection,
}
