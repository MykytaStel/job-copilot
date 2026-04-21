use crate::db::Database;
use crate::db::repositories::RepositoryError;

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
