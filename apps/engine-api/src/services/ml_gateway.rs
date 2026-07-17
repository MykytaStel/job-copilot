use std::time::Duration;

use reqwest::StatusCode;
use serde_json::Value;

const INTERNAL_TOKEN_HEADER: &str = "X-Internal-Token";

#[derive(Clone)]
pub struct MlGatewayService {
    client: reqwest::Client,
    base_url: String,
    internal_token: Option<String>,
}

#[derive(Debug)]
pub enum MlGatewayError {
    Http(String),
    Upstream { status: StatusCode, detail: String },
    Decode(String),
}

impl std::fmt::Display for MlGatewayError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Http(detail) | Self::Decode(detail) => formatter.write_str(detail),
            Self::Upstream { status, detail } => {
                write!(formatter, "ml sidecar returned {status}: {detail}")
            }
        }
    }
}

impl std::error::Error for MlGatewayError {}

impl MlGatewayService {
    pub fn new(
        base_url: String,
        timeout_seconds: u64,
        internal_token: Option<String>,
    ) -> Result<Self, reqwest::Error> {
        Ok(Self {
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(timeout_seconds.max(1)))
                .build()?,
            base_url: base_url.trim_end_matches('/').to_string(),
            internal_token,
        })
    }

    pub async fn post_json(&self, path: &str, payload: &Value) -> Result<Value, MlGatewayError> {
        let mut request = self
            .client
            .post(format!("{}{}", self.base_url, path))
            .json(payload);
        if let Some(token) = self
            .internal_token
            .as_deref()
            .filter(|token| !token.is_empty())
        {
            request = request.header(INTERNAL_TOKEN_HEADER, token);
        }

        let response = request
            .send()
            .await
            .map_err(|error| MlGatewayError::Http(error.to_string()))?;
        let status = response.status();
        if !status.is_success() {
            let detail = response
                .text()
                .await
                .unwrap_or_else(|_| "unknown ml sidecar error".to_string());
            return Err(MlGatewayError::Upstream { status, detail });
        }

        response
            .json::<Value>()
            .await
            .map_err(|error| MlGatewayError::Decode(error.to_string()))
    }

    pub async fn get_json(&self, path: &str) -> Result<Value, MlGatewayError> {
        let mut request = self.client.get(format!("{}{}", self.base_url, path));
        if let Some(token) = self
            .internal_token
            .as_deref()
            .filter(|token| !token.is_empty())
        {
            request = request.header(INTERNAL_TOKEN_HEADER, token);
        }

        let response = request
            .send()
            .await
            .map_err(|error| MlGatewayError::Http(error.to_string()))?;
        let status = response.status();
        if !status.is_success() {
            let detail = response
                .text()
                .await
                .unwrap_or_else(|_| "unknown ml sidecar error".to_string());
            return Err(MlGatewayError::Upstream { status, detail });
        }

        response
            .json::<Value>()
            .await
            .map_err(|error| MlGatewayError::Decode(error.to_string()))
    }

    pub async fn post_no_content(&self, path: &str, payload: &Value) -> Result<(), MlGatewayError> {
        let mut request = self
            .client
            .post(format!("{}{}", self.base_url, path))
            .json(payload);
        if let Some(token) = self
            .internal_token
            .as_deref()
            .filter(|token| !token.is_empty())
        {
            request = request.header(INTERNAL_TOKEN_HEADER, token);
        }

        let response = request
            .send()
            .await
            .map_err(|error| MlGatewayError::Http(error.to_string()))?;
        let status = response.status();
        if status.is_success() {
            Ok(())
        } else {
            let detail = response
                .text()
                .await
                .unwrap_or_else(|_| "unknown ml sidecar error".to_string());
            Err(MlGatewayError::Upstream { status, detail })
        }
    }
}
