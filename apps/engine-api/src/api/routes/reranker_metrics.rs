use axum::extract::{Path, Query, State};
use serde::Deserialize;

use crate::api::dto::reranker_metrics::{
    ProfileMlMetricRecordResponse, ProfileMlStateResponse, RerankerMetricsResponse,
};
use crate::api::error::ApiError;
use crate::api::routes::feedback::ensure_profile_exists;
use crate::domain::profile::ml::ProfileMlState;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct RerankerMetricsQuery {
    pub limit: Option<usize>,
}

pub async fn get_reranker_metrics(
    State(state): State<AppState>,
    Path(profile_id): Path<String>,
    Query(query): Query<RerankerMetricsQuery>,
) -> Result<axum::Json<RerankerMetricsResponse>, ApiError> {
    ensure_profile_exists(&state, &profile_id).await?;

    let limit = query.limit.unwrap_or(10).clamp(1, 50);
    let current_state = state
        .profile_ml_state
        .get_by_profile_id(&profile_id)
        .await
        .map_err(|error| ApiError::from_repository(error, "reranker_metrics_query_failed"))?
        .unwrap_or(ProfileMlState {
            profile_id: profile_id.clone(),
            ..ProfileMlState::default()
        });
    let runs = state
        .profile_ml_metrics
        .list_recent(&profile_id, limit)
        .await
        .map_err(|error| ApiError::from_repository(error, "reranker_metrics_query_failed"))?;

    Ok(axum::Json(RerankerMetricsResponse {
        profile_id,
        state: ProfileMlStateResponse::from(current_state),
        runs: runs
            .into_iter()
            .map(ProfileMlMetricRecordResponse::from)
            .collect(),
    }))
}

#[cfg(test)]
mod tests {
    use axum::extract::{Path, Query, State};
    use serde_json::json;

    use crate::api::dto::reranker_metrics::RerankerMetricsResponse;
    use crate::domain::profile::ml::{CreateProfileMlMetric, ProfileMlState};
    use crate::domain::profile::model::Profile;
    use crate::services::applications::{ApplicationsService, ApplicationsServiceStub};
    use crate::services::jobs::{JobsService, JobsServiceStub};
    use crate::services::profile_ml_metrics::{
        ProfileMlMetricsService, ProfileMlMetricsServiceStub,
    };
    use crate::services::profile_ml_state::{ProfileMlStateService, ProfileMlStateServiceStub};
    use crate::services::profiles::{ProfilesService, ProfilesServiceStub};
    use crate::services::resumes::{ResumesService, ResumesServiceStub};
    use crate::state::AppState;

    use super::{RerankerMetricsQuery, get_reranker_metrics};

    fn sample_profile() -> Profile {
        Profile {
            id: "profile-1".to_string(),
            name: "Jane Doe".to_string(),
            email: "jane@example.com".to_string(),
            location: Some("Kyiv".to_string()),
            raw_text: "Senior backend engineer".to_string(),
            analysis: None,
            years_of_experience: None,
            salary_min: None,
            salary_max: None,
            salary_currency: "USD".to_string(),
            languages: vec![],
            preferred_work_mode: None,
            search_preferences: None,
            created_at: "2026-04-14T00:00:00Z".to_string(),
            updated_at: "2026-04-14T00:00:00Z".to_string(),
            skills_updated_at: None,
        }
    }

    #[tokio::test]
    async fn returns_reranker_metrics_for_profile() {
        let metrics_service = ProfileMlMetricsService::for_tests(
            ProfileMlMetricsServiceStub::default().with_record(
                ProfileMlMetricsServiceStub::default()
                    .create(CreateProfileMlMetric {
                        profile_id: "profile-1".to_string(),
                        status: "trained".to_string(),
                        artifact_version: Some("trained_reranker_v3".to_string()),
                        model_type: Some("logistic_regression".to_string()),
                        reason: None,
                        metrics_json: Some(json!({ "variants": [] })),
                        training_json: Some(json!({ "example_count": 18 })),
                        feature_importances_json: Some(json!({ "matched_skill_count": 0.4 })),
                        benchmark_json: Some(json!({ "winner": "logistic_regression" })),
                    })
                    .expect("stub metric should be created"),
            ),
        );
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
                last_retrained_at: Some("2026-04-22T10:00:00Z".to_string()),
                examples_since_retrain: 4,
                last_artifact_version: Some("trained_reranker_v3".to_string()),
                last_training_status: Some("trained".to_string()),
            }),
        ))
        .with_profile_ml_metrics_service(metrics_service);

        let axum::Json(response) = get_reranker_metrics(
            State(state),
            Path("profile-1".to_string()),
            Query(RerankerMetricsQuery { limit: Some(5) }),
        )
        .await
        .expect("metrics route should succeed");

        let response: RerankerMetricsResponse = response;
        assert_eq!(response.profile_id, "profile-1");
        assert_eq!(response.state.examples_since_retrain, 4);
        assert_eq!(response.runs.len(), 1);
        assert_eq!(response.runs[0].status, "trained");
    }
}
