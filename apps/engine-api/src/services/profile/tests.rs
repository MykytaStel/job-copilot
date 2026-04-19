use super::matching::{normalize_text, tokenize};
use super::service::ProfileAnalysisService;
use crate::domain::role::RoleId;

#[test]
fn normalizes_punctuation_into_safe_phrase_text() {
    assert_eq!(
        normalize_text("Senior Full-Stack / React Native Engineer"),
        "senior fullstack react_native engineer"
    );
}

#[test]
fn canonicalizes_compound_terms_and_aliases() {
    assert_eq!(normalize_text("Front-end engineer"), "frontend engineer");
    assert_eq!(normalize_text("Front end engineer"), "frontend engineer");
    assert_eq!(normalize_text("Frontend engineer"), "frontend engineer");
    assert_eq!(normalize_text("Back-end engineer"), "backend engineer");
    assert_eq!(normalize_text("Full stack engineer"), "fullstack engineer");
    assert_eq!(normalize_text("Node.js platform"), "nodejs platform");
    assert_eq!(normalize_text("C++ services"), "cpp services");
    assert_eq!(normalize_text("C# services"), "csharp services");
    assert_eq!(
        normalize_text("React Native and distributed systems"),
        "react_native and distributed_systems"
    );
}

#[test]
fn tokenization_keeps_known_phrases_as_single_terms() {
    let tokens = tokenize(&normalize_text(
        "React Native distributed systems quality assurance",
    ));

    assert!(tokens.iter().any(|token| token == "react_native"));
    assert!(tokens.iter().any(|token| token == "distributed_systems"));
    assert!(tokens.iter().any(|token| token == "quality_assurance"));
    assert!(!tokens.iter().any(|token| token == "react"));
    assert!(!tokens.iter().any(|token| token == "native"));
}

#[test]
fn applies_combination_bonus_for_mobile_profiles() {
    let service = ProfileAnalysisService::new();

    let profile = service.analyze(
        "Senior React Native developer building mobile apps for iOS and Android with TypeScript and React.",
    );

    let top_role = &profile.role_candidates[0];

    assert_eq!(top_role.role, RoleId::MobileEngineer);
    assert!(top_role.score >= 20);
    assert!(
        top_role
            .matched_signals
            .iter()
            .any(|signal| signal == "bonus: react native + ios/android/mobile")
    );
}

#[test]
fn falls_back_to_generalist_when_no_role_reaches_threshold() {
    let service = ProfileAnalysisService::new();

    let profile = service.analyze("Strong communication, collaboration, and ownership.");

    assert_eq!(profile.primary_role, RoleId::Generalist);
    assert_eq!(profile.role_candidates.len(), 1);
    assert_eq!(profile.role_candidates[0].role, RoleId::Generalist);
    assert_eq!(profile.role_candidates[0].score, 0);
    assert_eq!(profile.role_candidates[0].confidence, 0);
    assert!(profile.suggested_search_terms.is_empty());
}

#[test]
fn does_not_match_java_inside_javascript() {
    let service = ProfileAnalysisService::new();

    let profile = service.analyze("JavaScript frontend developer working with React.");

    assert!(profile.skills.iter().any(|skill| skill == "javascript"));
    assert!(!profile.skills.iter().any(|skill| skill == "java"));
    assert!(profile.role_candidates.iter().all(|candidate| {
        !candidate
            .matched_signals
            .iter()
            .any(|signal| signal == "java")
    }));
}

#[test]
fn matches_react_native_as_a_phrase() {
    let service = ProfileAnalysisService::new();

    let profile = service.analyze("React Native engineer shipping cross-platform mobile apps.");

    assert_eq!(profile.primary_role, RoleId::MobileEngineer);
    assert!(profile.skills.iter().any(|skill| skill == "react native"));
}

#[test]
fn matches_full_stack_role_from_hyphenated_signal() {
    let service = ProfileAnalysisService::new();

    let profile = service.analyze("Full-stack engineer building frontend and backend systems.");

    assert_eq!(profile.primary_role, RoleId::FullstackEngineer);
    assert!(
        profile.role_candidates[0]
            .matched_signals
            .iter()
            .any(|signal| signal == "full-stack")
    );
}

#[test]
fn qa_requires_a_real_token_match() {
    let service = ProfileAnalysisService::new();

    let profile = service.analyze("Aqua testing specialist improving release quality.");

    assert!(!profile.skills.iter().any(|skill| skill == "qa"));
    assert_eq!(profile.primary_role, RoleId::Generalist);
}
