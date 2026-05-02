use crate::domain::job::model::{Job, JobLifecycleStage, JobSourceVariant, JobView};
use crate::domain::role::RoleId;
use crate::domain::search::profile::{
    SearchProfile, SearchRoleCandidate, SearchSalaryExpectation, TargetRegion, WorkMode,
};
use crate::domain::source::SourceId;

use super::SearchMatchingService;

fn search_profile() -> SearchProfile {
    SearchProfile {
        primary_role: RoleId::BackendEngineer,
        primary_role_confidence: Some(94),
        target_roles: vec![RoleId::BackendEngineer, RoleId::DevopsEngineer],
        role_candidates: vec![
            SearchRoleCandidate {
                role: RoleId::BackendEngineer,
                confidence: 94,
            },
            SearchRoleCandidate {
                role: RoleId::DevopsEngineer,
                confidence: 62,
            },
        ],
        seniority: "senior".to_string(),
        target_regions: vec![TargetRegion::EuRemote],
        work_modes: vec![WorkMode::Remote],
        allowed_sources: vec![SourceId::Djinni],
        profile_skills: vec!["rust".to_string(), "postgres".to_string()],
        profile_keywords: vec!["backend".to_string(), "platform".to_string()],
        search_terms: vec![
            "rust".to_string(),
            "postgres".to_string(),
            "distributed systems".to_string(),
        ],
        exclude_terms: vec!["gambling".to_string()],
        scoring_weights: Default::default(),
        salary_expectation: None,
    }
}

fn mobile_profile() -> SearchProfile {
    SearchProfile {
        primary_role: RoleId::MobileEngineer,
        primary_role_confidence: Some(97),
        target_roles: vec![RoleId::MobileEngineer, RoleId::FrontendEngineer],
        role_candidates: vec![
            SearchRoleCandidate {
                role: RoleId::MobileEngineer,
                confidence: 97,
            },
            SearchRoleCandidate {
                role: RoleId::FrontendEngineer,
                confidence: 68,
            },
        ],
        seniority: "senior".to_string(),
        target_regions: vec![TargetRegion::EuRemote],
        work_modes: vec![WorkMode::Remote],
        allowed_sources: vec![SourceId::Djinni],
        profile_skills: vec!["react native".to_string(), "typescript".to_string()],
        profile_keywords: vec!["mobile".to_string(), "frontend".to_string()],
        search_terms: vec![
            "react native".to_string(),
            "typescript".to_string(),
            "mobile product".to_string(),
        ],
        exclude_terms: vec!["gambling".to_string()],
        scoring_weights: Default::default(),
        salary_expectation: None,
    }
}

fn frontend_profile() -> SearchProfile {
    SearchProfile {
        primary_role: RoleId::FrontendEngineer,
        primary_role_confidence: Some(96),
        target_roles: vec![RoleId::FrontendEngineer, RoleId::MobileEngineer],
        role_candidates: vec![
            SearchRoleCandidate {
                role: RoleId::FrontendEngineer,
                confidence: 96,
            },
            SearchRoleCandidate {
                role: RoleId::MobileEngineer,
                confidence: 54,
            },
        ],
        seniority: "senior".to_string(),
        target_regions: vec![TargetRegion::EuRemote],
        work_modes: vec![WorkMode::Remote],
        allowed_sources: vec![SourceId::Djinni],
        profile_skills: vec!["react".to_string(), "typescript".to_string()],
        profile_keywords: vec!["frontend".to_string(), "design system".to_string()],
        search_terms: vec![
            "frontend developer".to_string(),
            "react".to_string(),
            "typescript".to_string(),
        ],
        exclude_terms: vec!["gambling".to_string()],
        scoring_weights: Default::default(),
        salary_expectation: None,
    }
}

fn job_view(
    id: &str,
    title: &str,
    description: &str,
    remote_type: Option<&str>,
    source: &str,
) -> JobView {
    JobView {
        job: Job {
            id: id.to_string(),
            title: title.to_string(),
            company_name: "NovaLedger".to_string(),
            location: None,
            remote_type: remote_type.map(str::to_string),
            seniority: Some("senior".to_string()),
            description_text: description.to_string(),
            salary_min: None,
            salary_max: None,
            salary_currency: None,
            language: None,
            posted_at: Some("2026-04-10T09:00:00Z".to_string()),
            last_seen_at: "2026-04-14T09:00:00Z".to_string(),
            is_active: true,
        },
        first_seen_at: "2026-04-10T09:00:00Z".to_string(),
        inactivated_at: None,
        reactivated_at: None,
        lifecycle_stage: JobLifecycleStage::Active,
        primary_variant: Some(JobSourceVariant {
            source: source.to_string(),
            source_job_id: format!("{id}-source"),
            source_url: format!("https://example.com/{id}"),
            raw_payload: None,
            fetched_at: "2026-04-14T09:00:00Z".to_string(),
            last_seen_at: "2026-04-14T09:00:00Z".to_string(),
            is_active: true,
            inactivated_at: None,
        }),
    }
}

#[test]
fn matching_role_and_terms_score_higher_than_unrelated_job() {
    let service = SearchMatchingService::new();
    let profile = search_profile();

    let matching_job = job_view(
        "job-1",
        "Senior Backend Developer",
        "Remote EU role working with Rust, Postgres, and distributed systems",
        Some("remote"),
        "djinni",
    );
    let unrelated_job = job_view(
        "job-2",
        "Marketing Specialist",
        "Onsite campaign execution and social media planning",
        Some("onsite"),
        "djinni",
    );

    let matching_fit = service.score_job_deterministic(&profile, &matching_job);
    let unrelated_fit = service.score_job_deterministic(&profile, &unrelated_job);

    assert!(matching_fit.score > unrelated_fit.score);
    assert!(
        matching_fit
            .matched_roles
            .contains(&RoleId::BackendEngineer)
    );
    assert!(matching_fit.matched_skills.contains(&"rust".to_string()));
}

