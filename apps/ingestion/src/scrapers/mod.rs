pub mod djinni;
pub mod dou_ua;
mod headers;
pub mod robota_ua;
pub mod work_ua;

pub(crate) mod company;
pub(crate) mod detail_merge;
pub(crate) mod http;
pub(crate) mod remote;
pub(crate) mod runner;
pub(crate) mod salary;
pub(crate) mod seniority;
pub(crate) mod skills;
pub(crate) mod text;

pub use company::{compute_job_quality_score, infer_company_meta, normalize_company_name};
pub use detail_merge::DetailSnapshot;
pub use http::{ScraperConfig, ScraperRun, detail_error_summaries, polite_delay};
pub use remote::infer_remote_type;
pub use salary::{normalize_salary_to_usd_monthly, parse_salary_range_with_usd_monthly};
pub use seniority::{infer_seniority, infer_seniority_from_title_and_description};
pub use skills::extract_skills;
pub use text::{cleanup_description_text, collect_text, normalized_non_empty};

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::models::{NormalizationResult, NormalizedJob, RawSnapshot};

    use super::company::{
        compute_job_quality_score, infer_company_industry_hint, infer_company_size_hint,
    };
    use super::detail_merge::merge_detail_into_result;
    use super::{
        DetailSnapshot, cleanup_description_text, extract_skills, infer_company_meta,
        infer_seniority, infer_seniority_from_title_and_description,
        normalize_salary_to_usd_monthly,
    };

    #[test]
    fn computes_job_quality_score_from_ingestion_signals() {
        let job = NormalizedJob {
            id: "job-1".to_string(),
            duplicate_of: None,
            title: "Senior Rust Engineer".to_string(),
            company_name: "SignalHire".to_string(),
            company_meta: None,
            location: Some("Kyiv".to_string()),
            remote_type: Some("remote".to_string()),
            seniority: Some("senior".to_string()),
            description_text: "Build reliable Rust APIs with PostgreSQL, Docker, and Kubernetes. "
                .repeat(4),
            extracted_skills: vec![
                "Rust".to_string(),
                "PostgreSQL".to_string(),
                "Docker".to_string(),
            ],
            salary_min: Some(4000),
            salary_max: Some(5500),
            salary_currency: Some("USD".to_string()),
            salary_usd_min: Some(4000),
            salary_usd_max: Some(5500),
            quality_score: None,
            posted_at: None,
            last_seen_at: "2026-04-18T10:00:00Z".to_string(),
            is_active: true,
        };

        assert_eq!(compute_job_quality_score(&job), 100);
    }

    #[test]
    fn quality_score_rejects_generic_company_and_missing_signals() {
        let job = NormalizedJob {
            id: "job-2".to_string(),
            duplicate_of: None,
            title: "Engineer".to_string(),
            company_name: "Unknown".to_string(),
            company_meta: None,
            location: None,
            remote_type: None,
            seniority: None,
            description_text: "Short snippet".to_string(),
            extracted_skills: vec!["Rust".to_string(), "Docker".to_string()],
            salary_min: None,
            salary_max: None,
            salary_currency: None,
            salary_usd_min: None,
            salary_usd_max: None,
            quality_score: None,
            posted_at: None,
            last_seen_at: "2026-04-18T10:00:00Z".to_string(),
            is_active: true,
        };

        assert_eq!(compute_job_quality_score(&job), 0);
    }

    #[test]
    fn infers_seniority_from_english_title_patterns() {
        let cases = [
            ("Junior Rust Developer", Some("junior")),
            ("Backend Jr. Engineer", Some("junior")),
            ("Entry level QA Engineer", Some("junior")),
            ("Frontend Intern", Some("junior")),
            ("Data Trainee", Some("junior")),
            ("Middle Rust Developer", Some("middle")),
            ("Mid-level Frontend Engineer", Some("middle")),
            ("Regular PHP Developer", Some("middle")),
            ("Intermediate QA Engineer", Some("middle")),
            ("Senior Backend Engineer", Some("senior")),
            ("Sr. Data Engineer", Some("senior")),
            ("Lead Software Engineer", Some("lead")),
            ("Tech Lead Rust", Some("lead")),
            ("Team Lead Backend", Some("lead")),
            ("Principal Engineer", Some("lead")),
            ("Staff Software Engineer", Some("lead")),
            ("Head of Engineering", Some("lead")),
        ];

        for (title, expected) in cases {
            assert_eq!(infer_seniority(title).as_deref(), expected, "{title}");
        }
    }

    #[test]
    fn infers_seniority_from_ukrainian_title_patterns() {
        let cases = [
            ("Початківець Python Developer", Some("junior")),
            ("Молодший QA Engineer", Some("junior")),
            ("Середній Java Developer", Some("middle")),
            ("Досвідчений Rust Developer", Some("senior")),
        ];

        for (title, expected) in cases {
            assert_eq!(infer_seniority(title).as_deref(), expected, "{title}");
        }
    }

    #[test]
    fn infers_seniority_from_year_patterns() {
        let cases = [
            (
                "Developer",
                "Requires 0+ years of commercial experience",
                Some("junior"),
            ),
            (
                "Developer",
                "Requires 1+ year of commercial experience",
                Some("junior"),
            ),
            (
                "Developer",
                "Requires 2+ years of commercial experience",
                Some("middle"),
            ),
            (
                "Developer",
                "Requires 3+ years of commercial experience",
                Some("middle"),
            ),
            (
                "Developer",
                "Requires 4+ years of commercial experience",
                Some("senior"),
            ),
            (
                "Developer",
                "Requires 6+ years of commercial experience",
                Some("senior"),
            ),
            (
                "Developer",
                "Requires 7+ years of commercial experience",
                Some("lead"),
            ),
            (
                "Developer",
                "Experience: 2-4 years with React",
                Some("middle"),
            ),
            (
                "Developer",
                "Experience: 5+ років with Rust",
                Some("senior"),
            ),
            (
                "Developer",
                "You will lead backend delivery for the product team",
                Some("senior"),
            ),
            (
                "Developer",
                "Work as a Tech Lead for a distributed engineering team",
                Some("lead"),
            ),
        ];

        for (title, description, expected) in cases {
            assert_eq!(
                infer_seniority_from_title_and_description(title, Some(description)).as_deref(),
                expected,
                "{description}"
            );
        }
    }

    #[test]
    fn title_seniority_takes_precedence_over_description() {
        assert_eq!(
            infer_seniority_from_title_and_description(
                "Junior Backend Engineer",
                Some("Requirements: 7+ years of production experience")
            )
            .as_deref(),
            Some("junior")
        );
    }

    #[test]
    fn infers_company_size_hints_from_description_patterns() {
        assert_eq!(
            infer_company_size_hint("We are a product startup building developer tooling.")
                .as_deref(),
            Some("startup")
        );
        assert_eq!(
            infer_company_size_hint("Join an enterprise platform team serving global customers.")
                .as_deref(),
            Some("enterprise")
        );
        assert_eq!(
            infer_company_size_hint("Our team has 50-200 employees across Europe.").as_deref(),
            Some("50-200 employees")
        );
    }

    #[test]
    fn infers_company_industry_hints_from_common_sectors() {
        assert_eq!(
            infer_company_industry_hint("FinTech platform for card payments").as_deref(),
            Some("fintech")
        );
        assert_eq!(
            infer_company_industry_hint("We build edtech products for online learning").as_deref(),
            Some("edtech")
        );
        assert_eq!(
            infer_company_industry_hint("E-commerce marketplace and retail tech").as_deref(),
            Some("e-commerce")
        );
        assert_eq!(
            infer_company_industry_hint("Outsourcing software development company").as_deref(),
            Some("outsourcing")
        );
    }

    #[test]
    fn builds_nullable_company_meta_with_optional_url() {
        let meta = infer_company_meta(
            "Startup in fintech hiring backend engineers",
            Some("https://example.com/company/acme"),
        )
        .expect("company meta should be inferred");

        assert_eq!(meta.size_hint.as_deref(), Some("startup"));
        assert_eq!(meta.industry_hint.as_deref(), Some("fintech"));
        assert_eq!(
            meta.url.as_deref(),
            Some("https://example.com/company/acme")
        );
        assert!(infer_company_meta("No company hints here", None).is_none());
    }

    #[test]
    fn extracts_skills_case_insensitively_without_duplicates() {
        let skills = extract_skills(
            "We use React, react.js, TypeScript, nodejs, PostgreSQL, Docker and AWS.",
        );

        assert_eq!(
            skills,
            vec![
                "React".to_string(),
                "TypeScript".to_string(),
                "Node.js".to_string(),
                "PostgreSQL".to_string(),
                "Docker".to_string(),
                "AWS".to_string(),
            ]
        );
    }

    #[test]
    fn extracts_skills_from_explicit_english_and_ukrainian_lists() {
        let skills = extract_skills(
            "Required: Python, FastAPI, Redis\nТехнології: Kubernetes, GCP, CI/CD\nВимоги: Git, Linux",
        );

        assert_eq!(
            skills,
            vec![
                "Python".to_string(),
                "Redis".to_string(),
                "Kubernetes".to_string(),
                "GCP".to_string(),
                "Git".to_string(),
                "CI/CD".to_string(),
                "FastAPI".to_string(),
                "Linux".to_string(),
            ]
        );
    }

    #[test]
    fn skill_extraction_avoids_common_substring_false_positives() {
        let skills = extract_skills("Interest in JavaScript and ongoing collaboration is useful.");

        assert!(skills.is_empty());
    }

    #[test]
    fn rejects_results_with_placeholder_company_after_detail_merge() {
        let result = NormalizationResult {
            job: NormalizedJob {
                id: "job-1".to_string(),
                duplicate_of: None,
                title: "Rust Engineer".to_string(),
                company_name: "Unknown".to_string(),
                company_meta: None,
                location: None,
                remote_type: None,
                seniority: None,
                description_text: "Short snippet".to_string(),
                extracted_skills: Vec::new(),
                salary_min: None,
                salary_max: None,
                salary_currency: None,
                salary_usd_min: None,
                salary_usd_max: None,
                quality_score: None,
                posted_at: None,
                last_seen_at: "2026-04-18T10:00:00Z".to_string(),
                is_active: true,
            },
            snapshot: RawSnapshot {
                source: "djinni".to_string(),
                source_job_id: "1".to_string(),
                source_url: "https://example.com/jobs/1".to_string(),
                raw_payload: json!({}),
                fetched_at: "2026-04-18T10:00:00Z".to_string(),
            },
        };

        let merged = merge_detail_into_result(
            result,
            DetailSnapshot {
                description_text: Some("Longer description".to_string()),
                ..DetailSnapshot::default()
            },
        );

        assert!(merged.is_none());
    }

    #[test]
    fn prefers_detail_description_when_it_has_substantially_richer_content() {
        let result = NormalizationResult {
            job: NormalizedJob {
                id: "job-2".to_string(),
                duplicate_of: None,
                title: "Front-end React Developer".to_string(),
                company_name: "SignalHire".to_string(),
                company_meta: None,
                location: None,
                remote_type: Some("remote".to_string()),
                seniority: Some("senior".to_string()),
                description_text: "React product team".to_string(),
                extracted_skills: vec!["React".to_string()],
                salary_min: None,
                salary_max: None,
                salary_currency: None,
                salary_usd_min: None,
                salary_usd_max: None,
                quality_score: None,
                posted_at: None,
                last_seen_at: "2026-04-18T10:00:00Z".to_string(),
                is_active: true,
            },
            snapshot: RawSnapshot {
                source: "djinni".to_string(),
                source_job_id: "2".to_string(),
                source_url: "https://example.com/jobs/2".to_string(),
                raw_payload: json!({}),
                fetched_at: "2026-04-18T10:00:00Z".to_string(),
            },
        };

        let merged = merge_detail_into_result(
            result,
            DetailSnapshot {
                description_text: Some(
                    "About the role\nBuild a frontend platform with React and TypeScript.\nPartner with product and design on experiments.".to_string(),
                ),
                ..DetailSnapshot::default()
            },
        )
        .expect("detail merge should succeed");

        assert!(
            merged
                .job
                .description_text
                .contains("Build a frontend platform")
        );
        assert!(
            merged
                .job
                .description_text
                .contains("Partner with product and design")
        );
    }

    #[test]
    fn cleanup_description_text_truncates_apply_blocks_and_related_jobs_noise() {
        let cleaned = cleanup_description_text(
            "Ship accessible frontend features with React and TypeScript. How to apply: send CV. Similar vacancies below.",
            "Senior Front-end React Developer",
            "SignalHire",
            &[],
        );

        assert_eq!(
            cleaned,
            "Ship accessible frontend features with React and TypeScript."
        );
    }

    #[test]
    fn normalizes_monthly_eur_salary_to_usd() {
        assert_eq!(
            normalize_salary_to_usd_monthly(Some(2000), Some(3000), Some("EUR"), "€2000-3000"),
            (Some(2200), Some(3300))
        );
    }

    #[test]
    fn normalizes_hourly_uah_salary_to_usd_monthly() {
        assert_eq!(
            normalize_salary_to_usd_monthly(Some(500), Some(700), Some("UAH"), "500-700 грн/год"),
            (Some(1920), Some(2688))
        );
    }

    #[test]
    fn normalizes_annual_usd_salary_to_monthly() {
        assert_eq!(
            normalize_salary_to_usd_monthly(
                Some(120000),
                Some(150000),
                Some("USD"),
                "$120000-$150000 per year"
            ),
            (Some(10000), Some(12500))
        );
    }
}
