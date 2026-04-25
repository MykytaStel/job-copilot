use crate::db::repositories::{JobsRepository, RepositoryError};
use crate::domain::analytics::model::SalaryBucket;
use crate::domain::job::model::Job;

const UAH_TO_USD: f64 = 1.0 / 41.0;
const EUR_TO_USD: f64 = 1.09;
const MAX_SALARY_BOOST: i16 = 8;
const MAX_SALARY_PENALTY: i16 = -8;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SearchSalaryExpectation {
    pub min: Option<i32>,
    pub max: Option<i32>,
    pub currency: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SearchSalaryScore {
    pub score_delta: i16,
    pub reason: Option<String>,
}

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

pub fn score_search_salary(
    expectation: Option<&SearchSalaryExpectation>,
    job: &Job,
) -> SearchSalaryScore {
    let Some(expectation) = expectation else {
        return SearchSalaryScore::default();
    };
    let (Some(candidate_min), Some(candidate_max)) = (expectation.min, expectation.max) else {
        return SearchSalaryScore::default();
    };
    let (Some(job_min), Some(job_max)) = (job.salary_min, job.salary_max) else {
        return SearchSalaryScore::default();
    };

    let candidate_min = normalize_to_usd(candidate_min, &expectation.currency);
    let candidate_max = normalize_to_usd(candidate_max, &expectation.currency);
    let job_min = normalize_to_usd(job_min, job.salary_currency.as_deref().unwrap_or("USD"));
    let job_max = normalize_to_usd(job_max, job.salary_currency.as_deref().unwrap_or("USD"));

    if job_max < candidate_min {
        return SearchSalaryScore {
            score_delta: MAX_SALARY_PENALTY,
            reason: Some("Salary range is below the profile target".to_string()),
        };
    }

    if job_min >= candidate_min {
        return SearchSalaryScore {
            score_delta: MAX_SALARY_BOOST,
            reason: Some("Salary range meets or exceeds the profile target".to_string()),
        };
    }

    let overlap = (candidate_max.min(job_max) - candidate_min.max(job_min)).max(0.0);
    let job_range = (job_max - job_min).max(1.0);
    let overlap_ratio = (overlap / job_range).clamp(0.0, 1.0);
    let score_delta = (overlap_ratio * f64::from(MAX_SALARY_BOOST)).round() as i16;

    if score_delta == 0 {
        return SearchSalaryScore::default();
    }

    SearchSalaryScore {
        score_delta,
        reason: Some("Salary range overlaps the profile target".to_string()),
    }
}

fn normalize_to_usd(amount: i32, currency: &str) -> f64 {
    let amount = amount as f64;

    match currency.to_uppercase().trim() {
        "UAH" | "UAH/MONTH" => amount * UAH_TO_USD,
        "EUR" => amount * EUR_TO_USD,
        _ => amount,
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::job::model::Job;

    use super::{SearchSalaryExpectation, score_search_salary};

    fn job() -> Job {
        Job {
            id: "job-1".to_string(),
            title: "Senior Backend Engineer".to_string(),
            company_name: "NovaLedger".to_string(),
            location: None,
            remote_type: Some("remote".to_string()),
            seniority: Some("senior".to_string()),
            description_text: "Rust and Postgres".to_string(),
            salary_min: Some(5000),
            salary_max: Some(7000),
            salary_currency: Some("USD".to_string()),
            posted_at: None,
            last_seen_at: "2026-04-14T00:00:00Z".to_string(),
            is_active: true,
        }
    }

    #[test]
    fn boosts_when_job_salary_meets_target() {
        let score = score_search_salary(
            Some(&SearchSalaryExpectation {
                min: Some(4000),
                max: Some(6000),
                currency: "USD".to_string(),
            }),
            &job(),
        );

        assert_eq!(score.score_delta, 8);
        assert_eq!(
            score.reason.as_deref(),
            Some("Salary range meets or exceeds the profile target")
        );
    }

    #[test]
    fn penalizes_when_job_salary_is_below_target() {
        let mut low_salary_job = job();
        low_salary_job.salary_min = Some(2000);
        low_salary_job.salary_max = Some(3000);

        let score = score_search_salary(
            Some(&SearchSalaryExpectation {
                min: Some(4000),
                max: Some(6000),
                currency: "USD".to_string(),
            }),
            &low_salary_job,
        );

        assert_eq!(score.score_delta, -8);
        assert_eq!(
            score.reason.as_deref(),
            Some("Salary range is below the profile target")
        );
    }

    #[test]
    fn returns_zero_when_no_expectation_set() {
        let score = score_search_salary(None, &job());
        assert_eq!(score.score_delta, 0);
        assert!(score.reason.is_none());
    }

    #[test]
    fn returns_zero_when_job_has_no_salary_data() {
        let mut no_salary_job = job();
        no_salary_job.salary_min = None;
        no_salary_job.salary_max = None;

        let score = score_search_salary(
            Some(&SearchSalaryExpectation {
                min: Some(4000),
                max: Some(6000),
                currency: "USD".to_string(),
            }),
            &no_salary_job,
        );

        assert_eq!(score.score_delta, 0);
        assert!(score.reason.is_none());
    }

    #[test]
    fn partial_overlap_scales_score_proportionally() {
        // job range 2000-5000 USD, candidate wants 4000-6000 USD
        // overlap = 5000-4000 = 1000, job_range = 3000
        // ratio ≈ 0.333 → round(0.333 * 8) = 3
        let mut partial_job = job();
        partial_job.salary_min = Some(2000);
        partial_job.salary_max = Some(5000);

        let score = score_search_salary(
            Some(&SearchSalaryExpectation {
                min: Some(4000),
                max: Some(6000),
                currency: "USD".to_string(),
            }),
            &partial_job,
        );

        assert_eq!(score.score_delta, 3);
        assert_eq!(
            score.reason.as_deref(),
            Some("Salary range overlaps the profile target")
        );
    }

    #[test]
    fn uah_salary_is_normalized_before_comparison() {
        // 200 000-280 000 UAH ÷ 41 ≈ 4878-6829 USD
        // job_min_usd (≈4878) >= candidate_min (4000) → full boost
        let mut uah_job = job();
        uah_job.salary_min = Some(200_000);
        uah_job.salary_max = Some(280_000);
        uah_job.salary_currency = Some("UAH".to_string());

        let score = score_search_salary(
            Some(&SearchSalaryExpectation {
                min: Some(4000),
                max: Some(6000),
                currency: "USD".to_string(),
            }),
            &uah_job,
        );

        assert_eq!(score.score_delta, 8);
    }
}
