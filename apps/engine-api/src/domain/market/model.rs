#[derive(Clone, Debug, PartialEq)]
pub struct MarketOverview {
    pub new_jobs_this_week: i64,
    pub active_companies_count: i64,
    pub active_jobs_count: i64,
    pub remote_percentage: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MarketCompanyEntry {
    pub company_name: String,
    pub active_jobs: i64,
    pub this_week: i64,
    pub prev_week: i64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MarketSalaryTrend {
    pub seniority: String,
    pub p25: i32,
    pub median: i32,
    pub p75: i32,
    pub sample_count: i64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MarketRoleDemandEntry {
    pub role_group: String,
    pub this_period: i64,
    pub prev_period: i64,
    pub trend: MarketTrendDirection,
}
