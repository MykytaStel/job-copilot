use axum::Json;
use axum::extract::State;

use crate::api::dto::debug::RerankerStatusResponse;
use crate::state::AppState;

pub async fn get_reranker_status(State(state): State<AppState>) -> Json<RerankerStatusResponse> {
    Json(RerankerStatusResponse::from_state(&state))
}

#[cfg(test)]
mod tests {
    use axum::body;
    use axum::extract::State;
    use axum::response::IntoResponse;
    use serde_json::{Value, json};

    use super::get_reranker_status;
    use crate::services::search_ranking::runtime::{
        RerankerRuntimeMode, TrainedRerankerAvailability,
    };
    use crate::services::trained_reranker::TrainedRerankerModel;
    use crate::state::AppState;

    fn trained_reranker_model() -> TrainedRerankerModel {
        TrainedRerankerModel::from_json_str(
            r#"{
              "artifact_version": "trained_reranker_v3",
              "model_type": "logistic_regression",
              "feature_names": ["deterministic_score"],
              "weights": {
                "deterministic_score": 1.0
              },
              "intercept": 0.0,
              "max_score_delta": 8
            }"#,
        )
        .expect("valid trained reranker artifact")
    }

    async fn response_payload(state: AppState) -> Value {
        let response = get_reranker_status(State(state)).await.into_response();
        let body = body::to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("response body should be readable");

        serde_json::from_slice(&body).expect("response body should be valid json")
    }

    #[tokio::test]
    async fn reports_loaded_trained_model_status() {
        let payload = response_payload(
            AppState::without_database()
                .with_trained_reranker(true, Some(trained_reranker_model()))
                .with_reranker_runtime_mode(RerankerRuntimeMode::Trained),
        )
        .await;

        assert_eq!(payload["reranker_mode_requested"], json!("trained"));
        assert_eq!(payload["trained_reranker_enabled"], json!(true));
        assert_eq!(payload["trained_reranker_availability"], json!("ready"));
        assert_eq!(payload["trained_reranker_model_loaded"], json!(true));
        assert_eq!(payload["learned_reranker_enabled"], json!(true));
    }

    #[tokio::test]
    async fn reports_missing_trained_model_path_status() {
        let payload = response_payload(
            AppState::without_database()
                .with_trained_reranker(true, None)
                .with_reranker_runtime_mode(RerankerRuntimeMode::Trained),
        )
        .await;

        assert_eq!(payload["trained_reranker_enabled"], json!(true));
        assert_eq!(
            payload["trained_reranker_availability"],
            json!("missing_path")
        );
        assert_eq!(payload["trained_reranker_model_loaded"], json!(false));
    }

    #[tokio::test]
    async fn reports_invalid_trained_artifact_status() {
        let payload = response_payload(
            AppState::without_database()
                .with_trained_reranker(true, None)
                .with_trained_reranker_availability(TrainedRerankerAvailability::InvalidArtifact(
                    "bad json".to_string(),
                ))
                .with_reranker_runtime_mode(RerankerRuntimeMode::Trained),
        )
        .await;

        assert_eq!(payload["trained_reranker_enabled"], json!(true));
        assert_eq!(
            payload["trained_reranker_availability"],
            json!("invalid_artifact")
        );
        assert_eq!(payload["trained_reranker_model_loaded"], json!(false));
        assert_eq!(payload.get("error"), None);
    }

    #[tokio::test]
    async fn reports_requested_mode_for_deterministic_and_learned_runtimes() {
        let deterministic = response_payload(
            AppState::without_database()
                .with_reranker_runtime_mode(RerankerRuntimeMode::Deterministic),
        )
        .await;
        let learned = response_payload(
            AppState::without_database().with_reranker_runtime_mode(RerankerRuntimeMode::Learned),
        )
        .await;

        assert_eq!(
            deterministic["reranker_mode_requested"],
            json!("deterministic")
        );
        assert_eq!(learned["reranker_mode_requested"], json!("learned"));
        assert_eq!(
            deterministic["trained_reranker_availability"],
            json!("disabled_by_flag")
        );
        assert_eq!(learned["learned_reranker_enabled"], json!(true));
    }
}
