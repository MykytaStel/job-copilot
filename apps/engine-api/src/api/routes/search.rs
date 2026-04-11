use axum::extract::{Query, State};
use serde::Deserialize;

use crate::api::dto::search::SearchResponse;
use crate::api::error::ApiError;
use crate::state::AppState;

#[derive(Deserialize)]
pub struct SearchQuery {
    pub q: String,
}

pub async fn search(
    State(state): State<AppState>,
    Query(query): Query<SearchQuery>,
) -> Result<axum::Json<SearchResponse>, ApiError> {
    let q = query.q.trim();

    if q.is_empty() {
        return Err(ApiError::bad_request(
            "invalid_search_query",
            "Query parameter 'q' must not be empty",
        ));
    }

    let jobs = state
        .jobs_service
        .search(q, 20)
        .await
        .map_err(|error| ApiError::from_repository(error, "search_query_failed"))?;

    Ok(axum::Json(SearchResponse::from_jobs(jobs)))
}
