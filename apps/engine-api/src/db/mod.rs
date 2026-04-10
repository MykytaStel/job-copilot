use serde::Serialize;
use sqlx::postgres::PgPoolOptions;
use sqlx::{PgPool, query_scalar};

use crate::config::Config;

#[derive(Clone)]
pub struct Database {
    pool: Option<PgPool>,
    migrations_enabled_on_startup: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct DatabaseStatus {
    pub status: &'static str,
    pub configured: bool,
    pub migrations_enabled_on_startup: bool,
}

impl Database {
    pub async fn from_config(config: &Config) -> Result<Self, String> {
        let Some(database_url) = &config.database_url else {
            return Ok(Self::disabled());
        };

        let pool = PgPoolOptions::new()
            .max_connections(config.database_max_connections)
            .connect(database_url)
            .await
            .map_err(|error| format!("failed to connect to Postgres: {error}"))?;

        if config.run_db_migrations {
            sqlx::migrate!("./migrations")
                .run(&pool)
                .await
                .map_err(|error| format!("failed to run Postgres migrations: {error}"))?;
        }

        Ok(Self {
            pool: Some(pool),
            migrations_enabled_on_startup: config.run_db_migrations,
        })
    }

    pub fn disabled() -> Self {
        Self {
            pool: None,
            migrations_enabled_on_startup: false,
        }
    }

    pub async fn status(&self) -> DatabaseStatus {
        match &self.pool {
            Some(pool) => {
                let is_healthy = query_scalar::<_, i32>("SELECT 1")
                    .fetch_one(pool)
                    .await
                    .is_ok();

                DatabaseStatus {
                    status: if is_healthy { "ok" } else { "error" },
                    configured: true,
                    migrations_enabled_on_startup: self.migrations_enabled_on_startup,
                }
            }
            None => DatabaseStatus {
                status: "disabled",
                configured: false,
                migrations_enabled_on_startup: self.migrations_enabled_on_startup,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Database;

    #[tokio::test]
    async fn reports_disabled_status_without_configuration() {
        let status = Database::disabled().status().await;

        assert_eq!(status.status, "disabled");
        assert!(!status.configured);
        assert!(!status.migrations_enabled_on_startup);
    }
}
