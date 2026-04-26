pub struct SalaryBucket {
    pub seniority: Option<String>,
    pub currency: Option<String>,
    pub min: Option<i32>,
    pub max: Option<i32>,
    pub avg: Option<f64>,
    pub job_count: i64,
}

#[derive(Clone, Debug)]
pub struct JobSourceCount {
    pub source: String,
    pub count: i64,
    pub last_seen: String,
}
