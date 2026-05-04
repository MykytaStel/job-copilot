use tracing::info;

use crate::cli::{Config, RunMode};
use crate::error::Result;
use crate::{db_runtime, pipeline};

pub(crate) async fn run(config: Config) -> Result<()> {
    match config.run_mode {
        RunMode::Daemon(ref daemon_mode) => {
            let pool = db_runtime::connect(&config.database_url).await?;
            db_runtime::run_migrations(&pool).await?;
            info!("migrations applied");
            pipeline::daemon::run_daemon(daemon_mode, &pool).await
        }
        RunMode::Scrape(ref scrape_mode) => {
            let pool = db_runtime::connect(&config.database_url).await?;
            pipeline::run_once::run_scrape(scrape_mode, &pool).await
        }
        RunMode::File(ref file_mode) => {
            pipeline::run_once::run_file(file_mode, &config.database_url).await
        }
    }
}
