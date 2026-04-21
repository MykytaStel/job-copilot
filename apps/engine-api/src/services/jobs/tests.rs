use crate::db::Database;
use crate::db::repositories::RepositoryError;

use super::{JobsService, JobsServiceStub};

#[tokio::test]
async fn returns_disabled_error_without_database() {
    let service = JobsService::new(crate::db::repositories::JobsRepository::new(
        Database::disabled(),
    ));

    let error = service
        .get_by_id("job-1")
        .await
        .expect_err("service should fail without configured database");

    assert!(matches!(error, RepositoryError::DatabaseDisabled));
}

#[tokio::test]
async fn returns_none_for_unknown_job_in_stub() {
    let service = JobsService::for_tests(JobsServiceStub::default());

    let job = service
        .get_by_id("missing-job")
        .await
        .expect("stub should not fail");

    assert!(job.is_none());
}