#[test]
fn source_mismatch_is_filtered_out_when_allowed_sources_are_set() {
    let service = SearchMatchingService::new();
    let profile = search_profile();

    let results = service.run(
        &profile,
        vec![
            job_view(
                "job-1",
                "Senior Backend Developer",
                "Remote EU role with Rust",
                Some("remote"),
                "djinni",
            ),
            job_view(
                "job-2",
                "Senior Backend Developer",
                "Remote EU role with Rust",
                Some("remote"),
                "work_ua",
            ),
        ],
    );

    assert_eq!(results.filtered_out_by_source, 1);
    assert_eq!(results.ranked_jobs.len(), 1);
    assert_eq!(results.ranked_jobs[0].job.job.id, "job-1");
}

#[test]
fn empty_allowed_sources_keeps_all_sources_eligible() {
    let service = SearchMatchingService::new();
    let mut profile = search_profile();
    profile.allowed_sources.clear();

    let results = service.run(
        &profile,
        vec![
            job_view(
                "job-1",
                "Senior Backend Developer",
                "Remote EU role with Rust",
                Some("remote"),
                "djinni",
            ),
            job_view(
                "job-2",
                "Senior Backend Developer",
                "Remote EU role with Rust",
                Some("remote"),
                "work_ua",
            ),
        ],
    );

    assert_eq!(results.filtered_out_by_source, 0);
    assert_eq!(results.ranked_jobs.len(), 2);
    assert!(results.ranked_jobs.iter().all(|job| job.fit.source_match));
}

#[test]
fn search_terms_contribute_to_score() {
    let service = SearchMatchingService::new();
    let profile = search_profile();

    let matching_terms_job = job_view(
        "job-1",
        "Senior Backend Developer",
        "Remote EU role with Rust, Postgres, and distributed systems",
        Some("remote"),
        "djinni",
    );
    let missing_terms_job = job_view(
        "job-2",
        "Senior Backend Developer",
        "Remote EU role with Scala and Cassandra",
        Some("remote"),
        "djinni",
    );

    let matching_terms_fit = service.score_job_deterministic(&profile, &matching_terms_job);
    let missing_terms_fit = service.score_job_deterministic(&profile, &missing_terms_job);

    assert!(matching_terms_fit.score > missing_terms_fit.score);
    assert!(
        !matching_terms_fit.matched_keywords.is_empty()
            || !matching_terms_fit.matched_skills.is_empty()
    );
}

#[test]
fn seniority_mismatch_lowers_score() {
    let service = SearchMatchingService::new();
    let profile = search_profile();
    let matching_job = job_view(
        "job-1",
        "Senior Backend Developer",
        "Remote EU role with Rust and Postgres",
        Some("remote"),
        "djinni",
    );
    let junior_job = JobView {
        job: Job {
            seniority: Some("junior".to_string()),
            ..matching_job.job.clone()
        },
        ..matching_job.clone()
    };

    let matching_fit = service.score_job_deterministic(&profile, &matching_job);
    let junior_fit = service.score_job_deterministic(&profile, &junior_job);

    assert!(matching_fit.score > junior_fit.score);
    assert!(
        junior_fit
            .reasons
            .iter()
            .any(|reason| reason.contains("Seniority mismatch penalty applied"))
    );
}

#[test]
fn role_family_overlap_gives_partial_credit() {
    let service = SearchMatchingService::new();
    let profile = mobile_profile();
    let exact_job = job_view(
        "job-1",
        "Senior React Native Developer",
        "Remote EU role building React Native apps with TypeScript",
        Some("remote"),
        "djinni",
    );
    let partial_job = job_view(
        "job-2",
        "Senior Fullstack Developer",
        "Remote EU product role with React, TypeScript, and backend APIs",
        Some("remote"),
        "djinni",
    );
    let unrelated_job = job_view(
        "job-3",
        "Senior Backend Developer",
        "Remote EU role with Rust and distributed systems",
        Some("remote"),
        "djinni",
    );

    let exact_fit = service.score_job_deterministic(&profile, &exact_job);
    let partial_fit = service.score_job_deterministic(&profile, &partial_job);
    let unrelated_fit = service.score_job_deterministic(&profile, &unrelated_job);

    assert!(exact_fit.score > partial_fit.score);
    assert!(partial_fit.score > unrelated_fit.score);
    assert!(
        partial_fit
            .reasons
            .iter()
            .any(|reason| reason.contains("Role family overlap"))
    );
}

#[test]
fn exclude_terms_lower_score() {
    let service = SearchMatchingService::new();
    let profile = search_profile();
    let clean_job = job_view(
        "job-1",
        "Senior Backend Developer",
        "Remote EU role with Rust and Postgres for a product platform",
        Some("remote"),
        "djinni",
    );
    let excluded_job = job_view(
        "job-2",
        "Senior Backend Developer",
        "Remote EU role with Rust and Postgres for a gambling platform",
        Some("remote"),
        "djinni",
    );

    let clean_fit = service.score_job_deterministic(&profile, &clean_job);
    let excluded_fit = service.score_job_deterministic(&profile, &excluded_job);

    assert!(clean_fit.score > excluded_fit.score);
    assert!(
        excluded_fit
            .reasons
            .iter()
            .any(|reason| reason.contains("Exclude term penalty applied"))
    );
}

#[test]
fn explanations_include_positive_and_negative_reasons() {
    let service = SearchMatchingService::new();
    let profile = search_profile();
    let job = JobView {
        job: Job {
            seniority: Some("junior".to_string()),
            ..job_view(
                "job-1",
                "Backend Platform Engineer",
                "Hybrid EU role with Rust and Postgres for a gambling platform",
                Some("hybrid"),
                "djinni",
            )
            .job
        },
        ..job_view(
            "job-1",
            "Backend Platform Engineer",
            "Hybrid EU role with Rust and Postgres for a gambling platform",
            Some("hybrid"),
            "djinni",
        )
    };

    let fit = service.score_job_deterministic(&profile, &job);

    assert!(
        fit.reasons
            .iter()
            .any(|reason| reason.contains("Matched profile skills"))
    );
    assert!(
        fit.reasons
            .iter()
            .any(|reason| reason.contains("Exclude term penalty applied"))
    );
    assert!(
        fit.reasons
            .iter()
            .any(|reason| reason.contains("Work mode mismatch penalty applied"))
    );
    assert!(
        fit.reasons
            .iter()
            .any(|reason| reason.contains("Seniority mismatch penalty applied"))
    );
}

