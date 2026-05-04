mod adapters;
mod app;
mod cli;
mod db;
mod db_runtime;
mod error;
mod models;
mod pipeline;
mod scrapers;

#[tokio::main]
async fn main() -> error::Result<()> {
    tracing_subscriber::fmt().with_env_filter("info").init();
    let config = cli::Config::from_env()?;
    app::run(config).await
}
