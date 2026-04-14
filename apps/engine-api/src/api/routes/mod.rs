pub mod analytics;
pub mod applications;
pub mod feedback;
pub mod health;
pub mod jobs;
pub mod profile;
pub mod resumes;
pub mod roles;
pub mod search;
pub mod search_profile;
pub mod sources;

use axum::{Router, routing::get, routing::patch, routing::post, routing::put};

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
        .route(
            "/api/v1/applications/{id}/activities",
            post(applications::create_activity),
        )
        .route(
            "/api/v1/applications/{id}/notes",
            post(applications::create_note),
        )
        .route(
            "/api/v1/applications/{id}/contacts",
            post(applications::add_application_contact),
        )
        .route(
            "/api/v1/applications/{id}/offer",
            put(applications::upsert_offer),
        )
        .route(
            "/api/v1/contacts",
            get(applications::list_contacts).post(applications::create_contact),
        )
        .route(
            "/api/v1/profiles/{id}/feedback",
            get(feedback::list_feedback),
        )
        .route(
            "/api/v1/profiles/{id}/jobs/{job_id}/saved",
            put(feedback::save_job),
        )
        .route(
            "/api/v1/profiles/{id}/jobs/{job_id}/hidden",
            put(feedback::hide_job),
        )
        .route(
            "/api/v1/profiles/{id}/jobs/{job_id}/bad-fit",
            put(feedback::mark_job_bad_fit),
        )
        .route(
            "/api/v1/profiles/{id}/companies/whitelist",
            put(feedback::add_company_whitelist).delete(feedback::remove_company_whitelist),
        )
        .route(
            "/api/v1/profiles/{id}/companies/blacklist",
            put(feedback::add_company_blacklist).delete(feedback::remove_company_blacklist),
        )
        .route("/api/v1/jobs/recent", get(jobs::get_recent_jobs))
        .route("/api/v1/jobs/{id}", get(jobs::get_job_by_id))
        .route(
            "/api/v1/ml/jobs/{id}/lifecycle",
            get(jobs::get_ml_job_lifecycle),
        )
        .route("/api/v1/jobs/{id}/fit", get(jobs::get_job_fit))
        .route("/api/v1/jobs/{id}/match", get(jobs::get_job_match))
        .route("/api/v1/jobs/{id}/match", post(jobs::score_job_match))
        .route(
            "/api/v1/analytics/salary",
            get(analytics::get_salary_intelligence),
        )
        .route("/api/v1/roles", get(roles::list_roles))
        .route("/api/v1/sources", get(sources::list_sources))
        .route("/api/v1/resumes", get(resumes::list_resumes))
        .route("/api/v1/resumes/active", get(resumes::get_active_resume))
        .route(
            "/api/v1/resumes/{id}/activate",
            post(resumes::activate_resume),
        )
        .route("/api/v1/resume/upload", post(resumes::upload_resume))
        .route("/api/v1/search", get(search::search))
        .route("/api/v1/search/run", post(search::run_search))
        .route(
            "/api/v1/search-profile/build",
            post(search_profile::build_search_profile),
        )
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
