use tracing::warn;

use crate::services::search_ranking::runtime::RerankerRuntimeMode;

#[derive(Clone)]
pub struct Config {
    pub port: u16,
    pub database_url: Option<String>,
    pub database_max_connections: u32,
    pub run_db_migrations: bool,
    pub reranker_runtime_mode: RerankerRuntimeMode,
    pub learned_reranker_enabled: bool,
    pub trained_reranker_enabled: bool,
    pub trained_reranker_model_path: Option<String>,
    pub ml_sidecar_base_url: String,
    pub ml_sidecar_timeout_seconds: u64,
    pub ml_retrain_threshold: usize,
    pub ml_retrain_poll_interval_seconds: u64,
    pub jwt_secret: Option<String>,
}

impl Config {
    pub fn from_env() -> Self {
        let port = std::env::var("PORT")
            .ok()
            .and_then(|value| value.parse::<u16>().ok())
            .unwrap_or(8080);
        let database_url = std::env::var("DATABASE_URL")
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty());
        let database_max_connections = std::env::var("DATABASE_MAX_CONNECTIONS")
            .ok()
            .and_then(|value| value.parse::<u32>().ok())
            .unwrap_or(5);
        let run_db_migrations = std::env::var("RUN_DB_MIGRATIONS")
            .ok()
            .as_deref()
            .map(parse_bool)
            .unwrap_or(true);
        let learned_reranker_enabled = std::env::var("LEARNED_RERANKER_ENABLED")
            .ok()
            .as_deref()
            .map(parse_bool)
            .unwrap_or(true);
        let trained_reranker_enabled = std::env::var("TRAINED_RERANKER_ENABLED")
            .ok()
            .as_deref()
            .map(parse_bool)
            .unwrap_or(false);
        let reranker_runtime_mode = std::env::var("RERANKER_RUNTIME_MODE")
            .ok()
            .and_then(|value| {
                let trimmed = value.trim().to_string();
                let parsed = RerankerRuntimeMode::parse(&trimmed);

                if parsed.is_none() {
                    warn!(
                        mode = trimmed,
                        "invalid RERANKER_RUNTIME_MODE; falling back to feature-flag derived mode"
                    );
                }

                parsed
            })
            .unwrap_or_else(|| {
                RerankerRuntimeMode::default_from_flags(
                    learned_reranker_enabled,
                    trained_reranker_enabled,
                )
            });
        let trained_reranker_model_path = std::env::var("TRAINED_RERANKER_MODEL_PATH")
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty());
        let ml_sidecar_base_url = std::env::var("ML_SIDECAR_BASE_URL")
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| "http://localhost:8000".to_string());
        let ml_sidecar_timeout_seconds = std::env::var("ML_SIDECAR_TIMEOUT_SECONDS")
            .ok()
            .and_then(|value| value.parse::<u64>().ok())
            .unwrap_or(15);
        let ml_retrain_threshold = std::env::var("ML_RETRAIN_THRESHOLD")
            .ok()
            .and_then(|value| value.parse::<usize>().ok())
            .unwrap_or(15);
        let ml_retrain_poll_interval_seconds = std::env::var("ML_RETRAIN_POLL_INTERVAL_SECONDS")
            .ok()
            .and_then(|value| value.parse::<u64>().ok())
            .unwrap_or(21_600);
        let jwt_secret = std::env::var("JWT_SECRET")
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty());

        Self {
            port,
            database_url,
            database_max_connections,
            run_db_migrations,
            reranker_runtime_mode,
            learned_reranker_enabled,
            trained_reranker_enabled,
            trained_reranker_model_path,
            ml_sidecar_base_url,
            ml_sidecar_timeout_seconds,
            ml_retrain_threshold,
            ml_retrain_poll_interval_seconds,
            jwt_secret,
        }
    }
}

fn parse_bool(value: &str) -> bool {
    matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "1" | "true" | "yes" | "on"
    )
}

#[cfg(test)]
mod tests {
    use crate::services::search_ranking::runtime::RerankerRuntimeMode;

    use super::parse_bool;

    #[test]
    fn parses_truthy_booleans() {
        assert!(parse_bool("true"));
        assert!(parse_bool("1"));
        assert!(parse_bool("yes"));
        assert!(parse_bool("on"));
        assert!(!parse_bool("false"));
        assert!(!parse_bool("0"));
    }

    #[test]
    fn parses_runtime_mode_values() {
        assert_eq!(
            RerankerRuntimeMode::parse("deterministic"),
            Some(RerankerRuntimeMode::Deterministic)
        );
        assert_eq!(
            RerankerRuntimeMode::parse("learned"),
            Some(RerankerRuntimeMode::Learned)
        );
        assert_eq!(
            RerankerRuntimeMode::parse("trained"),
            Some(RerankerRuntimeMode::Trained)
        );
        assert_eq!(RerankerRuntimeMode::parse("mystery"), None);
    }
}
