pub struct Config {
    pub port: u16,
    pub database_url: Option<String>,
    pub database_max_connections: u32,
    pub run_db_migrations: bool,
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

        Self {
            port,
            database_url,
            database_max_connections,
            run_db_migrations,
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
}
