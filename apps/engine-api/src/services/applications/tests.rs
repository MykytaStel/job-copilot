use crate::db::Database;
use crate::db::repositories::RepositoryError;
use crate::domain::application::model::{Application, CreateApplication};

use super::{ApplicationsService, ApplicationsServiceStub};

#[tokio::test]
async fn returns_disabled_error_without_database() {
    let service = ApplicationsService::new(crate::db::repositories::ApplicationsRepository::new(
        Database::disabled(),
    ));

    let error = service
        .get_by_id("application-1")
        .await
        .expect_err("service should fail without configured database");

    assert!(matches!(error, RepositoryError::DatabaseDisabled));
}

#[tokio::test]
async fn returns_none_for_unknown_application_in_stub() {
    let service = ApplicationsService::for_tests(ApplicationsServiceStub::default());

    let application = service
        .get_by_id("missing-application")
        .await
        .expect("stub should not fail");

    assert!(application.is_none());
}

#[tokio::test]
async fn create_returns_application_and_get_retrieves_it() {
    let service = ApplicationsService::for_tests(ApplicationsServiceStub::default());

    let created = service
        .create(CreateApplication {
            profile_id: None,
            job_id: "job-001".to_string(),
            status: "saved".to_string(),
            applied_at: None,
        })
        .await
        .expect("stub should not fail on create");

    assert_eq!(created.id, "application_test_001");
    assert_eq!(created.job_id, "job-001");
    assert_eq!(created.status, "saved");

    let fetched = service
        .get_by_id(&created.id)
        .await
        .expect("stub should not fail on get")
        .expect("created application should be retrievable by id");

    assert_eq!(fetched.id, created.id);
    assert_eq!(fetched.job_id, created.job_id);
}

#[tokio::test]
async fn seeded_application_is_retrievable_via_with_application() {
    let seeded = Application {
        id: "app-seeded-001".to_string(),
        job_id: "job-seeded".to_string(),
        resume_id: None,
        status: "interviewing".to_string(),
        applied_at: Some("2026-04-01T00:00:00Z".to_string()),
        due_date: None,
        outcome: None,
        outcome_date: None,
        rejection_stage: None,
        updated_at: "2026-04-01T00:00:00Z".to_string(),
    };

    let service = ApplicationsService::for_tests(
        ApplicationsServiceStub::default().with_application(seeded.clone()),
    );

    let fetched = service
        .get_by_id("app-seeded-001")
        .await
        .expect("stub should not fail")
        .expect("seeded application should be present");

    assert_eq!(fetched.id, seeded.id);
    assert_eq!(fetched.status, "interviewing");
}
