use crate::domain::candidate::profile::{CandidateProfile, RoleScore};
use crate::domain::role::RoleId;
use crate::domain::search::profile::{
    SearchPreferences, SearchRoleCandidate, TargetRegion, WorkMode,
};
use crate::domain::source::SourceId;

use super::service::SearchProfileService;

#[test]
fn builds_search_profile_from_profile_and_preferences() {
    let service = SearchProfileService::new();
    let analyzed_profile = sample_profile();
    let preferences = SearchPreferences {
        target_regions: vec![TargetRegion::Ua, TargetRegion::EuRemote],
        work_modes: vec![WorkMode::Remote, WorkMode::Hybrid],
        preferred_roles: vec![RoleId::FrontendEngineer],
        allowed_sources: vec![SourceId::Djinni, SourceId::WorkUa],
        include_keywords: vec!["product company".to_string()],
        exclude_keywords: vec!["gambling".to_string()],
        scoring_weights: Default::default(),
    };

    let search_profile = service.build(&analyzed_profile, &preferences);

    assert_eq!(search_profile.primary_role, RoleId::MobileEngineer);
    assert_eq!(search_profile.primary_role_confidence, Some(100));
    assert_eq!(search_profile.seniority, "senior");
    assert_eq!(
        search_profile.target_roles,
        vec![RoleId::MobileEngineer, RoleId::FrontendEngineer,]
    );
    assert_eq!(
        search_profile.target_regions,
        vec![TargetRegion::Ua, TargetRegion::EuRemote]
    );
    assert_eq!(
        search_profile.work_modes,
        vec![WorkMode::Remote, WorkMode::Hybrid]
    );
    assert_eq!(
        search_profile.allowed_sources,
        vec![SourceId::Djinni, SourceId::WorkUa]
    );
    assert_eq!(
        search_profile.role_candidates,
        vec![SearchRoleCandidate {
            role: RoleId::MobileEngineer,
            confidence: 100,
        },]
    );
    assert_eq!(
        search_profile.profile_skills,
        vec!["react native", "typescript"]
    );
    assert_eq!(search_profile.profile_keywords, vec!["mobile"]);
    assert_eq!(
        search_profile.search_terms,
        vec![
            "mobile engineer",
            "senior mobile engineer",
            "frontend engineer",
            "product company",
        ]
    );
    assert_eq!(search_profile.exclude_terms, vec!["gambling"]);
    assert_eq!(search_profile.scoring_weights.skill_match_importance, 8);
    assert_eq!(search_profile.scoring_weights.salary_fit_importance, 6);
    assert_eq!(search_profile.scoring_weights.job_freshness_importance, 5);
    assert_eq!(search_profile.scoring_weights.remote_work_importance, 5);
}

#[test]
fn merges_preferred_roles_without_duplicates() {
    let service = SearchProfileService::new();
    let analyzed_profile = sample_profile();
    let preferences = SearchPreferences {
        preferred_roles: vec![RoleId::MobileEngineer, RoleId::FrontendEngineer],
        ..SearchPreferences::default()
    };

    let search_profile = service.build(&analyzed_profile, &preferences);

    assert_eq!(
        search_profile.target_roles,
        vec![RoleId::MobileEngineer, RoleId::FrontendEngineer,]
    );
    assert_eq!(
        search_profile.search_terms,
        vec![
            "mobile engineer",
            "senior mobile engineer",
            "frontend engineer",
        ]
    );
}

#[test]
fn merges_include_and_exclude_keywords_correctly() {
    let service = SearchProfileService::new();
    let analyzed_profile = sample_profile();
    let preferences = SearchPreferences {
        include_keywords: vec![
            "product company".to_string(),
            "remote-first".to_string(),
            "product company".to_string(),
        ],
        exclude_keywords: vec![
            "outsourcing".to_string(),
            "gambling".to_string(),
            "outsourcing".to_string(),
        ],
        ..SearchPreferences::default()
    };

    let search_profile = service.build(&analyzed_profile, &preferences);

    assert_eq!(
        search_profile.search_terms,
        vec![
            "mobile engineer",
            "senior mobile engineer",
            "product company",
            "remote-first",
        ]
    );
    assert_eq!(
        search_profile.exclude_terms,
        vec!["outsourcing", "gambling"]
    );
}

#[test]
fn works_when_preferences_are_mostly_empty() {
    let service = SearchProfileService::new();
    let analyzed_profile = sample_profile();

    let search_profile = service.build(&analyzed_profile, &SearchPreferences::default());

    assert_eq!(search_profile.target_regions, Vec::<TargetRegion>::new());
    assert_eq!(search_profile.work_modes, Vec::<WorkMode>::new());
    assert_eq!(search_profile.allowed_sources, Vec::<SourceId>::new());
    assert_eq!(search_profile.primary_role_confidence, Some(100));
    assert_eq!(search_profile.target_roles, vec![RoleId::MobileEngineer]);
    assert_eq!(
        search_profile.search_terms,
        vec!["mobile engineer", "senior mobile engineer",]
    );
    assert!(search_profile.exclude_terms.is_empty());
}

#[test]
fn preserves_analyzed_profile_primary_role() {
    let service = SearchProfileService::new();
    let mut analyzed_profile = sample_profile();
    analyzed_profile.primary_role = RoleId::FrontendEngineer;

    let search_profile = service.build(&analyzed_profile, &SearchPreferences::default());

    assert_eq!(search_profile.primary_role, RoleId::FrontendEngineer);
}

fn sample_profile() -> CandidateProfile {
    CandidateProfile {
        summary: "Senior mobile candidate".to_string(),
        primary_role: RoleId::MobileEngineer,
        seniority: "senior".to_string(),
        skills: vec!["react native".to_string(), "typescript".to_string()],
        keywords: vec!["mobile".to_string()],
        role_candidates: vec![RoleScore {
            role: RoleId::MobileEngineer,
            score: 30,
            confidence: 100,
            matched_signals: vec!["react native".to_string()],
        }],
        suggested_search_terms: vec![
            "mobile engineer".to_string(),
            "senior mobile engineer".to_string(),
        ],
    }
}
