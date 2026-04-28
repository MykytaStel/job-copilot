use chrono::Utc;
use tracing::{error, info, warn};

use crate::domain::profile::ml::{CreateProfileMlMetric, UpdateProfileMlState};
use crate::services::reranker_bootstrap::{BootstrapRequestPayload, BootstrapResponsePayload};
use crate::state::AppState;

pub async fn run_retrain_cycle(state: &AppState, retrain_threshold: usize) {
    let candidates = match state
        .profile_ml_state
        .list_ready_for_retrain(retrain_threshold)
        .await
    {
        Ok(candidates) => candidates,
        Err(error) => {
            warn!(error = %error, "failed to list profiles ready for ML retrain");
            return;
        }
    };

    for candidate in candidates {
        let payload = BootstrapRequestPayload {
            profile_id: candidate.profile_id.clone(),
            min_examples: retrain_threshold,
        };
        match state.reranker_bootstrap.bootstrap(&payload).await {
            Ok(response) => {
                persist_bootstrap_response(state, &candidate.profile_id, response).await;
            }
            Err(error) => {
                warn!(
                    error = %error,
                    profile_id = candidate.profile_id,
                    "reranker bootstrap failed"
                );
                let _ = state
                    .profile_ml_state
                    .update_state(
                        &candidate.profile_id,
                        UpdateProfileMlState {
                            last_training_status: Some(Some("bootstrap_failed".to_string())),
                            ..UpdateProfileMlState::default()
                        },
                    )
                    .await;
                let _ = state
                    .profile_ml_metrics
                    .create(CreateProfileMlMetric {
                        profile_id: candidate.profile_id,
                        status: "bootstrap_failed".to_string(),
                        artifact_version: None,
                        model_type: None,
                        reason: Some(error.to_string()),
                        metrics_json: None,
                        training_json: None,
                        feature_importances_json: None,
                        benchmark_json: None,
                    })
                    .await;
            }
        }
    }
}

pub fn spawn_retrain_poller(state: AppState, retrain_threshold: usize, poll_interval_seconds: u64) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(
            poll_interval_seconds.max(60),
        ));
        interval.tick().await;

        loop {
            run_retrain_cycle(&state, retrain_threshold).await;
            interval.tick().await;
        }
    });
}

