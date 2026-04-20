use std::collections::HashMap;
use std::fs;

use serde::Deserialize;

const ARTIFACT_VERSION: &str = "trained_reranker_v2";
const MODEL_TYPE: &str = "logistic_regression";

#[derive(Clone, Debug, PartialEq)]
pub struct TrainedRerankerModel {
    artifact: TrainedRerankerArtifact,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TrainedRerankerFeatures {
    pub deterministic_score: u8,
    pub behavior_score_delta: i16,
    pub behavior_score: u8,
    pub learned_reranker_score_delta: i16,
    pub learned_reranker_score: u8,
    pub matched_role_count: usize,
    pub matched_skill_count: usize,
    pub matched_keyword_count: usize,
    pub source_present: bool,
    pub role_family_present: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TrainedRerankerScore {
    pub score_delta: i16,
    pub reasons: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
struct TrainedRerankerArtifact {
    artifact_version: String,
    model_type: String,
    feature_names: Vec<String>,
    weights: HashMap<String, f64>,
    intercept: f64,
    max_score_delta: i16,
}

impl TrainedRerankerModel {
    pub fn from_json_str(payload: &str) -> Result<Self, String> {
        let artifact: TrainedRerankerArtifact =
            serde_json::from_str(payload).map_err(|error| error.to_string())?;

        if artifact.artifact_version != ARTIFACT_VERSION {
            return Err(format!(
                "unsupported artifact_version: {}",
                artifact.artifact_version
            ));
        }
        if artifact.model_type != MODEL_TYPE {
            return Err(format!("unsupported model_type: {}", artifact.model_type));
        }
        if artifact.feature_names.is_empty() {
            return Err("artifact must include at least one feature".to_string());
        }
        for feature_name in &artifact.feature_names {
            if !is_supported_feature(feature_name) {
                return Err(format!("unsupported feature: {feature_name}"));
            }
            if !artifact.weights.contains_key(feature_name) {
                return Err(format!("missing weight for feature: {feature_name}"));
            }
        }

        Ok(Self { artifact })
    }

    pub fn load(path: &str) -> Result<Self, String> {
        let payload = fs::read_to_string(path).map_err(|error| error.to_string())?;
        Self::from_json_str(&payload)
    }

    pub fn score(&self, features: &TrainedRerankerFeatures) -> TrainedRerankerScore {
        let logit = self.artifact.intercept
            + self
                .artifact
                .feature_names
                .iter()
                .map(|feature_name| {
                    self.artifact
                        .weights
                        .get(feature_name)
                        .copied()
                        .unwrap_or(0.0)
                        * feature_value(feature_name, features)
                })
                .sum::<f64>();
        let probability = sigmoid(logit);
        let max_delta = self.artifact.max_score_delta.clamp(1, 20);
        let score_delta = (((probability - 0.5) * 2.0 * f64::from(max_delta)).round() as i16)
            .clamp(-max_delta, max_delta);

        if score_delta == 0 {
            return TrainedRerankerScore {
                score_delta,
                reasons: Vec::new(),
            };
        }

        TrainedRerankerScore {
            score_delta,
            reasons: vec![format!(
                "Trained reranker v2 applied an inspectable model adjustment ({score_delta:+})"
            )],
        }
    }
}

fn feature_value(feature_name: &str, features: &TrainedRerankerFeatures) -> f64 {
    match feature_name {
        "deterministic_score" => f64::from(features.deterministic_score) / 100.0,
        "behavior_score_delta" => f64::from(features.behavior_score_delta.clamp(-25, 25)) / 25.0,
        "behavior_score" => f64::from(features.behavior_score) / 100.0,
        "learned_reranker_score_delta" => {
            f64::from(features.learned_reranker_score_delta.clamp(-25, 25)) / 25.0
        }
        "learned_reranker_score" => f64::from(features.learned_reranker_score) / 100.0,
        "matched_role_count" => features.matched_role_count.min(10) as f64 / 10.0,
        "matched_skill_count" => features.matched_skill_count.min(20) as f64 / 20.0,
        "matched_keyword_count" => features.matched_keyword_count.min(20) as f64 / 20.0,
        "source_present" => {
            if features.source_present {
                1.0
            } else {
                0.0
            }
        }
        "role_family_present" => {
            if features.role_family_present {
                1.0
            } else {
                0.0
            }
        }
        _ => 0.0,
    }
}

fn is_supported_feature(feature_name: &str) -> bool {
    matches!(
        feature_name,
        "deterministic_score"
            | "behavior_score_delta"
            | "behavior_score"
            | "learned_reranker_score_delta"
            | "learned_reranker_score"
            | "matched_role_count"
            | "matched_skill_count"
            | "matched_keyword_count"
            | "source_present"
            | "role_family_present"
    )
}

fn sigmoid(value: f64) -> f64 {
    if value >= 0.0 {
        let z = (-value).exp();
        return 1.0 / (1.0 + z);
    }

    let z = value.exp();
    z / (1.0 + z)
}

#[cfg(test)]
mod tests {
    use super::{TrainedRerankerFeatures, TrainedRerankerModel};

    fn artifact_json() -> &'static str {
        r#"{
          "artifact_version": "trained_reranker_v2",
          "model_type": "logistic_regression",
          "label_policy_version": "outcome_label_v2",
          "feature_names": ["deterministic_score", "matched_skill_count"],
          "feature_transforms": {},
          "weights": {
            "deterministic_score": 1.0,
            "matched_skill_count": 8.0
          },
          "intercept": -2.0,
          "max_score_delta": 8,
          "training": {
            "example_count": 3,
            "positive_count": 1,
            "medium_count": 1,
            "negative_count": 1,
            "epochs": 10,
            "learning_rate": 0.1,
            "l2": 0.0,
            "loss": 0.5
          }
        }"#
    }

    #[test]
    fn loads_json_artifact_and_scores_bounded_delta() {
        let model = TrainedRerankerModel::from_json_str(artifact_json()).expect("valid artifact");
        let score = model.score(&TrainedRerankerFeatures {
            deterministic_score: 80,
            behavior_score_delta: 0,
            behavior_score: 80,
            learned_reranker_score_delta: 0,
            learned_reranker_score: 80,
            matched_role_count: 0,
            matched_skill_count: 8,
            matched_keyword_count: 0,
            source_present: true,
            role_family_present: true,
        });

        assert!(score.score_delta > 0);
        assert!(score.score_delta <= 8);
        assert!(score.reasons[0].contains("Trained reranker v2"));
    }

    #[test]
    fn rejects_missing_feature_weight() {
        let error = TrainedRerankerModel::from_json_str(
            r#"{
              "artifact_version": "trained_reranker_v2",
              "model_type": "logistic_regression",
              "feature_names": ["deterministic_score"],
              "weights": {},
              "intercept": 0.0,
              "max_score_delta": 8
            }"#,
        )
        .expect_err("missing weight should be rejected");

        assert!(error.contains("missing weight"));
    }
}
