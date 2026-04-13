use serde::Serialize;

use crate::domain::analytics::model::SalaryBucket;

#[derive(Debug, Serialize)]
pub struct SalaryBucketResponse {
    pub seniority: Option<String>,
    pub currency: Option<String>,
    pub salary_min: Option<i32>,
    pub salary_max: Option<i32>,
    pub salary_avg: Option<f64>,
    pub job_count: i64,
}

#[derive(Debug, Serialize)]
pub struct SalaryIntelligenceResponse {
    pub buckets: Vec<SalaryBucketResponse>,
}

impl From<SalaryBucket> for SalaryBucketResponse {
    fn from(bucket: SalaryBucket) -> Self {
        Self {
            seniority: bucket.seniority,
            currency: bucket.currency,
            salary_min: bucket.min,
            salary_max: bucket.max,
            salary_avg: bucket.avg,
            job_count: bucket.job_count,
        }
    }
}