#[test]
fn profile_aligned_jobs_rank_above_weakly_related_jobs() {
    let service = SearchMatchingService::new();
    let profile = search_profile();

    let exact_backend = job_view(
        "job-1",
        "Senior Backend Developer",
        "Remote EU role with Rust, Postgres, and backend platform work",
        Some("remote"),
        "djinni",
    );
    let partial_devops = job_view(
        "job-2",
        "Senior DevOps Engineer",
        "Remote EU platform role with AWS, docker, kubernetes, and backend API ownership",
        Some("remote"),
        "djinni",
    );
    let weak_match = job_view(
        "job-3",
        "Senior Product Manager",
        "Remote EU product strategy role with roadmap ownership",
        Some("remote"),
        "djinni",
    );

    let result = service.run(&profile, vec![weak_match, partial_devops, exact_backend]);

    assert_eq!(result.ranked_jobs[0].job.job.id, "job-1");
    assert_eq!(result.ranked_jobs[1].job.job.id, "job-2");
    assert_eq!(result.ranked_jobs[2].job.job.id, "job-3");
}

#[test]
fn canonical_frontend_terms_survive_matching_and_explanations() {
    let service = SearchMatchingService::new();
    let profile = frontend_profile();
    let job = job_view(
        "job-frontend-1",
        "Senior Front-end React Developer",
        "Remote EU role shipping frontend design system work with React and TypeScript",
        Some("remote"),
        "djinni",
    );

    let fit = service.score_job_deterministic(&profile, &job);

    assert!(fit.score >= 70);
    assert!(fit.matched_roles.contains(&RoleId::FrontendEngineer));
    assert!(fit.matched_skills.contains(&"react".to_string()));
    assert!(fit.matched_keywords.contains(&"frontend".to_string()));
    assert!(
        !fit.matched_keywords
            .iter()
            .any(|term| term == "front" || term == "end")
    );
    assert!(
        fit.reasons
            .iter()
            .any(|reason| reason.contains("Matched profile keywords: frontend"))
    );
}

#[test]
fn react_native_matching_keeps_phrase_safe_internal_tokens_internal() {
    let service = SearchMatchingService::new();
    let profile = mobile_profile();
    let base = job_view(
        "job-mobile-1",
        "Senior React-Native Developer",
        "Remote EU role building React Native apps with TypeScript and distributed systems work",
        Some("remote"),
        "djinni",
    );
    // Pin to a far-future date so freshness decay is never applied, keeping the
    // test stable regardless of when it runs.
    let job = JobView {
        job: Job {
            posted_at: Some("2099-01-01T00:00:00Z".to_string()),
            last_seen_at: "2099-01-01T00:00:00Z".to_string(),
            ..base.job.clone()
        },
        first_seen_at: "2099-01-01T00:00:00Z".to_string(),
        ..base
    };

    let fit = service.score_job_deterministic(&profile, &job);

    assert!(fit.score >= 70);
    assert!(fit.matched_roles.contains(&RoleId::MobileEngineer));
    assert!(fit.matched_skills.contains(&"react native".to_string()));
    assert!(!fit.matched_skills.iter().any(|term| term == "react_native"));
    assert!(
        fit.reasons
            .iter()
            .any(|reason| reason.contains("Matched profile skills: react native, typescript"))
    );
}

#[test]
fn frontend_react_overlap_beats_generic_engineering_overlap() {
    let service = SearchMatchingService::new();
    let profile = frontend_profile();
    let strong_match = job_view(
        "job-frontend-strong",
        "Senior Front-end React Developer",
        "Remote EU role shipping frontend design system work with React and TypeScript",
        Some("remote"),
        "djinni",
    );
    let weak_match = job_view(
        "job-frontend-weak",
        "Senior UI Engineer",
        "Remote EU role improving shared product experiences and collaborating with design",
        Some("remote"),
        "djinni",
    );

    let strong_fit = service.score_job_deterministic(&profile, &strong_match);
    let weak_fit = service.score_job_deterministic(&profile, &weak_match);

    assert!(strong_fit.score > weak_fit.score);
    assert!(
        strong_fit
            .matched_keywords
            .contains(&"frontend".to_string())
    );
    assert!(strong_fit.matched_skills.contains(&"react".to_string()));
}

#[test]
fn non_contiguous_frontend_search_phrase_matches_canonical_frontend_term() {
    let service = SearchMatchingService::new();
    let profile = SearchProfile {
        primary_role: RoleId::FrontendEngineer,
        primary_role_confidence: Some(96),
        target_roles: vec![RoleId::FrontendEngineer],
        role_candidates: vec![SearchRoleCandidate {
            role: RoleId::FrontendEngineer,
            confidence: 96,
        }],
        seniority: "senior".to_string(),
        target_regions: vec![TargetRegion::EuRemote],
        work_modes: vec![WorkMode::Remote],
        allowed_sources: vec![SourceId::Djinni],
        profile_skills: vec!["react".to_string()],
        profile_keywords: vec!["design system".to_string()],
        search_terms: vec!["frontend specialist".to_string()],
        exclude_terms: vec![],
        scoring_weights: Default::default(),
        salary_expectation: None,
    };
    let job = job_view(
        "job-frontend-search-term",
        "Senior Front-end React Developer",
        "Remote EU role shipping frontend design system work with React",
        Some("remote"),
        "djinni",
    );

    let fit = service.score_job_deterministic(&profile, &job);

    assert!(fit.score >= 70);
    assert!(
        fit.reasons
            .iter()
            .any(|reason| reason.contains("Matched search terms: frontend"))
    );
}

#[test]
fn backend_platform_overlap_prefers_engineering_stack_signals() {
    let service = SearchMatchingService::new();
    let profile = search_profile();
    let platform_job = job_view(
        "job-platform-1",
        "Senior Platform Engineer",
        "Remote EU platform role owning Rust APIs, Postgres, GraphQL, and distributed systems",
        Some("remote"),
        "djinni",
    );
    let generic_job = job_view(
        "job-generic-1",
        "Senior Software Engineer",
        "Remote EU role collaborating across product teams and improving internal tools",
        Some("remote"),
        "djinni",
    );

    let platform_fit = service.score_job_deterministic(&profile, &platform_job);
    let generic_fit = service.score_job_deterministic(&profile, &generic_job);

    assert!(platform_fit.score > generic_fit.score);
    assert!(platform_fit.matched_skills.contains(&"rust".to_string()));
    assert!(
        platform_fit
            .matched_skills
            .contains(&"postgres".to_string())
    );
    assert!(
        platform_fit
            .matched_keywords
            .contains(&"distributed systems".to_string())
    );
}

