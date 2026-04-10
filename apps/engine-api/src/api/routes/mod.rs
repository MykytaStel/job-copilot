pub mod health;
pub mod jobs;
pub mod profile;
pub mod roles;
pub mod search_profile;

use axum::{Router, routing::get, routing::post};

use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/health", get(health::health))
        .route("/api/v1/ping", get(health::ping))
        .route("/api/v1/jobs/mock", get(jobs::get_mock_job))
        .route("/api/v1/roles", get(roles::list_roles))
        .route("/api/v1/profile/analyze", post(profile::analyze_profile))
        .route(
            "/api/v1/search-profile/build",
            post(search_profile::build_search_profile),
        )
}
