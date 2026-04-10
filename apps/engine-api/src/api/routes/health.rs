use axum::{Json, extract::State};
use serde::Serialize;

use crate::db::DatabaseStatus;
use crate::state::AppState;

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: &'static str,
    pub database: DatabaseStatus,
}

#[derive(Serialize)]
pub struct PingResponse {
    pub message: &'static str,
    pub app: String,
    pub version: String,
    pub database: DatabaseStatus,
}

pub async fn health(State(state): State<AppState>) -> Json<HealthResponse> {
    let database = state.database.status().await;

    Json(HealthResponse {
        status: if database.status == "error" {
            "degraded"
        } else {
            "ok"
        },
        database,
    })
}

pub async fn ping(State(state): State<AppState>) -> Json<PingResponse> {
    let database = state.database.status().await;

    Json(PingResponse {
        message: "pong",
        app: state.app_name,
        version: state.version,
        database,
    })
}

#[cfg(test)]
mod tests {
    use axum::Json;
    use axum::extract::State;

    use crate::state::AppState;

    use super::{health, ping};

    #[tokio::test]
    async fn reports_disabled_database_in_health_response() {
        let Json(response) = health(State(AppState::without_database())).await;

        assert_eq!(response.status, "ok");
        assert_eq!(response.database.status, "disabled");
        assert!(!response.database.configured);
    }

    #[tokio::test]
    async fn reports_disabled_database_in_ping_response() {
        let Json(response) = ping(State(AppState::without_database())).await;

        assert_eq!(response.message, "pong");
        assert_eq!(response.database.status, "disabled");
        assert!(!response.database.configured);
    }
}