#[test]
fn stale_job_scores_lower_than_fresh_identical_job() {
    let service = SearchMatchingService::new();
    let profile = search_profile();
    // Pin fresh job to a far-future date so it never receives an age penalty
    // regardless of when the test runs.
    let base = job_view(
        "job-fresh",
        "Senior Backend Developer",
        "Remote EU role with Rust and Postgres",
        Some("remote"),
        "djinni",
    );
    let fresh_job = JobView {
        job: Job {
            posted_at: Some("2099-01-01T00:00:00Z".to_string()),
            last_seen_at: "2099-01-01T00:00:00Z".to_string(),
            ..base.job.clone()
        },
        first_seen_at: "2099-01-01T00:00:00Z".to_string(),
        ..base.clone()
    };
    // Stale job: all date fields set to a distant past date.
    let stale_job = JobView {
        job: Job {
            posted_at: Some("2020-01-01T00:00:00Z".to_string()),
            last_seen_at: "2020-01-01T00:00:00Z".to_string(),
            ..base.job.clone()
        },
        first_seen_at: "2020-01-01T00:00:00Z".to_string(),
        ..base.clone()
    };

    let fresh_fit = service.score_job_deterministic(&profile, &fresh_job);
    let stale_fit = service.score_job_deterministic(&profile, &stale_job);

    assert!(
        fresh_fit.score > stale_fit.score,
        "fresh score {} should beat stale score {}",
        fresh_fit.score,
        stale_fit.score,
    );
    assert!(
        stale_fit
            .reasons
            .iter()
            .any(|r| r.contains("Job age penalty applied")),
        "stale job reasons should contain job age penalty explanation"
    );
}

#[test]
fn required_skill_gap_penalizes_more_than_preferred_skill_gap() {
    let service = SearchMatchingService::new();

    let profile = SearchProfile {
        primary_role: RoleId::BackendEngineer,
        primary_role_confidence: Some(90),
        target_roles: vec![RoleId::BackendEngineer],
        role_candidates: vec![SearchRoleCandidate {
            role: RoleId::BackendEngineer,
            confidence: 90,
        }],
        seniority: "senior".to_string(),
        target_regions: vec![TargetRegion::EuRemote],
        work_modes: vec![WorkMode::Remote],
        allowed_sources: vec![SourceId::Djinni],
        profile_skills: vec!["rust".to_string(), "graphql".to_string()],
        profile_keywords: vec![],
        search_terms: vec![],
        exclude_terms: vec![],
        scoring_weights: Default::default(),
        salary_expectation: None,
    };

    let required_skill_match = job_view(
        "job-required-match",
        "Senior Backend Engineer",
        "Requirements:\n- Rust\nPreferred:\n- GraphQL",
        Some("remote"),
        "djinni",
    );

    let preferred_skill_only_match = job_view(
        "job-preferred-only",
        "Senior Backend Engineer",
        "Requirements:\n- Kubernetes\nPreferred:\n- GraphQL",
        Some("remote"),
        "djinni",
    );

    let required_fit = service.score_job_deterministic(&profile, &required_skill_match);
    let preferred_only_fit = service.score_job_deterministic(&profile, &preferred_skill_only_match);

    assert!(
        required_fit.score > preferred_only_fit.score,
        "matching a must-have skill should score higher than only matching a nice-to-have skill: required={}, preferred_only={}",
        required_fit.score,
        preferred_only_fit.score,
    );

    assert!(required_fit.matched_skills.contains(&"rust".to_string()));
    assert!(
        preferred_only_fit
            .matched_skills
            .contains(&"graphql".to_string())
    );
}

#[test]
fn missing_signals_stay_specific_and_drop_generic_noise() {
    let service = SearchMatchingService::new();
    let profile = frontend_profile();
    let weak_job = job_view(
        "job-frontend-gap",
        "Senior UI Engineer",
        "Remote EU role improving shared product experiences with design collaboration",
        Some("remote"),
        "djinni",
    );

    let fit = service.score_job_deterministic(&profile, &weak_job);

    assert!(fit.missing_signals.contains(&"react".to_string()));
    assert!(fit.missing_signals.contains(&"typescript".to_string()));
    assert!(fit.missing_signals.contains(&"design system".to_string()));
    assert!(!fit.missing_signals.iter().any(|term| term == "developer"));
    assert!(!fit.missing_signals.iter().any(|term| term == "engineer"));
}

#[test]
fn fresh_job_has_no_freshness_penalty() {
    let service = SearchMatchingService::new();
    let profile = search_profile();
    let base = job_view(
        "job-fresh-check",
        "Senior Backend Developer",
        "Remote EU role with Rust and Postgres",
        Some("remote"),
        "djinni",
    );
    // Far-future date: always 0 days old regardless of when the test runs.
    let fresh_job = JobView {
        job: Job {
            posted_at: Some("2099-01-01T00:00:00Z".to_string()),
            last_seen_at: "2099-01-01T00:00:00Z".to_string(),
            ..base.job.clone()
        },
        first_seen_at: "2099-01-01T00:00:00Z".to_string(),
        ..base
    };

    let fit = service.score_job_deterministic(&profile, &fresh_job);

    assert_eq!(
        fit.score_breakdown.freshness_score, 0,
        "job within 14-day grace period must have zero freshness score"
    );
    assert!(
        !fit.reasons
            .iter()
            .any(|r| r.contains("Job age penalty applied")),
        "fresh job must not include an age penalty reason"
    );
}

