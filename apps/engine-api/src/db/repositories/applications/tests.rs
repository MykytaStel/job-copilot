use crate::db::Database;
use crate::db::repositories::{ApplicationsRepository, RepositoryError};
use crate::domain::application::model::CreateApplication;

#[tokio::test]
async fn returns_disabled_error_without_database() {
    let repository = ApplicationsRepository::new(Database::disabled());

    let error = repository
        .get_by_id("application-1")
        .await
        .expect_err("repository should fail without configured database");

    assert!(matches!(error, RepositoryError::DatabaseDisabled));
}

#[tokio::test]
async fn create_returns_disabled_without_database() {
    let repository = ApplicationsRepository::new(Database::disabled());

    let error = repository
        .create(&CreateApplication {
            profile_id: None,
            job_id: "job-1".to_string(),
            status: "saved".to_string(),
            applied_at: None,
        })
        .await
        .expect_err("repository should fail without configured database");

    assert!(matches!(error, RepositoryError::DatabaseDisabled));
}
