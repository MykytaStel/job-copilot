use thiserror::Error;

#[derive(Debug, Error)]
pub enum IngestionError {
    #[error("configuration error: {0}")]
    Config(String),

    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("scraper error: {0}")]
    Scraper(String),

    #[error("normalization error: {0}")]
    Normalization(String),

    #[error("ingestion input error: {0}")]
    Input(String),

    #[error("validation error: {0}")]
    Validation(String),

    #[error("{0}")]
    Message(String),
}

pub type Result<T> = std::result::Result<T, IngestionError>;

impl From<String> for IngestionError {
    fn from(value: String) -> Self {
        Self::Message(value)
    }
}

impl From<&str> for IngestionError {
    fn from(value: &str) -> Self {
        Self::Message(value.to_owned())
    }
}