#[test]
fn old_job_has_negative_freshness_score_and_reason() {
    let service = SearchMatchingService::new();
    let profile = search_profile();
    let base = job_view(
        "job-old-check",
        "Senior Backend Developer",
        "Remote EU role with Rust and Postgres",
        Some("remote"),
        "djinni",
    );
    // Ancient date: always well past the 30-day maximum-penalty threshold.
    let old_job = JobView {
        job: Job {
            posted_at: Some("2020-01-01T00:00:00Z".to_string()),
            last_seen_at: "2020-01-01T00:00:00Z".to_string(),
            ..base.job.clone()
        },
        first_seen_at: "2020-01-01T00:00:00Z".to_string(),
        ..base
    };

    let fit = service.score_job_deterministic(&profile, &old_job);

    assert!(
        fit.score_breakdown.freshness_score < 0,
        "job older than 14 days must have a negative freshness_score, got {}",
        fit.score_breakdown.freshness_score
    );
    assert!(
        fit.reasons
            .iter()
            .any(|r| r.contains("Job age penalty applied")),
        "old job must include an age penalty reason in the explanation"
    );
}

#[test]
fn work_mode_no_preference_does_not_affect_score() {
    let service = SearchMatchingService::new();
    let mut profile = search_profile();
    profile.work_modes = vec![];
    profile.target_regions = vec![];

    let remote_job = job_view(
        "job-remote",
        "Senior Backend Developer",
        "Rust and Postgres platform role",
        Some("remote"),
        "djinni",
    );
    let onsite_job = job_view(
        "job-onsite",
        "Senior Backend Developer",
        "Rust and Postgres platform role",
        Some("onsite"),
        "djinni",
    );

    let remote_fit = service.score_job_deterministic(&profile, &remote_job);
    let onsite_fit = service.score_job_deterministic(&profile, &onsite_job);

    assert_eq!(
        remote_fit.score, onsite_fit.score,
        "no work mode preference must not affect score: remote={}, onsite={}",
        remote_fit.score, onsite_fit.score
    );
    assert!(remote_fit.work_mode_match.is_none());
    assert!(onsite_fit.work_mode_match.is_none());
}

#[test]
fn work_mode_match_applies_positive_score_effect() {
    let service = SearchMatchingService::new();
    let mut profile = search_profile();
    profile.target_regions = vec![];

    let remote_job = job_view(
        "job-remote",
        "Senior Backend Developer",
        "Rust and Postgres platform role",
        Some("remote"),
        "djinni",
    );
    let unknown_mode_job = job_view(
        "job-unknown",
        "Senior Backend Developer",
        "Rust and Postgres platform role",
        None,
        "djinni",
    );

    let remote_fit = service.score_job_deterministic(&profile, &remote_job);
    let unknown_fit = service.score_job_deterministic(&profile, &unknown_mode_job);

    assert!(
        remote_fit.score > unknown_fit.score,
        "matching work mode must score higher than unknown work mode: remote={}, unknown={}",
        remote_fit.score,
        unknown_fit.score,
    );
    assert_eq!(remote_fit.work_mode_match, Some(true));
    assert!(
        remote_fit
            .reasons
            .iter()
            .any(|r| r.contains("Work mode matched")),
        "matching work mode must produce a positive reason"
    );
}

#[test]
fn work_mode_mismatch_applies_negative_score_effect() {
    let service = SearchMatchingService::new();
    let mut profile = search_profile();
    profile.target_regions = vec![];

    let onsite_job = job_view(
        "job-onsite",
        "Senior Backend Developer",
        "Rust and Postgres platform role",
        Some("onsite"),
        "djinni",
    );
    let unknown_mode_job = job_view(
        "job-unknown",
        "Senior Backend Developer",
        "Rust and Postgres platform role",
        None,
        "djinni",
    );

    let onsite_fit = service.score_job_deterministic(&profile, &onsite_job);
    let unknown_fit = service.score_job_deterministic(&profile, &unknown_mode_job);

    assert!(
        onsite_fit.score < unknown_fit.score,
        "mismatching work mode must score lower than unknown work mode: onsite={}, unknown={}",
        onsite_fit.score,
        unknown_fit.score,
    );
    assert_eq!(onsite_fit.work_mode_match, Some(false));
    assert!(
        onsite_fit
            .reasons
            .iter()
            .any(|r| r.contains("Work mode mismatch penalty applied")),
        "mismatching work mode must produce a penalty reason"
    );
}

#[test]
fn unknown_job_work_mode_does_not_penalize() {
    let service = SearchMatchingService::new();
    let profile = search_profile();

    let unknown_job = job_view(
        "job-unknown-mode",
        "Senior Backend Developer",
        "Rust and Postgres platform role",
        None,
        "djinni",
    );

    let fit = service.score_job_deterministic(&profile, &unknown_job);

    assert!(
        fit.work_mode_match.is_none(),
        "unknown work mode must return None match"
    );
    assert!(
        !fit.score_breakdown
            .penalties
            .iter()
            .any(|p| p.kind == "work_mode_mismatch"),
        "unknown work mode must not produce a mismatch penalty entry"
    );
}

#[test]
fn work_mode_score_stays_within_zero_to_one_hundred() {
    let service = SearchMatchingService::new();
    let mut profile = search_profile();
    profile.work_modes = vec![WorkMode::Remote];
    profile.target_regions = vec![];
    profile.allowed_sources = vec![];

    let mismatch_job = job_view(
        "job-mismatch-clamp",
        "Pastry Chef",
        "Onsite kitchen work preparing desserts",
        Some("onsite"),
        "djinni",
    );

    let fit = service.score_job_deterministic(&profile, &mismatch_job);

    assert_eq!(
        fit.score, fit.score_breakdown.total_score,
        "fit.score and score_breakdown.total_score must stay in sync"
    );
    assert!(fit.score <= 100, "score must not exceed 100");
}

#[test]
fn work_mode_match_and_mismatch_produce_explanation_reasons() {
    let service = SearchMatchingService::new();
    let mut profile = search_profile();
    profile.target_regions = vec![];

    let matching_job = job_view(
        "job-mode-match",
        "Senior Backend Developer",
        "Rust and Postgres platform role",
        Some("remote"),
        "djinni",
    );
    let mismatch_job = job_view(
        "job-mode-mismatch",
        "Senior Backend Developer",
        "Rust and Postgres platform role",
        Some("onsite"),
        "djinni",
    );

    let match_fit = service.score_job_deterministic(&profile, &matching_job);
    let mismatch_fit = service.score_job_deterministic(&profile, &mismatch_job);

    assert!(
        match_fit
            .reasons
            .iter()
            .any(|r| r.contains("Work mode matched")),
        "matching work mode must include a positive explanation reason"
    );
    assert!(
        mismatch_fit
            .reasons
            .iter()
            .any(|r| r.contains("Work mode mismatch penalty applied")),
        "mismatching work mode must include a penalty explanation reason"
    );
}

