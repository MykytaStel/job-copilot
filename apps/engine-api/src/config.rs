use std::fmt;
use std::str::FromStr;

use thiserror::Error;
use tracing::warn;

use crate::services::search_ranking::runtime::RerankerRuntimeMode;

#[derive(Clone, Debug)]
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

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ConfigError {
    #[error("missing required environment variable: {0}")]
    MissingEnv(&'static str),

    #[error("invalid environment variable {name}: {reason}")]
    InvalidEnv { name: &'static str, reason: String },

    #[error("production configuration error: {0}")]
    Production(String),
}

impl Config {
    pub fn is_production(&self) -> bool {
        self.app_env == "production"
    }

    pub fn from_env() -> Result<Self, ConfigError> {
        Self::from_env_vars(|name| std::env::var(name).ok())
    }

    fn from_env_vars(get_env: impl Fn(&str) -> Option<String>) -> Result<Self, ConfigError> {
        let port = parse_or_default(&get_env, "PORT", 8080_u16)?;

        let database_url = optional_trimmed(&get_env, "DATABASE_URL");

        let database_max_connections =
            parse_or_default(&get_env, "DATABASE_MAX_CONNECTIONS", 5_u32)?;

        let run_db_migrations = parse_bool_or_default(&get_env, "RUN_DB_MIGRATIONS", true)?;

        let learned_reranker_enabled =
            parse_bool_or_default(&get_env, "LEARNED_RERANKER_ENABLED", true)?;

        let trained_reranker_enabled =
            parse_bool_or_default(&get_env, "TRAINED_RERANKER_ENABLED", false)?;

        let reranker_runtime_mode = optional_trimmed(&get_env, "RERANKER_RUNTIME_MODE")
            .and_then(|value| {
                let parsed = RerankerRuntimeMode::parse(&value);

                if parsed.is_none() {
                    warn!(
                        mode = value,
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

        let trained_reranker_model_path = optional_trimmed(&get_env, "TRAINED_RERANKER_MODEL_PATH");

        let ml_sidecar_base_url = optional_trimmed(&get_env, "ML_SIDECAR_BASE_URL")
            .unwrap_or_else(|| "http://localhost:8000".to_string());

        let ml_sidecar_timeout_seconds =
            parse_or_default(&get_env, "ML_SIDECAR_TIMEOUT_SECONDS", 90_u64)?;

        let ml_sidecar_internal_token = optional_trimmed(&get_env, "ML_INTERNAL_TOKEN");

        let ml_retrain_threshold = parse_or_default(&get_env, "ML_RETRAIN_THRESHOLD", 15_usize)?;

        let ml_retrain_poll_interval_seconds =
            parse_or_default(&get_env, "ML_RETRAIN_POLL_INTERVAL_SECONDS", 21_600_u64)?;

        let jwt_secret = optional_trimmed(&get_env, "JWT_SECRET");

        let app_env = optional_trimmed(&get_env, "APP_ENV")
            .map(|value| value.to_ascii_lowercase())
            .unwrap_or_else(|| "development".to_string());

        let is_production = app_env == "production";

        let cors_allowed_origins = parse_cors_allowed_origins(&get_env, is_production)?;

        let metrics_port = parse_or_default(&get_env, "METRICS_PORT", 9090_u16)?;

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
        validate_positive_u32("DATABASE_MAX_CONNECTIONS", self.database_max_connections)?;
        validate_positive_u64(
            "ML_SIDECAR_TIMEOUT_SECONDS",
            self.ml_sidecar_timeout_seconds,
        )?;
        validate_positive_u64(
            "ML_RETRAIN_POLL_INTERVAL_SECONDS",
            self.ml_retrain_poll_interval_seconds,
        )?;

        validate_http_url("ML_SIDECAR_BASE_URL", &self.ml_sidecar_base_url)?;

        if self.is_production() && !is_production_jwt_secret_acceptable(self.jwt_secret.as_deref())
        {
            return Err(ConfigError::Production(
                "JWT_SECRET must be set in production — refusing to start without authentication"
                    .to_string(),
            ));
        }

        if self.is_production() && self.ml_sidecar_internal_token.is_none() {
            return Err(ConfigError::Production(
                "ML_INTERNAL_TOKEN must be set in production — refusing to start without ML service authentication"
                    .to_string(),
            ));
        }

        Ok(())
    }
}

fn optional_trimmed(
    get_env: &impl Fn(&str) -> Option<String>,
    name: &'static str,
) -> Option<String> {
    get_env(name)
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn parse_or_default<T>(
    get_env: &impl Fn(&str) -> Option<String>,
    name: &'static str,
    default: T,
) -> Result<T, ConfigError>
where
    T: FromStr,
    T::Err: fmt::Display,
{
    let Some(raw_value) = optional_trimmed(get_env, name) else {
        return Ok(default);
    };

    raw_value
        .parse::<T>()
        .map_err(|error| ConfigError::InvalidEnv {
            name,
            reason: error.to_string(),
        })
}

fn parse_bool_or_default(
    get_env: &impl Fn(&str) -> Option<String>,
    name: &'static str,
    default: bool,
) -> Result<bool, ConfigError> {
    let Some(raw_value) = optional_trimmed(get_env, name) else {
        return Ok(default);
    };

    parse_bool(&raw_value).ok_or_else(|| ConfigError::InvalidEnv {
        name,
        reason: "expected one of: true, false, 1, 0, yes, no, on, off".to_string(),
    })
}

fn parse_bool(value: &str) -> Option<bool> {
    match value.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Some(true),
        "0" | "false" | "no" | "off" => Some(false),
        _ => None,
    }
}

fn parse_cors_allowed_origins(
    get_env: &impl Fn(&str) -> Option<String>,
    is_production: bool,
) -> Result<Vec<String>, ConfigError> {
    let origins = optional_trimmed(get_env, "CORS_ALLOWED_ORIGINS")
        .map(|value| {
            value
                .split(',')
                .map(|origin| origin.trim().to_string())
                .filter(|origin| !origin.is_empty())
                .collect::<Vec<_>>()
        })
        .filter(|origins| !origins.is_empty());

    match origins {
        Some(origins) => Ok(origins),
        None if is_production => Err(ConfigError::MissingEnv("CORS_ALLOWED_ORIGINS")),
        None => Ok(vec![
            "http://localhost:3000".to_string(),
            "http://127.0.0.1:3000".to_string(),
            "http://localhost:5173".to_string(),
            "http://127.0.0.1:5173".to_string(),
        ]),
    }
}

fn validate_positive_u32(name: &'static str, value: u32) -> Result<(), ConfigError> {
    if value == 0 {
        return Err(ConfigError::InvalidEnv {
            name,
            reason: "must be greater than 0".to_string(),
        });
    }

    Ok(())
}

fn validate_positive_u64(name: &'static str, value: u64) -> Result<(), ConfigError> {
    if value == 0 {
        return Err(ConfigError::InvalidEnv {
            name,
            reason: "must be greater than 0".to_string(),
        });
    }

    Ok(())
}

fn validate_http_url(name: &'static str, value: &str) -> Result<(), ConfigError> {
    let parsed = reqwest::Url::parse(value).map_err(|error| ConfigError::InvalidEnv {
        name,
        reason: error.to_string(),
    })?;

    match parsed.scheme() {
        "http" | "https" => Ok(()),
        scheme => Err(ConfigError::InvalidEnv {
            name,
            reason: format!("expected http or https URL, got scheme '{scheme}'"),
        }),
    }
}

fn is_production_jwt_secret_acceptable(secret: Option<&str>) -> bool {
    let Some(secret) = secret.map(str::trim).filter(|value| !value.is_empty()) else {
        return false;
    };

    secret != "local-dev-secret-change-me"
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::services::search_ranking::runtime::RerankerRuntimeMode;

    use super::{Config, ConfigError, is_production_jwt_secret_acceptable, parse_bool};

    fn load_config(vars: &[(&str, &str)]) -> Result<Config, ConfigError> {
        let env = vars
            .iter()
            .map(|(key, value)| ((*key).to_string(), (*value).to_string()))
            .collect::<HashMap<_, _>>();

        Config::from_env_vars(|name| env.get(name).cloned())
    }

    #[test]
    fn parses_truthy_and_falsey_booleans() {
        assert_eq!(parse_bool("true"), Some(true));
        assert_eq!(parse_bool("1"), Some(true));
        assert_eq!(parse_bool("yes"), Some(true));
        assert_eq!(parse_bool("on"), Some(true));

        assert_eq!(parse_bool("false"), Some(false));
        assert_eq!(parse_bool("0"), Some(false));
        assert_eq!(parse_bool("no"), Some(false));
        assert_eq!(parse_bool("off"), Some(false));

        assert_eq!(parse_bool("maybe"), None);
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
    fn valid_development_config_uses_defaults() {
        let config = load_config(&[]).expect("development config should load");

        assert_eq!(config.port, 8080);
        assert_eq!(config.app_env, "development");
        assert_eq!(config.database_max_connections, 5);
        assert_eq!(config.ml_sidecar_base_url, "http://localhost:8000");
        assert_eq!(config.ml_sidecar_timeout_seconds, 90);
        assert_eq!(config.metrics_port, 9090);
        assert_eq!(
            config.cors_allowed_origins,
            vec![
                "http://localhost:3000",
                "http://127.0.0.1:3000",
                "http://localhost:5173",
                "http://127.0.0.1:5173",
            ]
        );
    }

    #[test]
    fn invalid_port_fails() {
        let error = load_config(&[("PORT", "not-a-port")]).expect_err("invalid port should fail");

        assert!(matches!(
            error,
            ConfigError::InvalidEnv { name: "PORT", .. }
        ));
    }

    #[test]
    fn zero_database_max_connections_fails() {
        let error = load_config(&[("DATABASE_MAX_CONNECTIONS", "0")])
            .expect_err("zero max connections should fail");

        assert_eq!(
            error,
            ConfigError::InvalidEnv {
                name: "DATABASE_MAX_CONNECTIONS",
                reason: "must be greater than 0".to_string(),
            }
        );
    }

    #[test]
    fn zero_ml_sidecar_timeout_fails() {
        let error = load_config(&[("ML_SIDECAR_TIMEOUT_SECONDS", "0")])
            .expect_err("zero timeout should fail");

        assert_eq!(
            error,
            ConfigError::InvalidEnv {
                name: "ML_SIDECAR_TIMEOUT_SECONDS",
                reason: "must be greater than 0".to_string(),
            }
        );
    }

    #[test]
    fn invalid_ml_sidecar_url_fails() {
        let error = load_config(&[("ML_SIDECAR_BASE_URL", "not-a-url")])
            .expect_err("invalid url should fail");

        assert!(matches!(
            error,
            ConfigError::InvalidEnv {
                name: "ML_SIDECAR_BASE_URL",
                ..
            }
        ));
    }

    #[test]
    fn production_requires_cors_allowed_origins() {
        let error = load_config(&[
            ("APP_ENV", "production"),
            ("JWT_SECRET", "a-real-production-secret"),
            ("ML_INTERNAL_TOKEN", "internal-token"),
        ])
        .expect_err("production without CORS_ALLOWED_ORIGINS should fail");

        assert_eq!(error, ConfigError::MissingEnv("CORS_ALLOWED_ORIGINS"));
    }

    #[test]
    fn production_requires_safe_jwt_secret() {
        let error = load_config(&[
            ("APP_ENV", "production"),
            ("CORS_ALLOWED_ORIGINS", "https://example.com"),
            ("ML_INTERNAL_TOKEN", "internal-token"),
            ("JWT_SECRET", "local-dev-secret-change-me"),
        ])
        .expect_err("placeholder JWT secret should fail in production");

        assert!(matches!(error, ConfigError::Production(_)));
    }

    #[test]
    fn production_requires_ml_internal_token() {
        let error = load_config(&[
            ("APP_ENV", "production"),
            ("CORS_ALLOWED_ORIGINS", "https://example.com"),
            ("JWT_SECRET", "a-real-production-secret"),
        ])
        .expect_err("missing ML internal token should fail in production");

        assert!(matches!(error, ConfigError::Production(_)));
    }

    #[test]
    fn valid_production_config_passes() {
        let config = load_config(&[
            ("APP_ENV", "production"),
            ("CORS_ALLOWED_ORIGINS", "https://example.com"),
            ("JWT_SECRET", "a-real-production-secret"),
            ("ML_INTERNAL_TOKEN", "internal-token"),
            ("ML_SIDECAR_BASE_URL", "https://ml.example.com"),
        ])
        .expect("valid production config should pass");

        assert!(config.is_production());
        assert_eq!(config.cors_allowed_origins, vec!["https://example.com"]);
    }
}
