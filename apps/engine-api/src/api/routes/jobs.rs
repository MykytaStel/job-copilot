use axum::{Json, extract::State};

use crate::api::dto::jobs::JobResponse;
use crate::state::AppState;

pub async fn get_mock_job(State(state): State<AppState>) -> Json<JobResponse> {
    let job = state.jobs_service.get_mock_job();
    let response = JobResponse::from(job);

    Json(response)
}
