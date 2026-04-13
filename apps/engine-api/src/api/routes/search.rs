use axum::extract::{Query, State};
use serde::Deserialize;

use crate::api::dto::search::SearchResponse;
use crate::api::error::ApiError;
use crate::state::AppState;

#[derive(Deserialize)]
pub struct SearchQuery {
    pub q: String,
    pub limit: Option<i64>,
    pub page: Option<i64>,
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

    let per_page = query.limit.unwrap_or(20).clamp(1, 50);
    let page = query.page.unwrap_or(1).max(1);
    let offset = (page - 1) * per_page;

    // Fetch one extra row so we can cheaply detect whether a next page exists.
    let mut jobs = state
        .jobs_service
        .search(q, per_page + 1, offset)
        .await
        .map_err(|error| ApiError::from_repository(error, "search_query_failed"))?;

    let has_more = jobs.len() as i64 > per_page;
    jobs.truncate(per_page as usize);

    Ok(axum::Json(SearchResponse::from_jobs(
        jobs, page, per_page, has_more,
    )))
}
