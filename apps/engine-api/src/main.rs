mod api;
mod config;
mod db;
mod domain;
mod services;
mod state;

use crate::config::Config;
use crate::db::Database;
use crate::state::AppState;
use tokio::net::TcpListener;
use tracing::info;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().with_env_filter("debug").init();

    let config = Config::from_env();
    let database = Database::from_config(&config)
        .await
        .expect("Failed to initialize Postgres foundation");
    let state = AppState::new_with_config(database, &config);
    let app = api::build_router(state);

    let address = format!("0.0.0.0:{}", config.port);
    let listener = TcpListener::bind(&address)
        .await
        .expect("Failed to bind TCP listener");

    info!("engine-api is running on http://{}", address);

    axum::serve(listener, app).await.expect("Server failed");
}
