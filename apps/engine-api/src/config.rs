use thiserror::Error;
use tracing::warn;

use crate::services::search_ranking::runtime::RerankerRuntimeMode;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("invalid environment variable {name}: {reason}")]
    InvalidEnv { name: &'static str, reason: String },

    #[error("production configuration error: {0}")]
    Production(String),
}

#[derive(Clone)]
pub struct Config {
    pub port: u16,
    pub app_env: String,
    pub database_url: Option<String>,
    pub database_max_connections: u32,
    pub run_db_migrations: bool,
    pub reranker_runtime_mode: RerankerRuntimeMode,
    pub learned_reranker_enabled: bool,
    pub trained_reranker_enabled: bool,
    pub trained_reranker_model_path: Option<String>,
    pub ml_sidecar_base_url: String,
    pub ml_sidecar_timeout_seconds: u64,
    pub ml_sidecar_internal_token: Option<String>,
    pub ml_retrain_threshold: usize,
    pub ml_retrain_poll_interval_seconds: u64,
    pub jwt_secret: Option<String>,
    pub cors_allowed_origins: Vec<String>,
    pub metrics_port: u16,
}

impl Config {
    pub fn is_production(&self) -> bool {
        self.app_env == "production"
    }

    pub fn from_env() -> Result<Self, ConfigError> {
        let port = parse_env_or_default("PORT", 8080u16)?;
        let database_url = std::env::var("DATABASE_URL")
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty());
        let database_max_connections = parse_env_or_default("DATABASE_MAX_CONNECTIONS", 5u32)?;
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
        let ml_sidecar_timeout_seconds = parse_env_or_default("ML_SIDECAR_TIMEOUT_SECONDS", 90u64)?;
        let ml_sidecar_internal_token = std::env::var("ML_INTERNAL_TOKEN")
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty());
        let ml_retrain_threshold = parse_env_or_default("ML_RETRAIN_THRESHOLD", 15usize)?;
        let ml_retrain_poll_interval_seconds =
            parse_env_or_default("ML_RETRAIN_POLL_INTERVAL_SECONDS", 21_600u64)?;
        let jwt_secret = std::env::var("JWT_SECRET")
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty());
        let app_env = std::env::var("APP_ENV")
            .ok()
            .map(|value| value.trim().to_ascii_lowercase())
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| "development".to_string());
        let is_production = app_env == "production";
        let cors_allowed_origins = std::env::var("CORS_ALLOWED_ORIGINS")
            .ok()
            .map(|value| {
                value
                    .split(',')
                    .map(|origin| origin.trim().to_string())
                    .filter(|origin| !origin.is_empty())
                    .collect::<Vec<_>>()
            })
            .filter(|origins| !origins.is_empty())
            .unwrap_or_else(|| {
                if is_production {
                    Vec::new()
                } else {
                    vec![
                        "http://localhost:3000".to_string(),
                        "http://127.0.0.1:3000".to_string(),
                        "http://localhost:5173".to_string(),
                        "http://127.0.0.1:5173".to_string(),
                    ]
                }
            });
        let metrics_port = parse_env_or_default("METRICS_PORT", 9090u16)?;

        let config = Self {
            port,
            database_url,
            app_env,
            database_max_connections,
            run_db_migrations,
            reranker_runtime_mode,
            learned_reranker_enabled,
            trained_reranker_enabled,
            trained_reranker_model_path,
            ml_sidecar_base_url,
            ml_sidecar_timeout_seconds,
            ml_sidecar_internal_token,
            ml_retrain_threshold,
            ml_retrain_poll_interval_seconds,
            jwt_secret,
            cors_allowed_origins,
            metrics_port,
        };

        config.validate()?;
        Ok(config)
    }

    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.is_production() {
            if self.cors_allowed_origins.is_empty() {
                return Err(ConfigError::Production(
                    "CORS_ALLOWED_ORIGINS must be set in production".to_string(),
                ));
            }
            if !is_production_jwt_secret_acceptable(self.jwt_secret.as_deref()) {
                return Err(ConfigError::Production(
                    "JWT_SECRET must be set in production — refusing to start without authentication"
                        .to_string(),
                ));
            }
            if self.ml_sidecar_internal_token.is_none() {
                return Err(ConfigError::Production(
                    "ML_INTERNAL_TOKEN must be set in production — refusing to start without ML service authentication"
                        .to_string(),
                ));
            }
        }
        Ok(())
    }
}

fn parse_env_or_default<T>(name: &'static str, default: T) -> Result<T, ConfigError>
where
    T: std::str::FromStr,
    T::Err: std::fmt::Display,
{
    match std::env::var(name) {
        Err(_) => Ok(default),
        Ok(value) => value
            .trim()
            .parse::<T>()
            .map_err(|err| ConfigError::InvalidEnv {
                name,
                reason: err.to_string(),
            }),
    }
}

fn parse_bool(value: &str) -> bool {
    matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "1" | "true" | "yes" | "on"
    )
}

fn is_production_jwt_secret_acceptable(secret: Option<&str>) -> bool {
    let Some(secret) = secret.map(str::trim).filter(|value| !value.is_empty()) else {
        return false;
    };

    secret != "local-dev-secret-change-me"
}