#[test]
fn region_no_preference_does_not_affect_score() {
    let service = SearchMatchingService::new();
    let mut profile = search_profile();
    profile.target_regions = vec![];
    profile.work_modes = vec![];

    let eu_job = job_view(
        "job-eu",
        "Senior Backend Developer",
        "Remote Europe role with Rust and Postgres",
        Some("remote"),
        "djinni",
    );
    let us_job = job_view(
        "job-us",
        "Senior Backend Developer",
        "USA-based role with Rust and Postgres",
        Some("remote"),
        "djinni",
    );

    let eu_fit = service.score_job_deterministic(&profile, &eu_job);
    let us_fit = service.score_job_deterministic(&profile, &us_job);

    assert_eq!(
        eu_fit.score, us_fit.score,
        "no region preference must not affect score: eu={}, us={}",
        eu_fit.score, us_fit.score,
    );
    assert!(eu_fit.region_match.is_none());
    assert!(us_fit.region_match.is_none());
}

#[test]
fn region_match_applies_positive_score_effect() {
    let service = SearchMatchingService::new();
    let mut profile = search_profile();
    profile.target_regions = vec![TargetRegion::EuRemote];
    profile.work_modes = vec![];

    let eu_remote_job = job_view(
        "job-eu-remote",
        "Senior Backend Developer",
        "Remote Europe role with Rust and Postgres",
        Some("remote"),
        "djinni",
    );
    let no_region_job = job_view(
        "job-no-region",
        "Senior Backend Developer",
        "Rust and Postgres platform role",
        Some("remote"),
        "djinni",
    );

    let eu_fit = service.score_job_deterministic(&profile, &eu_remote_job);
    let no_region_fit = service.score_job_deterministic(&profile, &no_region_job);

    assert!(
        eu_fit.score > no_region_fit.score,
        "matching region must score higher than unknown region: eu={}, no_region={}",
        eu_fit.score,
        no_region_fit.score,
    );
    assert_eq!(eu_fit.region_match, Some(true));
    assert!(
        eu_fit
            .reasons
            .iter()
            .any(|r| r.contains("Target region matched")),
        "matching region must produce a positive reason"
    );
}

#[test]
fn region_mismatch_applies_negative_score_effect() {
    let service = SearchMatchingService::new();
    let mut profile = search_profile();
    profile.target_regions = vec![TargetRegion::Ua];
    profile.work_modes = vec![];

    let us_job = job_view(
        "job-us",
        "Senior Backend Developer",
        "USA-based role with Rust and Postgres",
        Some("remote"),
        "djinni",
    );
    let no_region_job = job_view(
        "job-no-region",
        "Senior Backend Developer",
        "Rust and Postgres platform role",
        Some("remote"),
        "djinni",
    );

    let us_fit = service.score_job_deterministic(&profile, &us_job);
    let no_region_fit = service.score_job_deterministic(&profile, &no_region_job);

    assert!(
        us_fit.score < no_region_fit.score,
        "mismatching region must score lower than unknown region: us={}, no_region={}",
        us_fit.score,
        no_region_fit.score,
    );
    assert_eq!(us_fit.region_match, Some(false));
    assert!(
        us_fit
            .reasons
            .iter()
            .any(|r| r.contains("Region mismatch penalty applied")),
        "mismatching region must produce a penalty reason"
    );
}

#[test]
fn unknown_job_region_does_not_penalize() {
    let service = SearchMatchingService::new();
    let mut profile = search_profile();
    profile.target_regions = vec![TargetRegion::Ua];
    profile.work_modes = vec![];

    let no_region_job = job_view(
        "job-no-region",
        "Senior Backend Developer",
        "Rust and Postgres platform role with strong architecture ownership",
        Some("remote"),
        "djinni",
    );

    let fit = service.score_job_deterministic(&profile, &no_region_job);

    assert!(
        fit.region_match.is_none(),
        "unknown job region must return None match, got {:?}",
        fit.region_match,
    );
    assert!(
        !fit.score_breakdown
            .penalties
            .iter()
            .any(|p| p.kind == "region_mismatch"),
        "unknown job region must not produce a mismatch penalty entry"
    );
    assert!(
        !fit.reasons
            .iter()
            .any(|r| r.contains("Region mismatch penalty applied")),
        "unknown job region must not produce a mismatch reason"
    );
}

#[test]
fn remote_only_job_without_eu_keywords_does_not_produce_eu_remote_region() {
    // `remote_type = "remote"` enables the is_remote flag inside detect_job_regions,
    // but EuRemote detection also requires at least one EU-area keyword in the text.
    // A plain remote job with no EU signals must return None, not Some(false).
    let service = SearchMatchingService::new();
    let mut profile = search_profile();
    profile.target_regions = vec![TargetRegion::EuRemote];
    profile.work_modes = vec![];

    let remote_no_eu_job = job_view(
        "job-remote-no-eu",
        "Senior Backend Developer",
        "Fully remote role with Rust and Postgres, open to candidates worldwide",
        Some("remote"),
        "djinni",
    );

    let fit = service.score_job_deterministic(&profile, &remote_no_eu_job);

    assert!(
        fit.region_match.is_none(),
        "remote job without EU keywords must return None, not Some(false): got {:?}",
        fit.region_match,
    );
    assert!(
        !fit.score_breakdown
            .penalties
            .iter()
            .any(|p| p.kind == "region_mismatch"),
        "remote job without EU keywords must not produce a region_mismatch penalty"
    );
}

#[test]
fn region_score_stays_within_zero_to_one_hundred() {
    let service = SearchMatchingService::new();
    let mut profile = search_profile();
    profile.target_regions = vec![TargetRegion::Ua];
    profile.work_modes = vec![WorkMode::Remote];
    profile.allowed_sources = vec![];

    let mismatch_job = job_view(
        "job-region-clamp",
        "Pastry Chef",
        "USA-based role preparing desserts",
        Some("onsite"),
        "djinni",
    );

    let fit = service.score_job_deterministic(&profile, &mismatch_job);

    assert_eq!(
        fit.score, fit.score_breakdown.total_score,
        "fit.score and score_breakdown.total_score must stay in sync"
    );
    assert!(fit.score <= 100, "score must not exceed 100");
}

