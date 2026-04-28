use crate::db::Database;
use crate::db::repositories::RepositoryError;
use crate::domain::profile::model::{CreateProfile, ProfileAnalysis};
use crate::domain::role::RoleId;

use super::{ProfilesService, ProfilesServiceStub};

#[tokio::test]
async fn returns_disabled_error_without_database() {
    let service = ProfilesService::new(crate::db::repositories::ProfilesRepository::new(
        Database::disabled(),
    ));

    let error = service
        .get_by_id("profile-1")
        .await
        .expect_err("service should fail without configured database");

    assert!(matches!(error, RepositoryError::DatabaseDisabled));
}

#[tokio::test]
async fn creates_profile_in_stub() {
    let service = ProfilesService::for_tests(ProfilesServiceStub::default());

    let profile = service
        .create(CreateProfile {
            name: "Jane Doe".to_string(),
            email: "jane@example.com".to_string(),
            location: Some("Kyiv".to_string()),
            raw_text: "Senior frontend engineer".to_string(),
            years_of_experience: None,
            salary_min: None,
            salary_max: None,
            salary_currency: "USD".to_string(),
            languages: vec![],
            preferred_locations: vec![],
            work_mode_preference: "any".to_string(),
            search_preferences: None,
            portfolio_url: None,
            github_url: None,
            linkedin_url: None,
        })
        .await
        .expect("stub should create a profile");

    assert_eq!(profile.id, "profile_test_001");
    assert_eq!(profile.name, "Jane Doe");
}

#[tokio::test]
async fn unrelated_profile_updates_keep_skills_timestamp() {
    let service = ProfilesService::for_tests(ProfilesServiceStub::default());

    let created = service
        .create(CreateProfile {
            name: "Jane Doe".to_string(),
            email: "jane@example.com".to_string(),
            location: Some("Kyiv".to_string()),
            raw_text: "Senior frontend engineer".to_string(),
            years_of_experience: None,
            salary_min: None,
            salary_max: None,
            salary_currency: "USD".to_string(),
            languages: vec![],
            preferred_locations: vec![],
            work_mode_preference: "any".to_string(),
            search_preferences: None,
            portfolio_url: None,
            github_url: None,
            linkedin_url: None,
        })
        .await
        .expect("stub should create a profile");

    let analyzed = service
        .save_analysis(
            &created.id,
            ProfileAnalysis {
                summary: "Experienced frontend engineer".to_string(),
                primary_role: RoleId::FrontendEngineer,
                seniority: "senior".to_string(),
                skills: vec!["react".to_string()],
                keywords: vec!["frontend".to_string()],
            },
        )
        .await
        .expect("analysis save should succeed")
        .expect("profile should exist");

    let updated = service
        .update(
            &created.id,
            crate::domain::profile::model::UpdateProfile {
                name: Some("Jane Smith".to_string()),
                ..Default::default()
            },
        )
        .await
        .expect("profile update should succeed")
        .expect("profile should exist");

    assert_eq!(updated.name, "Jane Smith");
    assert_eq!(updated.skills_updated_at, analyzed.skills_updated_at);
}
