use std::time::{Duration, Instant};

use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

#[cfg(test)]
#[path = "cv_tailoring/stub.rs"]
mod stub;

#[cfg(test)]
use std::sync::Arc;

#[cfg(test)]
pub use stub::CvTailoringServiceStub;

#[derive(Clone)]
pub struct CvTailoringService {
    backend: CvTailoringBackend,
}

#[derive(Clone)]
enum CvTailoringBackend {
    Http {
        client: reqwest::Client,
        base_url: String,
    },
    #[cfg(test)]
    Stub(Arc<CvTailoringServiceStub>),
}

#[derive(Clone, Debug, Serialize)]
pub struct CvTailoringMlRequest {
    pub profile_id: String,
    pub job_id: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub profile_summary: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub candidate_skills: Vec<String>,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub job_title: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub job_description: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub job_required_skills: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub job_nice_to_have_skills: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub candidate_cv_text: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CvTailoringGapItem {
    pub skill: String,
    pub suggestion: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CvTailoringSuggestions {
    #[serde(default)]
    pub skills_to_highlight: Vec<String>,
    #[serde(default)]
    pub skills_to_mention: Vec<String>,
    #[serde(default)]
    pub gaps_to_address: Vec<CvTailoringGapItem>,
    #[serde(default)]
    pub summary_rewrite: String,
    #[serde(default)]
    pub key_phrases: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CvTailoringMlResponse {
    pub suggestions: CvTailoringSuggestions,
    pub provider: String,
    pub generated_at: String,
}

#[derive(Debug)]
pub enum CvTailoringError {
    Http(String),
    Upstream { status: StatusCode, detail: String },
    Decode(String),
}

impl std::fmt::Display for CvTailoringError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Http(detail) => f.write_str(detail),
            Self::Upstream { status, detail } => {
                write!(f, "ml sidecar returned {status}: {detail}")
            }
            Self::Decode(detail) => f.write_str(detail),
        }
    }
}

impl std::error::Error for CvTailoringError {}

impl CvTailoringService {
    pub fn new(base_url: String, timeout_seconds: u64) -> Result<Self, reqwest::Error> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(timeout_seconds.max(1)))
            .build()?;
        Ok(Self {
            backend: CvTailoringBackend::Http {
                client,
                base_url: base_url.trim_end_matches('/').to_string(),
            },
        })
    }

    pub async fn tailor(
        &self,
        payload: &CvTailoringMlRequest,
    ) -> Result<CvTailoringMlResponse, CvTailoringError> {
        match &self.backend {
            CvTailoringBackend::Http { client, base_url } => {
                let url = format!("{}/api/v1/cv-tailoring", base_url);
                let started = Instant::now();
                let response = client
                    .post(&url)
                    .json(payload)
                    .send()
                    .await
                    .map_err(|error| {
                        warn!(
                            profile_id = %payload.profile_id,
                            job_id = %payload.job_id,
                            error = %error,
                            "cv tailoring HTTP call failed"
                        );
                        CvTailoringError::Http(error.to_string())
                    })?;

                let status = response.status();
                let latency_ms = started.elapsed().as_millis();

                if !status.is_success() {
                    let detail = response
                        .text()
                        .await
                        .unwrap_or_else(|_| "unknown ml sidecar error".to_string());
                    warn!(
                        profile_id = %payload.profile_id,
                        job_id = %payload.job_id,
                        %status,
                        latency_ms,
                        "cv tailoring ml returned error"
                    );
                    return Err(CvTailoringError::Upstream { status, detail });
                }

                info!(
                    profile_id = %payload.profile_id,
                    job_id = %payload.job_id,
                    status = %status.as_u16(),
                    latency_ms,
                    "cv tailoring call completed"
                );

                response
                    .json::<CvTailoringMlResponse>()
                    .await
                    .map_err(|error| CvTailoringError::Decode(error.to_string()))
            }
            #[cfg(test)]
            CvTailoringBackend::Stub(stub) => stub.tailor(payload),
        }
    }

    #[cfg(test)]
    pub fn for_tests(stub: CvTailoringServiceStub) -> Self {
        Self {
            backend: CvTailoringBackend::Stub(Arc::new(stub)),
        }
    }
}