#[test]
fn region_match_and_mismatch_produce_explanation_reasons() {
    let service = SearchMatchingService::new();
    let mut profile = search_profile();
    profile.target_regions = vec![TargetRegion::EuRemote];
    profile.work_modes = vec![];

    let matching_job = job_view(
        "job-region-match",
        "Senior Backend Developer",
        "Remote Europe role with Rust and Postgres",
        Some("remote"),
        "djinni",
    );
    let mismatch_job = job_view(
        "job-region-mismatch",
        "Senior Backend Developer",
        "USA-based Rust and Postgres role",
        Some("remote"),
        "djinni",
    );

    let match_fit = service.score_job_deterministic(&profile, &matching_job);
    let mismatch_fit = service.score_job_deterministic(&profile, &mismatch_job);

    assert!(
        match_fit
            .reasons
            .iter()
            .any(|r| r.contains("Target region matched")),
        "matching region must include a positive explanation reason"
    );
    assert!(
        mismatch_fit
            .reasons
            .iter()
            .any(|r| r.contains("Region mismatch penalty applied")),
        "mismatching region must include a penalty explanation reason"
    );
}

#[test]
fn score_never_goes_below_zero_for_irrelevant_old_job() {
    let service = SearchMatchingService::new();
    let profile = search_profile();
    let base = job_view(
        "job-irrelevant-old",
        "Pastry Chef",
        "Onsite confectionery role preparing desserts and pastries",
        Some("onsite"),
        "djinni",
    );
    // Completely irrelevant job content + ancient date = worst-case score without clamping.
    let job = JobView {
        job: Job {
            posted_at: Some("2020-01-01T00:00:00Z".to_string()),
            last_seen_at: "2020-01-01T00:00:00Z".to_string(),
            ..base.job.clone()
        },
        first_seen_at: "2020-01-01T00:00:00Z".to_string(),
        ..base
    };

    let fit = service.score_job_deterministic(&profile, &job);

    // u8 can never be negative, but this guards against silent wrap-around
    // if the clamp inside refresh_total were ever removed.
    assert_eq!(
        fit.score, fit.score_breakdown.total_score,
        "fit.score and score_breakdown.total_score must stay in sync"
    );
    assert!(
        fit.score_breakdown.freshness_score < 0,
        "old irrelevant job must carry a freshness penalty"
    );
}

#[test]
fn salary_no_expectation_does_not_affect_score() {
    let service = SearchMatchingService::new();
    let profile = search_profile(); // salary_expectation: None

    let base = job_view(
        "job-salary-no-expectation",
        "Senior Backend Developer",
        "Remote EU role with Rust and Postgres",
        Some("remote"),
        "djinni",
    );
    let job_with_salary = JobView {
        job: Job {
            salary_min: Some(5000),
            salary_max: Some(8000),
            salary_currency: Some("USD".to_string()),
            ..base.job
        },
        ..base
    };

    let fit = service.score_job_deterministic(&profile, &job_with_salary);

    assert_eq!(
        fit.score_breakdown.salary_score, 0,
        "no salary expectation must not affect salary score"
    );
    assert!(
        !fit.reasons
            .iter()
            .any(|r| r.to_lowercase().contains("salary")),
        "no salary expectation must not produce a salary reason"
    );
}

#[test]
fn salary_missing_job_salary_does_not_penalize() {
    let service = SearchMatchingService::new();
    let mut profile = search_profile();
    profile.salary_expectation = Some(SearchSalaryExpectation {
        min: Some(4000),
        max: Some(7000),
        currency: "USD".to_string(),
    });

    // job_view has salary_min: None, salary_max: None by default
    let no_salary_job = job_view(
        "job-no-salary",
        "Senior Backend Developer",
        "Remote EU role with Rust and Postgres",
        Some("remote"),
        "djinni",
    );

    let fit = service.score_job_deterministic(&profile, &no_salary_job);

    assert_eq!(
        fit.score_breakdown.salary_score, 0,
        "missing job salary must not affect salary score"
    );
    assert!(
        !fit.reasons
            .iter()
            .any(|r| r.to_lowercase().contains("below")),
        "missing job salary must not produce a below-target reason"
    );
}

#[test]
fn salary_overlap_applies_positive_score_effect() {
    let service = SearchMatchingService::new();
    let base = job_view(
        "job-salary-overlap",
        "Senior Backend Developer",
        "Remote EU role with Rust and Postgres",
        Some("remote"),
        "djinni",
    );
    // Pin to a future date so freshness decay does not interfere.
    let overlapping_salary_job = JobView {
        job: Job {
            salary_min: Some(5000),
            salary_max: Some(8000),
            salary_currency: Some("USD".to_string()),
            posted_at: Some("2099-01-01T00:00:00Z".to_string()),
            last_seen_at: "2099-01-01T00:00:00Z".to_string(),
            ..base.job.clone()
        },
        first_seen_at: "2099-01-01T00:00:00Z".to_string(),
        ..base.clone()
    };
    let no_salary_job = JobView {
        job: Job {
            posted_at: Some("2099-01-01T00:00:00Z".to_string()),
            last_seen_at: "2099-01-01T00:00:00Z".to_string(),
            ..base.job
        },
        first_seen_at: "2099-01-01T00:00:00Z".to_string(),
        ..base
    };

    let mut profile_with_salary = search_profile();
    profile_with_salary.salary_expectation = Some(SearchSalaryExpectation {
        min: Some(4000),
        max: Some(7000),
        currency: "USD".to_string(),
    });
    let profile_without_salary = search_profile();

    let with_salary_fit =
        service.score_job_deterministic(&profile_with_salary, &overlapping_salary_job);
    let without_salary_fit =
        service.score_job_deterministic(&profile_without_salary, &no_salary_job);

    assert!(
        with_salary_fit.score > without_salary_fit.score,
        "overlapping salary must give higher score than no salary expectation: with={}, without={}",
        with_salary_fit.score,
        without_salary_fit.score,
    );
    assert!(
        with_salary_fit.score_breakdown.salary_score > 0,
        "overlapping salary must produce positive salary_score, got {}",
        with_salary_fit.score_breakdown.salary_score,
    );
}

