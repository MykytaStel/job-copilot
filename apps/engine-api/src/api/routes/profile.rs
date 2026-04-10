use axum::{Json, extract::State};

use crate::api::dto::profile::{AnalyzeProfileRequest, AnalyzeProfileResponse};
use crate::state::AppState;

pub async fn analyze_profile(
    State(state): State<AppState>,
    Json(payload): Json<AnalyzeProfileRequest>,
) -> Json<AnalyzeProfileResponse> {
    let profile = state.profile_analysis_service.analyze(&payload.raw_text);
    let response = AnalyzeProfileResponse::from(profile);

    Json(response)
}
