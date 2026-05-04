use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;

use crate::error::{IngestionError, Result};

pub async fn connect(database_url: &str) -> Result<PgPool> {
    PgPoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await
        .map_err(IngestionError::Database)
}

/// Apply all engine-api migrations so ingestion can run standalone without
/// engine-api having started first. Safe to call multiple times (sqlx is idempotent).
pub async fn run_migrations(pool: &PgPool) -> Result<()> {
    sqlx::migrate!("../engine-api/migrations")
        .run(pool)
        .await
        .map_err(|error| IngestionError::Message(format!("failed to run migrations: {error}")))
}
