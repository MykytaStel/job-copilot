use axum::extract::State;

use crate::api::dto::analytics::SalaryIntelligenceResponse;
use crate::api::error::ApiError;
use crate::state::AppState;

pub async fn get_salary_intelligence(
    State(state): State<AppState>,
) -> Result<axum::Json<SalaryIntelligenceResponse>, ApiError> {
    let buckets = state
        .salary_service
        .salary_intelligence()
        .await
        .map_err(|error| ApiError::from_repository(error, "salary_query_failed"))?;

    Ok(axum::Json(SalaryIntelligenceResponse {
        buckets: buckets
            .into_iter()
            .map(crate::api::dto::analytics::SalaryBucketResponse::from)
            .collect(),
    }))
}
