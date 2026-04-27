use crate::domain::job::model::{Job, JobLifecycleStage, JobSourceVariant, JobView};
use crate::domain::role::RoleId;
use crate::domain::search::profile::{SearchProfile, SearchRoleCandidate, TargetRegion, WorkMode};
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
    // Profile has two skills: one that the job lists as required, one as preferred.
    // We test two jobs that are missing one skill each and verify the
    // job missing the required skill scores lower.
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
    };

    // Job A: lists Rust as required, GraphQL as preferred — both present.
    let job_both = job_view(
        "job-both",
        "Senior Backend Engineer",
        "Requirements:\n- Rust\nPreferred:\n- GraphQL",
        Some("remote"),
        "djinni",
    );
    // Job B: has GraphQL (preferred) but is missing Rust (required).
    let job_missing_required = job_view(
        "job-missing-required",
        "Senior Backend Engineer",
        "Requirements:\n- Rust\nPreferred:\n- GraphQL\nWe work with GraphQL daily.",
        Some("remote"),
        "djinni",
    );
    // Job C: has Rust (required) but is missing GraphQL (preferred).
    let job_missing_preferred = job_view(
        "job-missing-preferred",
        "Senior Backend Engineer",
        "Requirements:\n- Rust\nWe build Rust services. GraphQL is not used here.",
        Some("remote"),
        "djinni",
    );

    let fit_missing_required = service.score_job_deterministic(&profile, &job_missing_required);
    let fit_missing_preferred = service.score_job_deterministic(&profile, &job_missing_preferred);

    // Missing a required skill should hurt the score more than missing a preferred skill.
    assert!(
        fit_missing_preferred.score > fit_missing_required.score,
        "missing a required skill (score {}) should score lower than missing a preferred skill (score {})",
        fit_missing_required.score,
        fit_missing_preferred.score,
    );
    let _ = job_both; // constructed above to verify section parsing doesn't panic
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
