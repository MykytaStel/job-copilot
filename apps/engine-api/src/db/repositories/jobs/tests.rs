use crate::db::Database;
use crate::db::repositories::{JobsRepository, RepositoryError};

use super::queries::job_view_query;

#[tokio::test]
async fn returns_disabled_error_without_database() {
    let repository = JobsRepository::new(Database::disabled());

    let error = repository
        .list_filtered_views(10, None, None)
        .await
        .expect_err("repository should fail without configured database");

    assert!(matches!(error, RepositoryError::DatabaseDisabled));
}

#[test]
fn job_view_query_appends_limit_after_sorting() {
    let query = job_view_query(None, Some("LIMIT $1"));

    assert!(query.contains("LIMIT $1"));
    assert!(query.contains("ORDER BY jobs.last_seen_at DESC"));
}
