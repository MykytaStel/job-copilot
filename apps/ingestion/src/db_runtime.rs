use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;

pub async fn connect(database_url: &str) -> Result<PgPool, String> {
    PgPoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await
        .map_err(|error| format!("failed to connect to Postgres: {error}"))
}

/// Apply all engine-api migrations so ingestion can run standalone without
/// engine-api having started first. Safe to call multiple times (sqlx is idempotent).
pub async fn run_migrations(pool: &PgPool) -> Result<(), String> {
    sqlx::migrate!("../engine-api/migrations")
        .run(pool)
        .await
        .map_err(|error| format!("failed to run migrations: {error}"))
}
