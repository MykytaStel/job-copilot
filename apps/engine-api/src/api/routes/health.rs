use std::sync::OnceLock;
use std::time::{Duration, Instant};

use axum::{Json, extract::State, http::StatusCode};
use serde::Serialize;
use sqlx::{Row, query, query_scalar};
use tokio::sync::Mutex;
use tokio::time::timeout;

use crate::db::DatabaseStatus;
use crate::state::AppState;

const DB_READY_TIMEOUT: Duration = Duration::from_millis(300);
const ML_READY_TIMEOUT: Duration = Duration::from_millis(500);
const INGESTION_READY_TIMEOUT: Duration = Duration::from_millis(300);
const INGESTION_CACHE_TTL: Duration = Duration::from_secs(60);

static INGESTION_CACHE: OnceLock<Mutex<Option<CachedIngestionStatus>>> = OnceLock::new();

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: &'static str,
    pub database: DatabaseStatus,
}

#[derive(Clone, Debug)]
struct CachedIngestionStatus {
    checked_at: Instant,
    component: IngestionComponent,
}

#[derive(Clone, Debug, Serialize)]
pub struct ReadyResponse {
    pub status: &'static str,
    pub components: ReadyComponents,
}

#[derive(Clone, Debug, Serialize)]
pub struct ReadyComponents {
    pub database: DatabaseComponent,
    pub ml_sidecar: MlSidecarComponent,
    pub ingestion: IngestionComponent,
}

#[derive(Clone, Debug, Serialize)]
pub struct DatabaseComponent {
    pub status: &'static str,
    pub latency_ms: u32,
}

#[derive(Clone, Debug, Serialize)]
pub struct MlSidecarComponent {
    pub status: &'static str,
}

#[derive(Clone, Debug, Serialize)]
pub struct IngestionComponent {
    pub status: &'static str,
    pub last_run_at: Option<String>,
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

pub async fn ready(State(state): State<AppState>) -> (StatusCode, Json<ReadyResponse>) {
    let database_check = check_database(&state);
    let ml_sidecar_check = check_ml_sidecar(&state);
    let (database, ml_sidecar) = tokio::join!(database_check, ml_sidecar_check);

    let ingestion = if database.status == "ok" {
        check_ingestion(&state).await
    } else {
        IngestionComponent {
            status: "stale",
            last_run_at: None,
        }
    };

    let status = if database.status == "error" {
        "not_ready"
    } else if ml_sidecar.status != "ok" || ingestion.status != "ok" {
        "degraded"
    } else {
        "ready"
    };

    let http_status = if status == "not_ready" {
        StatusCode::SERVICE_UNAVAILABLE
    } else {
        StatusCode::OK
    };

    (
        http_status,
        Json(ReadyResponse {
            status,
            components: ReadyComponents {
                database,
                ml_sidecar,
                ingestion,
            },
        }),
    )
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

async fn check_database(state: &AppState) -> DatabaseComponent {
    let Some(pool) = state.database.pool() else {
        return DatabaseComponent {
            status: "error",
            latency_ms: 0,
        };
    };

    let started = Instant::now();
    let result = timeout(
        DB_READY_TIMEOUT,
        query_scalar::<_, i32>("SELECT 1").fetch_one(pool),
    )
    .await;

    DatabaseComponent {
        status: match result {
            Ok(Ok(1)) => "ok",
            _ => "error",
        },
        latency_ms: elapsed_ms(started),
    }
}

async fn check_ml_sidecar(state: &AppState) -> MlSidecarComponent {
    let Ok(client) = reqwest::Client::builder().timeout(ML_READY_TIMEOUT).build() else {
        return MlSidecarComponent { status: "error" };
    };

    let url = format!("{}/health", state.ml_sidecar_base_url);
    match timeout(ML_READY_TIMEOUT, client.get(url).send()).await {
        Ok(Ok(response)) if response.status().is_success() => MlSidecarComponent { status: "ok" },
        Ok(Ok(_)) => MlSidecarComponent { status: "error" },
        _ => MlSidecarComponent {
            status: "unreachable",
        },
    }
}

async fn check_ingestion(state: &AppState) -> IngestionComponent {
    let cache = INGESTION_CACHE.get_or_init(|| Mutex::new(None));
    let mut guard = cache.lock().await;

    if let Some(cached) = guard.as_ref()
        && cached.checked_at.elapsed() < INGESTION_CACHE_TTL
    {
        return cached.component.clone();
    }

    let component = query_ingestion(state).await;
    *guard = Some(CachedIngestionStatus {
        checked_at: Instant::now(),
        component: component.clone(),
    });
    component
}

async fn query_ingestion(state: &AppState) -> IngestionComponent {
    let Some(pool) = state.database.pool() else {
        return IngestionComponent {
            status: "stale",
            last_run_at: None,
        };
    };

    let result = timeout(
        INGESTION_READY_TIMEOUT,
        query(
            r#"
            SELECT
                MAX(first_seen_at)::text AS last_run_at,
                COALESCE(MAX(first_seen_at) >= NOW() - INTERVAL '2 hours', FALSE) AS fresh
            FROM jobs
            "#,
        )
        .fetch_one(pool),
    )
    .await;

    match result {
        Ok(Ok(row)) => {
            let last_run_at: Option<String> = row.try_get("last_run_at").ok().flatten();
            let fresh = row.try_get::<bool, _>("fresh").unwrap_or(false);
            IngestionComponent {
                status: if fresh { "ok" } else { "stale" },
                last_run_at,
            }
        }
        _ => IngestionComponent {
            status: "stale",
            last_run_at: None,
        },
    }
}

fn elapsed_ms(started: Instant) -> u32 {
    started.elapsed().as_millis().min(u128::from(u32::MAX)) as u32
}

#[cfg(test)]
mod tests {
    use axum::Json;
    use axum::extract::State;
    use axum::http::StatusCode;

    use crate::state::AppState;

    use super::{health, ping, ready};

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

    #[tokio::test]
    async fn reports_not_ready_when_database_is_disabled() {
        let (status_code, Json(response)) = ready(State(AppState::without_database())).await;

        assert_eq!(status_code, StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(response.status, "not_ready");
        assert_eq!(response.components.database.status, "error");
        assert_eq!(response.components.ingestion.status, "stale");
    }
}
