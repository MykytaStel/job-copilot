use std::time::Duration;

use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

#[cfg(test)]
#[path = "reranker_bootstrap/stub.rs"]
mod stub;

#[cfg(test)]
use std::sync::Arc;

#[cfg(test)]
pub use stub::RerankerBootstrapServiceStub;

#[derive(Clone)]
pub struct RerankerBootstrapService {
    backend: RerankerBootstrapBackend,
}

#[derive(Clone)]
enum RerankerBootstrapBackend {
    Http {
        client: reqwest::Client,
        base_url: String,
    },
    #[cfg(test)]
    Stub(Arc<RerankerBootstrapServiceStub>),
}

#[derive(Clone, Debug, Serialize)]
pub struct BootstrapRequestPayload {
    pub profile_id: String,
    pub min_examples: usize,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct BootstrapTrainingSummary {
    pub example_count: usize,
    pub positive_count: usize,
    pub medium_count: usize,
    pub negative_count: usize,
    pub saved_only_count: usize,
    pub viewed_only_count: usize,
    pub medium_default_count: usize,
    pub epochs: usize,
    pub learning_rate: f64,
    pub l2: f64,
    pub loss: f64,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct BootstrapVariantMetrics {
    pub variant: String,
    pub top_n: usize,
    pub ordered_job_ids: Vec<String>,
    pub top_k_positives: usize,
    pub average_label_score_top_n: f64,
    pub average_training_weight_top_n: f64,
    pub positive_hit_rate: f64,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct BootstrapEvaluationSummary {
    pub profile_id: String,
    pub label_policy_version: String,
    pub signal_weight_policy_version: String,
    pub split_method: String,
    pub example_count: usize,
    pub train_example_count: usize,
    pub test_example_count: usize,
    pub positive_count: usize,
    pub top_n: usize,
    pub variants: Vec<BootstrapVariantMetrics>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct BootstrapBenchmarkSummary {
    pub baseline_model_type: String,
    pub candidate_model_type: String,
    pub baseline_positive_hit_rate: f64,
    pub candidate_positive_hit_rate: f64,
    pub candidate_available: bool,
    pub winner: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct BootstrapResponsePayload {
    pub retrained: bool,
    pub example_count: usize,
    pub reason: Option<String>,
    pub model_path: Option<String>,
    pub artifact_version: Option<String>,
    pub model_type: Option<String>,
    pub training: Option<BootstrapTrainingSummary>,
    pub evaluation: Option<BootstrapEvaluationSummary>,
    pub benchmark: Option<BootstrapBenchmarkSummary>,
    pub feature_importances: Option<std::collections::BTreeMap<String, f64>>,
}

#[derive(Clone, Debug)]
pub enum RerankerBootstrapError {
    Http(String),
    Upstream { status: StatusCode, detail: String },
    Decode(String),
}

impl std::fmt::Display for RerankerBootstrapError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Http(detail) => formatter.write_str(detail),
            Self::Upstream { status, detail } => {
                write!(formatter, "ml sidecar returned {status}: {detail}")
            }
            Self::Decode(detail) => formatter.write_str(detail),
        }
    }
}

impl std::error::Error for RerankerBootstrapError {}

impl RerankerBootstrapService {
    pub fn new(base_url: String, timeout_seconds: u64) -> Result<Self, reqwest::Error> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(timeout_seconds.max(1)))
            .build()?;

        Ok(Self {
            backend: RerankerBootstrapBackend::Http {
                client,
                base_url: base_url.trim_end_matches('/').to_string(),
            },
        })
    }

    pub async fn bootstrap(
        &self,
        payload: &BootstrapRequestPayload,
    ) -> Result<BootstrapResponsePayload, RerankerBootstrapError> {
        match &self.backend {
            RerankerBootstrapBackend::Http { client, base_url } => {
                let url = format!("{}/api/v1/reranker/bootstrap", base_url);
                let response = client
                    .post(url)
                    .json(payload)
                    .send()
                    .await
                    .map_err(|error| RerankerBootstrapError::Http(error.to_string()))?;

                if !response.status().is_success() {
                    let status = response.status();
                    let detail = response
                        .text()
                        .await
                        .unwrap_or_else(|_| "unknown ml sidecar error".to_string());
                    return Err(RerankerBootstrapError::Upstream { status, detail });
                }

                response
                    .json::<BootstrapResponsePayload>()
                    .await
                    .map_err(|error| RerankerBootstrapError::Decode(error.to_string()))
            }
            #[cfg(test)]
            RerankerBootstrapBackend::Stub(stub) => stub.bootstrap(payload),
        }
    }

    #[cfg(test)]
    pub fn for_tests(stub: RerankerBootstrapServiceStub) -> Self {
        Self {
            backend: RerankerBootstrapBackend::Stub(Arc::new(stub)),
        }
    }
}
