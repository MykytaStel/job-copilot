use crate::db::repositories::{JobsRepository, RepositoryError};
use crate::domain::analytics::model::SalaryBucket;
use crate::domain::job::model::Job;

const UAH_TO_USD: f64 = 1.0 / 41.0;
const EUR_TO_USD: f64 = 1.09;

const FULL_SALARY_FIT_BONUS: i16 = 10;
const PARTIAL_SALARY_OVERLAP_BONUS: i16 = 5;
const BELOW_TARGET_SALARY_PENALTY: i16 = -10;
const BELOW_TARGET_TOLERANCE_RATIO: f64 = 0.20;

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
    pub label: Option<String>,
    pub missing_salary: bool,
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

    let candidate_min = normalize_to_usd(candidate_min, &expectation.currency);
    let candidate_max = normalize_to_usd(candidate_max, &expectation.currency);

    if candidate_min <= 0.0 || candidate_max <= 0.0 || candidate_min > candidate_max {
        return SearchSalaryScore::default();
    }

    let (Some(job_min), Some(job_max)) = (job.salary_min, job.salary_max) else {
        return SearchSalaryScore {
            score_delta: 0,
            reason: None,
            label: Some("Salary not provided".to_string()),
            missing_salary: true,
        };
    };

    let job_currency = job.salary_currency.as_deref().unwrap_or("USD");
    let job_min = normalize_to_usd(job_min, job_currency);
    let job_max = normalize_to_usd(job_max, job_currency);

    if job_min <= 0.0 || job_max <= 0.0 || job_min > job_max {
        return SearchSalaryScore {
            score_delta: 0,
            reason: None,
            label: Some("Salary not provided".to_string()),
            missing_salary: true,
        };
    }

    if job_min >= candidate_min && job_max <= candidate_max {
        return SearchSalaryScore {
            score_delta: FULL_SALARY_FIT_BONUS,
            reason: Some("Salary range is fully within the profile target".to_string()),
            label: Some("Salary fits target".to_string()),
            missing_salary: false,
        };
    }

    if ranges_overlap(candidate_min, candidate_max, job_min, job_max) {
        return SearchSalaryScore {
            score_delta: PARTIAL_SALARY_OVERLAP_BONUS,
            reason: Some("Salary range overlaps the profile target".to_string()),
            label: Some("Salary partially overlaps target".to_string()),
            missing_salary: false,
        };
    }

    if job_max < candidate_min * (1.0 - BELOW_TARGET_TOLERANCE_RATIO) {
        return SearchSalaryScore {
            score_delta: BELOW_TARGET_SALARY_PENALTY,
            reason: Some("Salary range is more than 20% below the profile minimum".to_string()),
            label: Some("Salary below target".to_string()),
            missing_salary: false,
        };
    }

    SearchSalaryScore {
        score_delta: 0,
        reason: Some("Salary range is outside the profile target".to_string()),
        label: Some("Salary outside target".to_string()),
        missing_salary: false,
    }
}

fn ranges_overlap(candidate_min: f64, candidate_max: f64, job_min: f64, job_max: f64) -> bool {
    job_min <= candidate_max && job_max >= candidate_min
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

    fn expectation() -> SearchSalaryExpectation {
        SearchSalaryExpectation {
            min: Some(4000),
            max: Some(7000),
            currency: "USD".to_string(),
        }
    }

    #[test]
    fn returns_zero_when_no_expectation_set() {
        let score = score_search_salary(None, &job());

        assert_eq!(score.score_delta, 0);
        assert!(score.reason.is_none());
        assert!(score.label.is_none());
        assert!(!score.missing_salary);
    }

    #[test]
    fn returns_zero_when_profile_salary_range_is_incomplete() {
        let score = score_search_salary(
            Some(&SearchSalaryExpectation {
                min: Some(4000),
                max: None,
                currency: "USD".to_string(),
            }),
            &job(),
        );

        assert_eq!(score.score_delta, 0);
        assert!(score.reason.is_none());
        assert!(!score.missing_salary);
    }

    #[test]
    fn flags_missing_salary_without_penalty() {
        let mut no_salary_job = job();
        no_salary_job.salary_min = None;
        no_salary_job.salary_max = None;

        let score = score_search_salary(Some(&expectation()), &no_salary_job);

        assert_eq!(score.score_delta, 0);
        assert!(score.reason.is_none());
        assert_eq!(score.label.as_deref(), Some("Salary not provided"));
        assert!(score.missing_salary);
    }

    #[test]
    fn boosts_when_job_salary_is_fully_inside_candidate_range() {
        let mut matching_job = job();
        matching_job.salary_min = Some(4500);
        matching_job.salary_max = Some(6500);

        let score = score_search_salary(Some(&expectation()), &matching_job);

        assert_eq!(score.score_delta, 10);
        assert_eq!(
            score.reason.as_deref(),
            Some("Salary range is fully within the profile target")
        );
        assert_eq!(score.label.as_deref(), Some("Salary fits target"));
        assert!(!score.missing_salary);
    }

    #[test]
    fn boosts_when_job_salary_overlaps_candidate_range() {
        let mut overlap_job = job();
        overlap_job.salary_min = Some(6500);
        overlap_job.salary_max = Some(8500);

        let score = score_search_salary(Some(&expectation()), &overlap_job);

        assert_eq!(score.score_delta, 5);
        assert_eq!(
            score.reason.as_deref(),
            Some("Salary range overlaps the profile target")
        );
        assert_eq!(
            score.label.as_deref(),
            Some("Salary partially overlaps target")
        );
    }

    #[test]
    fn penalizes_when_job_salary_is_more_than_twenty_percent_below_minimum() {
        let mut low_salary_job = job();
        low_salary_job.salary_min = Some(2500);
        low_salary_job.salary_max = Some(3100);

        let score = score_search_salary(Some(&expectation()), &low_salary_job);

        assert_eq!(score.score_delta, -10);
        assert_eq!(
            score.reason.as_deref(),
            Some("Salary range is more than 20% below the profile minimum")
        );
        assert_eq!(score.label.as_deref(), Some("Salary below target"));
    }

    #[test]
    fn does_not_penalize_when_below_target_but_within_twenty_percent_tolerance() {
        let mut near_target_job = job();
        near_target_job.salary_min = Some(3000);
        near_target_job.salary_max = Some(3300);

        let score = score_search_salary(Some(&expectation()), &near_target_job);

        assert_eq!(score.score_delta, 0);
        assert_eq!(
            score.reason.as_deref(),
            Some("Salary range is outside the profile target")
        );
        assert_eq!(score.label.as_deref(), Some("Salary outside target"));
    }

    #[test]
    fn normalizes_uah_salary_before_comparison() {
        let mut uah_job = job();
        uah_job.salary_min = Some(184_500);
        uah_job.salary_max = Some(266_500);
        uah_job.salary_currency = Some("UAH".to_string());

        let score = score_search_salary(Some(&expectation()), &uah_job);

        assert_eq!(score.score_delta, 10);
        assert_eq!(score.label.as_deref(), Some("Salary fits target"));
    }

    #[test]
    fn normalizes_eur_salary_before_comparison() {
        let mut eur_job = job();
        eur_job.salary_min = Some(4200);
        eur_job.salary_max = Some(5600);
        eur_job.salary_currency = Some("EUR".to_string());

        let score = score_search_salary(Some(&expectation()), &eur_job);

        assert_eq!(score.score_delta, 10);
        assert_eq!(score.label.as_deref(), Some("Salary fits target"));
    }
}
