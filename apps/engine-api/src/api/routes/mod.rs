pub mod analytics;
pub mod applications;
pub mod auth;
pub mod behavior;
pub mod debug;
pub mod events;
pub mod feedback;
pub mod health;
pub mod jobs;
pub mod market;
pub mod notifications;
pub mod profile;
pub mod reranker_dataset;
pub mod reranker_metrics;
pub mod resumes;
pub mod roles;
pub mod search;
pub mod search_profile;
pub mod sources;

use axum::{Router, routing::get, routing::patch, routing::post, routing::put};

use crate::state::AppState;

pub fn public_router() -> Router<AppState> {
    Router::new()
        .route("/health", get(health::health))
        .route("/api/v1/ping", get(health::ping))
        .route("/api/v1/auth/register", post(auth::register))
        .route("/api/v1/auth/login", post(auth::login))
}

pub fn internal_router() -> Router<AppState> {
    Router::new().route(
        "/api/v1/debug/reranker-status",
        get(debug::get_reranker_status),
    )
}

pub fn protected_router() -> Router<AppState> {
    Router::new()
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
            "/api/v1/profiles/{id}/behavior-summary",
            get(behavior::get_behavior_summary),
        )
        .route(
            "/api/v1/profiles/{id}/feedback",
            get(feedback::list_feedback),
        )
        .route(
            "/api/v1/profiles/{id}/reranker-dataset",
            get(reranker_dataset::get_reranker_dataset),
        )
        .route(
            "/api/v1/profiles/{id}/reranker/metrics",
            get(reranker_metrics::get_reranker_metrics),
        )
        .route("/api/v1/profiles/{id}/events", post(events::log_user_event))
        .route(
            "/api/v1/profiles/{id}/events/summary",
            get(events::get_user_event_summary),
        )
        .route(
            "/api/v1/profiles/{id}/jobs/{job_id}/saved",
            put(feedback::save_job).delete(feedback::unsave_job),
        )
        .route(
            "/api/v1/profiles/{id}/jobs/{job_id}/hidden",
            put(feedback::hide_job).delete(feedback::unhide_job),
        )
        .route(
            "/api/v1/profiles/{id}/jobs/{job_id}/bad-fit",
            put(feedback::mark_job_bad_fit).delete(feedback::unmark_job_bad_fit),
        )
        .route(
            "/api/v1/profiles/{id}/jobs/{job_id}/salary-signal",
            put(feedback::set_job_salary_signal),
        )
        .route(
            "/api/v1/profiles/{id}/jobs/{job_id}/interest-rating",
            put(feedback::set_job_interest_rating),
        )
        .route(
            "/api/v1/profiles/{id}/jobs/{job_id}/work-mode-signal",
            put(feedback::set_job_work_mode_signal),
        )
        .route(
            "/api/v1/profiles/{id}/jobs/{job_id}/legitimacy-signal",
            put(feedback::set_job_legitimacy_signal),
        )
        .route(
            "/api/v1/profiles/{id}/jobs/{job_id}/tags",
            put(feedback::tag_job_feedback),
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
            "/api/v1/profiles/{id}/jobs/match",
            post(jobs::bulk_profile_job_match),
        )
        .route(
            "/api/v1/profiles/{id}/jobs/{job_id}/match",
            get(jobs::get_profile_job_match),
        )
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
        .route("/api/v1/market/overview", get(market::get_market_overview))
        .route(
            "/api/v1/market/companies",
            get(market::get_market_companies),
        )
        .route(
            "/api/v1/notifications",
            get(notifications::list_notifications),
        )
        .route(
            "/api/v1/notifications/unread-count",
            get(notifications::get_unread_count),
        )
        .route(
            "/api/v1/notifications/{id}/read",
            post(notifications::mark_notification_read),
        )
        .route(
            "/api/v1/market/salaries",
            get(market::get_market_salary_trend),
        )
        .route(
            "/api/v1/market/salary-trends",
            get(market::get_market_salary_trends),
        )
        .route("/api/v1/market/roles", get(market::get_market_role_demand))
        .route(
            "/api/v1/profiles/{id}/analytics/summary",
            get(analytics::get_analytics_summary),
        )
        .route(
            "/api/v1/profiles/{id}/funnel-summary",
            get(analytics::get_funnel_summary),
        )
        .route(
            "/api/v1/profiles/{id}/analytics/llm-context",
            get(analytics::get_llm_context),
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