async fn persist_bootstrap_response(
    state: &AppState,
    profile_id: &str,
    response: BootstrapResponsePayload,
) {
    let status = if response.retrained {
        "trained".to_string()
    } else if response
        .reason
        .as_deref()
        .is_some_and(|detail| detail.contains("need at least"))
    {
        "skipped_insufficient_examples".to_string()
    } else {
        "skipped".to_string()
    };

    let metrics_json = response
        .evaluation
        .as_ref()
        .and_then(|value| serde_json::to_value(value).ok());
    let training_json = response
        .training
        .as_ref()
        .and_then(|value| serde_json::to_value(value).ok());
    let feature_importances_json = response
        .feature_importances
        .as_ref()
        .and_then(|value| serde_json::to_value(value).ok());
    let benchmark_json = response
        .benchmark
        .as_ref()
        .and_then(|value| serde_json::to_value(value).ok());

    if let Err(error) = state
        .profile_ml_metrics
        .create(CreateProfileMlMetric {
            profile_id: profile_id.to_string(),
            status: status.clone(),
            artifact_version: response.artifact_version.clone(),
            model_type: response.model_type.clone(),
            reason: response.reason.clone(),
            metrics_json,
            training_json,
            feature_importances_json,
            benchmark_json,
        })
        .await
    {
        error!(error = %error, profile_id, "failed to persist profile ML metric record");
    }

    let update = UpdateProfileMlState {
        last_retrained_at: if response.retrained {
            Some(Some(Utc::now().to_rfc3339()))
        } else {
            None
        },
        examples_since_retrain: response.retrained.then_some(0),
        last_artifact_version: response
            .artifact_version
            .clone()
            .map(Some)
            .or(Some(None).filter(|_| response.retrained)),
        last_training_status: Some(Some(status.clone())),
    };

    if let Err(error) = state
        .profile_ml_state
        .update_state(profile_id, update)
        .await
    {
        error!(error = %error, profile_id, "failed to update profile ML state");
        return;
    }

    info!(
        profile_id,
        status,
        retrained = response.retrained,
        "persisted reranker bootstrap result"
    );
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::domain::profile::ml::{ProfileMlMetricRecord, ProfileMlState};
    use crate::domain::profile::model::Profile;
    use crate::services::applications::{ApplicationsService, ApplicationsServiceStub};
    use crate::services::jobs::{JobsService, JobsServiceStub};
    use crate::services::profile_ml_metrics::{
        ProfileMlMetricsService, ProfileMlMetricsServiceStub,
    };
    use crate::services::profile_ml_state::{ProfileMlStateService, ProfileMlStateServiceStub};
    use crate::services::profiles::{ProfilesService, ProfilesServiceStub};
    use crate::services::reranker_bootstrap::{
        BootstrapEvaluationSummary, BootstrapResponsePayload, BootstrapTrainingSummary,
        BootstrapVariantMetrics, RerankerBootstrapError, RerankerBootstrapService,
        RerankerBootstrapServiceStub,
    };
    use crate::services::resumes::{ResumesService, ResumesServiceStub};
    use crate::state::AppState;

    use super::run_retrain_cycle;

    fn sample_profile() -> Profile {
        Profile {
            id: "profile-1".to_string(),
            name: "Jane Doe".to_string(),
            email: "jane@example.com".to_string(),
            location: None,
            raw_text: "Senior backend engineer".to_string(),
            analysis: None,
            years_of_experience: None,
            salary_min: None,
            salary_max: None,
            salary_currency: "USD".to_string(),
            languages: vec![],
            preferred_locations: vec![],
            experience: vec![],
            work_mode_preference: "any".to_string(),
            preferred_language: None,
            search_preferences: None,
            created_at: "2026-04-14T00:00:00Z".to_string(),
            updated_at: "2026-04-14T00:00:00Z".to_string(),
            skills_updated_at: None,
            portfolio_url: None,
            github_url: None,
            linkedin_url: None,
        }
    }

    #[tokio::test]
    async fn retrain_cycle_resets_counter_after_successful_bootstrap() {
        let metrics_stub = ProfileMlMetricsServiceStub::default();
        let state = AppState::for_services(
            ProfilesService::for_tests(
                ProfilesServiceStub::default().with_profile(sample_profile()),
            ),
            JobsService::for_tests(JobsServiceStub::default()),
            ApplicationsService::for_tests(ApplicationsServiceStub::default()),
            ResumesService::for_tests(ResumesServiceStub::default()),
        )
        .with_profile_ml_state_service(ProfileMlStateService::for_tests(
            ProfileMlStateServiceStub::default().with_state(ProfileMlState {
                profile_id: "profile-1".to_string(),
                last_retrained_at: None,
                examples_since_retrain: 16,
                last_artifact_version: None,
                last_training_status: None,
            }),
        ))
        .with_profile_ml_metrics_service(ProfileMlMetricsService::for_tests(metrics_stub))
        .with_reranker_bootstrap_service(RerankerBootstrapService::for_tests(
            RerankerBootstrapServiceStub::default().with_response(Ok(BootstrapResponsePayload {
                retrained: true,
                example_count: 16,
                reason: None,
                model_path: Some("/tmp/model.json".to_string()),
                artifact_version: Some("trained_reranker_v3".to_string()),
                model_type: Some("logistic_regression".to_string()),
                training: Some(BootstrapTrainingSummary {
                    example_count: 16,
                    positive_count: 5,
                    medium_count: 7,
                    negative_count: 4,
                    saved_only_count: 2,
                    viewed_only_count: 3,
                    medium_default_count: 0,
                    epochs: 10,
                    learning_rate: 0.1,
                    l2: 0.01,
                    loss: 0.5,
                }),
                evaluation: Some(BootstrapEvaluationSummary {
                    profile_id: "profile-1".to_string(),
                    label_policy_version: "outcome_label_v3".to_string(),
                    metrics_version: "reranker_eval_v2".to_string(),
                    signal_weight_policy_version: "outcome_signal_weight_v2".to_string(),
                    split_method: "temporal".to_string(),
                    example_count: 16,
                    train_example_count: 12,
                    test_example_count: 4,
                    positive_count: 5,
                    top_n: 10,
                    rolling_window_count: 0,
                    variants: vec![BootstrapVariantMetrics {
                        variant: "trained_reranker_prediction".to_string(),
                        top_n: 10,
                        ordered_job_ids: vec!["job-1".to_string()],
                        top_k_positives: 1,
                        average_label_score_top_n: 1.0,
                        average_training_weight_top_n: 1.0,
                        positive_hit_rate: 1.0,
                        ndcg_at_top_n: 0.9,
                        mrr_at_top_n: 0.8,
                        map_at_top_n: 0.7,
                        precision_at_3: 0.6,
                    }],
                }),
                benchmark: Some(
                    crate::services::reranker_bootstrap::BootstrapBenchmarkSummary {
                        baseline_model_type: "logistic_regression".to_string(),
                        candidate_model_type: "bpr".to_string(),
                        baseline_positive_hit_rate: 0.5,
                        candidate_positive_hit_rate: 0.6,
                        candidate_available: true,
                        winner: "bpr".to_string(),
                        feature_set_winner: Some("full_feature_set".to_string()),
                        ablated_positive_hit_rate: Some(0.55),
                        ablation_fallback_used: true,
                        ablation_fallback_reason: Some(
                            "no_low_variance_features_detected".to_string(),
                        ),
                    },
                ),
                feature_importances: Some(std::collections::BTreeMap::from([(
                    "matched_skill_count".to_string(),
                    0.4,
                )])),
            })),
        ));

        run_retrain_cycle(&state, 15).await;

        let current_state = state
            .profile_ml_state
            .get_by_profile_id("profile-1")
            .await
            .expect("state query should work")
            .expect("state should exist");
        assert_eq!(current_state.examples_since_retrain, 0);
        assert_eq!(
            current_state.last_training_status.as_deref(),
            Some("trained")
        );

        let records = state
            .profile_ml_metrics
            .list_recent("profile-1", 10)
            .await
            .expect("metrics query should work");
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].status, "trained");
        let metrics_json = records[0]
            .metrics_json
            .as_ref()
            .expect("metrics json should be stored");
        assert_eq!(metrics_json["metrics_version"], json!("reranker_eval_v2"));
        assert_eq!(metrics_json["rolling_window_count"], json!(0));
        assert_eq!(metrics_json["variants"][0]["ndcg_at_top_n"], json!(0.9));
        assert_eq!(metrics_json["variants"][0]["mrr_at_top_n"], json!(0.8));
        assert_eq!(metrics_json["variants"][0]["map_at_top_n"], json!(0.7));
        assert_eq!(metrics_json["variants"][0]["precision_at_3"], json!(0.6));
        let benchmark_json = records[0]
            .benchmark_json
            .as_ref()
            .expect("benchmark json should be stored");
        assert_eq!(
            benchmark_json["feature_set_winner"],
            json!("full_feature_set")
        );
        assert_eq!(benchmark_json["ablated_positive_hit_rate"], json!(0.55));
        assert_eq!(benchmark_json["ablation_fallback_used"], json!(true));
        assert_eq!(
            benchmark_json["ablation_fallback_reason"],
            json!("no_low_variance_features_detected")
        );
    }

    #[tokio::test]
    async fn retrain_cycle_persists_failure_status() {
        let state = AppState::for_services(
            ProfilesService::for_tests(
                ProfilesServiceStub::default().with_profile(sample_profile()),
            ),
            JobsService::for_tests(JobsServiceStub::default()),
            ApplicationsService::for_tests(ApplicationsServiceStub::default()),
            ResumesService::for_tests(ResumesServiceStub::default()),
        )
        .with_profile_ml_state_service(ProfileMlStateService::for_tests(
            ProfileMlStateServiceStub::default().with_state(ProfileMlState {
                profile_id: "profile-1".to_string(),
                last_retrained_at: None,
                examples_since_retrain: 17,
                last_artifact_version: None,
                last_training_status: None,
            }),
        ))
        .with_profile_ml_metrics_service(ProfileMlMetricsService::for_tests(
            ProfileMlMetricsServiceStub::default(),
        ))
        .with_reranker_bootstrap_service(RerankerBootstrapService::for_tests(
            RerankerBootstrapServiceStub::default()
                .with_response(Err(RerankerBootstrapError::Http("boom".to_string()))),
        ));

        run_retrain_cycle(&state, 15).await;

        let current_state = state
            .profile_ml_state
            .get_by_profile_id("profile-1")
            .await
            .expect("state query should work")
            .expect("state should exist");
        assert_eq!(
            current_state.last_training_status.as_deref(),
            Some("bootstrap_failed")
        );

        let records = state
            .profile_ml_metrics
            .list_recent("profile-1", 10)
            .await
            .expect("metrics query should work");
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].status, "bootstrap_failed");
        assert_eq!(records[0].reason.as_deref(), Some("boom"));
        assert_eq!(records[0].metrics_json, None::<serde_json::Value>);
        let _typed: &ProfileMlMetricRecord = &records[0];
        let _ = json!({});
    }
}
