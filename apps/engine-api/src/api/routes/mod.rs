pub mod applications;
pub mod health;
pub mod jobs;
pub mod profile;
pub mod resumes;
pub mod roles;
pub mod search;

use axum::{Router, routing::get, routing::patch, routing::post};

use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/health", get(health::health))
        .route("/api/v1/ping", get(health::ping))
        .route(
            "/api/v1/applications/recent",
            get(applications::get_recent_applications),
        )
        .route(
            "/api/v1/applications",
            post(applications::create_application),
        )
        .route(
            "/api/v1/applications/{id}",
            get(applications::get_application_by_id).patch(applications::patch_application),
        )
        .route("/api/v1/jobs/recent", get(jobs::get_recent_jobs))
        .route("/api/v1/jobs/{id}", get(jobs::get_job_by_id))
        .route("/api/v1/jobs/{id}/fit", get(jobs::get_job_fit))
        .route("/api/v1/jobs/{id}/match", get(jobs::get_job_match))
        .route("/api/v1/jobs/{id}/match", post(jobs::score_job_match))
        .route("/api/v1/roles", get(roles::list_roles))
        .route("/api/v1/resumes", get(resumes::list_resumes))
        .route("/api/v1/resumes/active", get(resumes::get_active_resume))
        .route(
            "/api/v1/resumes/{id}/activate",
            post(resumes::activate_resume),
        )
        .route("/api/v1/resume/upload", post(resumes::upload_resume))
        .route("/api/v1/search", get(search::search))
        .route("/api/v1/profiles", post(profile::create_profile))
        .route("/api/v1/profiles/{id}", get(profile::get_profile_by_id))
        .route("/api/v1/profiles/{id}", patch(profile::update_profile))
        .route(
            "/api/v1/profiles/{id}/analyze",
            post(profile::analyze_profile),
        )
        .route(
            "/api/v1/profiles/{id}/search-profile/build",
            post(profile::build_search_profile),
        )
}
