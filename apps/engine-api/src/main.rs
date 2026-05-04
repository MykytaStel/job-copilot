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
    init_tracing();

    let config = Config::from_env().unwrap_or_else(|err| {
        eprintln!("engine-api configuration error: {err}");
        std::process::exit(1);
    });
    info!(
        app_env = config.app_env,
        production = config.is_production(),
        "loaded engine-api configuration"
    );
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

    let internal_address = format!("0.0.0.0:{}", config.metrics_port);
    let internal_listener = TcpListener::bind(&internal_address)
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

fn init_tracing() {
    let env_filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("info"))
        .expect("valid tracing filter");
    let subscriber = tracing_subscriber::fmt().with_env_filter(env_filter);

    if std::env::var("LOG_FORMAT")
        .map(|value| value.eq_ignore_ascii_case("json"))
        .unwrap_or(false)
    {
        subscriber.json().init();
    } else {
        subscriber.init();
    }
}