#[cfg(test)]
mod tests {
    use crate::services::search_ranking::runtime::RerankerRuntimeMode;

    use super::{
        Config, ConfigError, is_production_jwt_secret_acceptable, parse_bool, parse_env_or_default,
    };

    fn minimal_prod_config() -> Config {
        Config {
            port: 8080,
            app_env: "production".to_string(),
            database_url: None,
            database_max_connections: 5,
            run_db_migrations: true,
            reranker_runtime_mode: RerankerRuntimeMode::Deterministic,
            learned_reranker_enabled: true,
            trained_reranker_enabled: false,
            trained_reranker_model_path: None,
            ml_sidecar_base_url: "http://localhost:8000".to_string(),
            ml_sidecar_timeout_seconds: 90,
            ml_sidecar_internal_token: Some("prod-token".to_string()),
            ml_retrain_threshold: 15,
            ml_retrain_poll_interval_seconds: 21_600,
            jwt_secret: Some("strong-production-secret".to_string()),
            cors_allowed_origins: vec!["https://example.com".to_string()],
            metrics_port: 9090,
        }
    }

    fn minimal_dev_config() -> Config {
        Config {
            port: 8080,
            app_env: "development".to_string(),
            database_url: None,
            database_max_connections: 5,
            run_db_migrations: true,
            reranker_runtime_mode: RerankerRuntimeMode::Deterministic,
            learned_reranker_enabled: true,
            trained_reranker_enabled: false,
            trained_reranker_model_path: None,
            ml_sidecar_base_url: "http://localhost:8000".to_string(),
            ml_sidecar_timeout_seconds: 90,
            ml_sidecar_internal_token: None,
            ml_retrain_threshold: 15,
            ml_retrain_poll_interval_seconds: 21_600,
            jwt_secret: None,
            cors_allowed_origins: vec!["http://localhost:3000".to_string()],
            metrics_port: 9090,
        }
    }

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

    #[test]
    fn rejects_missing_or_placeholder_production_jwt_secret() {
        assert!(!is_production_jwt_secret_acceptable(None));
        assert!(!is_production_jwt_secret_acceptable(Some("")));
        assert!(!is_production_jwt_secret_acceptable(Some(
            "local-dev-secret-change-me"
        )));
        assert!(is_production_jwt_secret_acceptable(Some(
            "a-real-production-secret"
        )));
    }

    #[test]
    fn valid_dev_config_passes_validate() {
        assert!(minimal_dev_config().validate().is_ok());
    }

    #[test]
    fn valid_prod_config_passes_validate() {
        assert!(minimal_prod_config().validate().is_ok());
    }

    #[test]
    fn production_requires_strong_jwt_secret() {
        let mut config = minimal_prod_config();

        config.jwt_secret = None;
        assert!(matches!(config.validate(), Err(ConfigError::Production(_))));

        config.jwt_secret = Some("local-dev-secret-change-me".to_string());
        assert!(matches!(config.validate(), Err(ConfigError::Production(_))));

        config.jwt_secret = Some("strong-production-secret".to_string());
        assert!(config.validate().is_ok());
    }

    #[test]
    fn production_requires_ml_internal_token() {
        let mut config = minimal_prod_config();

        config.ml_sidecar_internal_token = None;
        assert!(matches!(config.validate(), Err(ConfigError::Production(_))));

        config.ml_sidecar_internal_token = Some("token".to_string());
        assert!(config.validate().is_ok());
    }

    #[test]
    fn production_requires_cors_origins() {
        let mut config = minimal_prod_config();

        config.cors_allowed_origins = Vec::new();
        assert!(matches!(config.validate(), Err(ConfigError::Production(_))));

        config.cors_allowed_origins = vec!["https://example.com".to_string()];
        assert!(config.validate().is_ok());
    }

    #[test]
    fn invalid_port_fails() {
        let err = parse_env_or_default::<u16>("PORT_TEST_UNUSED", 8080).unwrap();
        assert_eq!(err, 8080);

        let result = "not-a-port"
            .trim()
            .parse::<u16>()
            .map_err(|e| ConfigError::InvalidEnv {
                name: "PORT",
                reason: e.to_string(),
            });
        assert!(matches!(
            result,
            Err(ConfigError::InvalidEnv { name: "PORT", .. })
        ));
    }

    #[test]
    fn invalid_timeout_fails() {
        let result = "not-a-number"
            .trim()
            .parse::<u64>()
            .map_err(|e| ConfigError::InvalidEnv {
                name: "ML_SIDECAR_TIMEOUT_SECONDS",
                reason: e.to_string(),
            });
        assert!(matches!(
            result,
            Err(ConfigError::InvalidEnv {
                name: "ML_SIDECAR_TIMEOUT_SECONDS",
                ..
            })
        ));
    }

    #[test]
    fn invalid_database_max_connections_fails() {
        let result = "not-a-number"
            .trim()
            .parse::<u32>()
            .map_err(|e| ConfigError::InvalidEnv {
                name: "DATABASE_MAX_CONNECTIONS",
                reason: e.to_string(),
            });
        assert!(matches!(
            result,
            Err(ConfigError::InvalidEnv {
                name: "DATABASE_MAX_CONNECTIONS",
                ..
            })
        ));
    }
}
