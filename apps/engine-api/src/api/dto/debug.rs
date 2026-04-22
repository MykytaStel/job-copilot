use serde::Serialize;

use crate::services::search_ranking::runtime::TrainedRerankerAvailability;
use crate::state::AppState;

#[derive(Debug, Serialize)]
pub struct RerankerStatusResponse {
    pub reranker_mode_requested: String,
    pub learned_reranker_enabled: bool,
    pub trained_reranker_enabled: bool,
    pub trained_reranker_availability: String,
    pub trained_reranker_model_loaded: bool,
}

impl RerankerStatusResponse {
    pub fn from_state(state: &AppState) -> Self {
        Self {
            reranker_mode_requested: state.reranker_runtime_mode.as_str().to_string(),
            learned_reranker_enabled: state.learned_reranker_enabled,
            trained_reranker_enabled: state.trained_reranker_enabled,
            trained_reranker_availability: trained_reranker_availability_value(
                &state.trained_reranker_availability,
            )
            .to_string(),
            trained_reranker_model_loaded: state.trained_reranker_model.is_some(),
        }
    }
}

fn trained_reranker_availability_value(availability: &TrainedRerankerAvailability) -> &'static str {
    availability.as_str()
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::RerankerStatusResponse;
    use crate::services::search_ranking::runtime::{
        RerankerRuntimeMode, TrainedRerankerAvailability,
    };
    use crate::state::AppState;

    #[test]
    fn serializes_safe_reranker_status_fields() {
        let state = AppState::without_database()
            .with_learned_reranker_enabled(false)
            .with_trained_reranker(true, None)
            .with_trained_reranker_availability(TrainedRerankerAvailability::InvalidArtifact(
                "bad artifact".to_string(),
            ))
            .with_reranker_runtime_mode(RerankerRuntimeMode::Trained);

        let payload = serde_json::to_value(RerankerStatusResponse::from_state(&state))
            .expect("status response should serialize");

        assert_eq!(payload["reranker_mode_requested"], json!("trained"));
        assert_eq!(payload["learned_reranker_enabled"], json!(false));
        assert_eq!(payload["trained_reranker_enabled"], json!(true));
        assert_eq!(
            payload["trained_reranker_availability"],
            json!("invalid_artifact")
        );
        assert_eq!(payload["trained_reranker_model_loaded"], json!(false));
        assert_eq!(payload.get("model_path"), None);
    }
}