#[test]
fn salary_clearly_below_minimum_applies_negative_score_effect() {
    let service = SearchMatchingService::new();
    let base = job_view(
        "job-salary-low",
        "Senior Backend Developer",
        "Remote EU role with Rust and Postgres",
        Some("remote"),
        "djinni",
    );
    // salary_max $2800 is >20% below expected min of $4000
    let low_salary_job = JobView {
        job: Job {
            salary_min: Some(2000),
            salary_max: Some(2800),
            salary_currency: Some("USD".to_string()),
            ..base.job.clone()
        },
        ..base.clone()
    };
    let no_salary_job = base;

    let mut profile = search_profile();
    profile.salary_expectation = Some(SearchSalaryExpectation {
        min: Some(4000),
        max: Some(7000),
        currency: "USD".to_string(),
    });

    let low_salary_fit = service.score_job_deterministic(&profile, &low_salary_job);
    let no_salary_fit = service.score_job_deterministic(&profile, &no_salary_job);

    assert!(
        low_salary_fit.score < no_salary_fit.score,
        "below-minimum salary must score lower than missing salary: low={}, no_salary={}",
        low_salary_fit.score,
        no_salary_fit.score,
    );
    assert!(
        low_salary_fit.score_breakdown.salary_score < 0,
        "below-minimum salary must produce negative salary_score, got {}",
        low_salary_fit.score_breakdown.salary_score,
    );
    assert!(
        low_salary_fit
            .reasons
            .iter()
            .any(|r| r.to_lowercase().contains("below")),
        "below-minimum salary must produce a penalty reason"
    );
}

#[test]
fn salary_unrecognized_currency_does_not_penalize() {
    let service = SearchMatchingService::new();
    let base = job_view(
        "job-gbp-salary",
        "Senior Backend Developer",
        "Remote EU role with Rust and Postgres",
        Some("remote"),
        "djinni",
    );
    // GBP is not in the recognized currency list
    let gbp_job = JobView {
        job: Job {
            salary_min: Some(1500),
            salary_max: Some(2500),
            salary_currency: Some("GBP".to_string()),
            ..base.job
        },
        ..base
    };

    let mut profile = search_profile();
    profile.salary_expectation = Some(SearchSalaryExpectation {
        min: Some(4000),
        max: Some(7000),
        currency: "USD".to_string(),
    });

    let fit = service.score_job_deterministic(&profile, &gbp_job);

    assert_eq!(
        fit.score_breakdown.salary_score, 0,
        "unrecognized job currency must not affect salary score, got {}",
        fit.score_breakdown.salary_score,
    );
    assert!(
        !fit.reasons
            .iter()
            .any(|r| r.to_lowercase().contains("salary")),
        "unrecognized job currency must not produce a salary reason"
    );
}

#[test]
fn salary_score_stays_within_zero_to_one_hundred() {
    let service = SearchMatchingService::new();
    let mut profile = search_profile();
    profile.salary_expectation = Some(SearchSalaryExpectation {
        min: Some(4000),
        max: Some(7000),
        currency: "USD".to_string(),
    });
    profile.allowed_sources = vec![];
    profile.work_modes = vec![WorkMode::Remote];
    profile.target_regions = vec![];

    let base = job_view(
        "job-salary-clamp",
        "Pastry Chef",
        "Onsite kitchen role preparing desserts",
        Some("onsite"),
        "djinni",
    );
    // Completely irrelevant job, old date, far-below-minimum salary
    let job = JobView {
        job: Job {
            salary_min: Some(500),
            salary_max: Some(1000),
            salary_currency: Some("USD".to_string()),
            posted_at: Some("2020-01-01T00:00:00Z".to_string()),
            last_seen_at: "2020-01-01T00:00:00Z".to_string(),
            ..base.job
        },
        first_seen_at: "2020-01-01T00:00:00Z".to_string(),
        ..base
    };

    let fit = service.score_job_deterministic(&profile, &job);

    assert_eq!(
        fit.score, fit.score_breakdown.total_score,
        "fit.score and score_breakdown.total_score must stay in sync"
    );
    assert!(fit.score <= 100, "score must not exceed 100");
    assert!(
        fit.score_breakdown.salary_score < 0,
        "far-below-target salary must produce negative salary_score"
    );
}

#[test]
fn salary_match_and_penalty_produce_explanation_reasons() {
    let service = SearchMatchingService::new();
    let base = job_view(
        "job-salary-reason",
        "Senior Backend Developer",
        "Remote EU role with Rust and Postgres",
        Some("remote"),
        "djinni",
    );
    // salary fully inside candidate range: reason "fully within"
    let matching_salary_job = JobView {
        job: Job {
            salary_min: Some(4500),
            salary_max: Some(6500),
            salary_currency: Some("USD".to_string()),
            posted_at: Some("2099-01-01T00:00:00Z".to_string()),
            last_seen_at: "2099-01-01T00:00:00Z".to_string(),
            ..base.job.clone()
        },
        first_seen_at: "2099-01-01T00:00:00Z".to_string(),
        ..base.clone()
    };
    // salary max $2800 is >20% below expected min of $4000: reason "20% below"
    let below_salary_job = JobView {
        job: Job {
            salary_min: Some(2000),
            salary_max: Some(2800),
            salary_currency: Some("USD".to_string()),
            ..base.job
        },
        ..base
    };

    let mut profile = search_profile();
    profile.salary_expectation = Some(SearchSalaryExpectation {
        min: Some(4000),
        max: Some(7000),
        currency: "USD".to_string(),
    });

    let matching_fit = service.score_job_deterministic(&profile, &matching_salary_job);
    let below_fit = service.score_job_deterministic(&profile, &below_salary_job);

    assert!(
        matching_fit
            .reasons
            .iter()
            .any(|r| r.contains("Salary range is fully within the profile target")),
        "matching salary must produce a positive explanation reason; got: {:?}",
        matching_fit.reasons,
    );
    assert!(
        below_fit
            .reasons
            .iter()
            .any(|r| r.contains("Salary range is more than 20% below the profile minimum")),
        "below-target salary must produce a penalty explanation reason; got: {:?}",
        below_fit.reasons,
    );
}
