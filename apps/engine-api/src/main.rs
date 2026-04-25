mod api;
mod config;
mod db;
mod domain;
mod services;
mod state;

use crate::config::Config;
use crate::db::Database;
use crate::services::reranker_automation::spawn_retrain_poller;
use crate::state::AppState;
use tokio::net::TcpListener;
use tracing::info;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    let env_filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("info"))
        .expect("valid tracing filter");

    tracing_subscriber::fmt()
        .json()
        .with_env_filter(env_filter)
        .init();

    let config = Config::from_env();
    let database = Database::from_config(&config)
        .await
        .expect("Failed to initialize Postgres foundation");

    let state = AppState::new_with_config(database, &config);
    spawn_retrain_poller(
        state.clone(),
        config.ml_retrain_threshold,
        config.ml_retrain_poll_interval_seconds,
    );
    let routers = api::build_routers(state);

    let internal_listener = TcpListener::bind("127.0.0.1:9090")
        .await
        .expect("Failed to bind internal listener");
    tokio::spawn(async move {
        axum::serve(internal_listener, routers.internal)
            .await
            .expect("Internal server failed");
    });

    let address = format!("0.0.0.0:{}", config.port);
    let listener = TcpListener::bind(&address)
        .await
        .expect("Failed to bind TCP listener");

    info!("engine-api is running on http://{}", address);

    axum::serve(listener, routers.app)
        .await
        .expect("Server failed");
}
