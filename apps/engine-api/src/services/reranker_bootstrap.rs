use std::time::{Duration, Instant};

use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

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
    pub ndcg_at_top_n: f64,
    pub mrr_at_top_n: f64,
    pub map_at_top_n: f64,
    pub precision_at_3: f64,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct BootstrapEvaluationSummary {
    pub profile_id: String,
    pub label_policy_version: String,
    #[serde(default)]
    pub metrics_version: String,
    pub signal_weight_policy_version: String,
    pub split_method: String,
    pub example_count: usize,
    pub train_example_count: usize,
    pub test_example_count: usize,
    pub positive_count: usize,
    pub top_n: usize,
    #[serde(default)]
    pub rolling_window_count: usize,
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
    pub feature_set_winner: Option<String>,
    pub ablated_positive_hit_rate: Option<f64>,
    #[serde(default)]
    pub ablation_fallback_used: bool,
    pub ablation_fallback_reason: Option<String>,
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
                let started = Instant::now();
                let response = client
                    .post(&url)
                    .json(payload)
                    .send()
                    .await
                    .map_err(|error| {
                        warn!(
                            profile_id = %payload.profile_id,
                            error = %error,
                            "ml bootstrap HTTP call failed"
                        );
                        RerankerBootstrapError::Http(error.to_string())
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
                        %status,
                        latency_ms,
                        "ml bootstrap returned error"
                    );
                    return Err(RerankerBootstrapError::Upstream { status, detail });
                }

                info!(
                    profile_id = %payload.profile_id,
                    status = %status.as_u16(),
                    latency_ms,
                    "ml bootstrap call completed"
                );

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

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::BootstrapResponsePayload;

    #[test]
    fn deserializes_full_variant_metrics_and_benchmark_metadata() {
        let payload = json!({
            "retrained": true,
            "example_count": 16,
            "reason": "insufficient temporal spread across examples",
            "model_path": "/tmp/model.json",
            "artifact_version": "trained_reranker_v3",
            "model_type": "logistic_regression",
            "training": {
                "example_count": 16,
                "positive_count": 5,
                "medium_count": 7,
                "negative_count": 4,
                "saved_only_count": 2,
                "viewed_only_count": 3,
                "medium_default_count": 0,
                "epochs": 10,
                "learning_rate": 0.1,
                "l2": 0.01,
                "loss": 0.5
            },
            "evaluation": {
                "profile_id": "profile-1",
                "label_policy_version": "outcome_label_v3",
                "metrics_version": "reranker_eval_v2",
                "signal_weight_policy_version": "outcome_signal_weight_v2",
                "split_method": "rolling_temporal",
                "example_count": 16,
                "train_example_count": 12,
                "test_example_count": 4,
                "positive_count": 5,
                "top_n": 10,
                "rolling_window_count": 2,
                "variants": [{
                    "variant": "trained_reranker_prediction",
                    "top_n": 10,
                    "ordered_job_ids": ["job-1"],
                    "top_k_positives": 1,
                    "average_label_score_top_n": 1.0,
                    "average_training_weight_top_n": 1.0,
                    "positive_hit_rate": 1.0,
                    "ndcg_at_top_n": 0.9,
                    "mrr_at_top_n": 0.8,
                    "map_at_top_n": 0.7,
                    "precision_at_3": 0.6
                }]
            },
            "benchmark": {
                "baseline_model_type": "logistic_regression",
                "candidate_model_type": "bpr",
                "baseline_positive_hit_rate": 0.5,
                "candidate_positive_hit_rate": 0.6,
                "candidate_available": true,
                "winner": "bpr",
                "feature_set_winner": "ablated_without_learned_scores",
                "ablated_positive_hit_rate": 0.55,
                "ablation_fallback_used": true,
                "ablation_fallback_reason": "no_low_variance_features_detected"
            },
            "feature_importances": {
                "matched_skill_count": 0.4
            }
        });

        let decoded: BootstrapResponsePayload =
            serde_json::from_value(payload).expect("payload should decode");

        let evaluation = decoded.evaluation.expect("evaluation should exist");
        assert_eq!(evaluation.metrics_version, "reranker_eval_v2");
        assert_eq!(evaluation.rolling_window_count, 2);
        assert_eq!(evaluation.variants[0].map_at_top_n, 0.7);
        assert_eq!(evaluation.variants[0].precision_at_3, 0.6);

        let benchmark = decoded.benchmark.expect("benchmark should exist");
        assert_eq!(
            benchmark.feature_set_winner.as_deref(),
            Some("ablated_without_learned_scores")
        );
        assert_eq!(benchmark.ablated_positive_hit_rate, Some(0.55));
        assert!(benchmark.ablation_fallback_used);
        assert_eq!(
            benchmark.ablation_fallback_reason.as_deref(),
            Some("no_low_variance_features_detected")
        );
        assert_eq!(
            decoded.reason.as_deref(),
            Some("insufficient temporal spread across examples")
        );
    }
}
