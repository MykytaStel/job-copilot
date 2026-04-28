use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::Uri;
use axum::response::IntoResponse;
use axum::{body, http::StatusCode};
use serde_json::{Value, json};

use crate::api::error::ApiJson;
use crate::domain::job::model::{
    Job, JobFeedSummary, JobLifecycleStage, JobSourceVariant, JobView,
};
use crate::domain::profile::model::{Profile, ProfileAnalysis};
use crate::domain::role::RoleId;
use crate::domain::source::SourceId;
use crate::services::applications::{ApplicationsService, ApplicationsServiceStub};
use crate::services::jobs::{JobsService, JobsServiceStub};
use crate::services::profiles::{ProfilesService, ProfilesServiceStub};
use crate::services::ranking::runtime::RerankerRuntimeMode;
use crate::services::resumes::{ResumesService, ResumesServiceStub};
use crate::services::user_events::{UserEventsService, UserEventsServiceStub};
use crate::state::AppState;

use super::{
    BulkProfileJobMatchRequest, JobContextQuery, RecentJobsQuery, bulk_profile_job_match,
    get_job_by_id, get_ml_job_lifecycle, get_profile_job_match, get_recent_jobs,
};

fn sample_job_view(id: &str) -> JobView {
    JobView {
        job: Job {
            id: id.to_string(),
            title: "Platform Engineer".to_string(),
            company_name: "SignalHire".to_string(),
            location: Some("Remote, Europe".to_string()),
            remote_type: Some("remote".to_string()),
            seniority: Some("senior".to_string()),
            description_text: "Rust and Postgres".to_string(),
            salary_min: Some(5000),
            salary_max: Some(6500),
            salary_currency: Some("USD".to_string()),
            language: None,
            posted_at: Some("2026-04-14T08:00:00Z".to_string()),
            last_seen_at: "2026-04-16T09:00:00Z".to_string(),
            is_active: true,
        },
        first_seen_at: "2026-04-14T08:00:00Z".to_string(),
        inactivated_at: None,
        reactivated_at: Some("2026-04-16T09:00:00Z".to_string()),
        lifecycle_stage: JobLifecycleStage::Reactivated,
        primary_variant: Some(JobSourceVariant {
            source: "mock_source".to_string(),
            source_job_id: "platform-001".to_string(),
            source_url: "https://mock-source.example/jobs/platform-001".to_string(),
            raw_payload: None,
            fetched_at: "2026-04-16T09:00:00Z".to_string(),
            last_seen_at: "2026-04-16T09:00:00Z".to_string(),
            is_active: true,
            inactivated_at: None,
        }),
    }
}

fn job_view_with_lifecycle(
    id: &str,
    posted_at: Option<&str>,
    first_seen_at: &str,
    last_seen_at: &str,
    inactivated_at: Option<&str>,
    reactivated_at: Option<&str>,
    lifecycle_stage: JobLifecycleStage,
) -> JobView {
    let mut view = sample_job_view(id);
    view.job.posted_at = posted_at.map(str::to_string);
    view.first_seen_at = first_seen_at.to_string();
    view.job.last_seen_at = last_seen_at.to_string();
    view.inactivated_at = inactivated_at.map(str::to_string);
    view.reactivated_at = reactivated_at.map(str::to_string);
    view.lifecycle_stage = lifecycle_stage.clone();
    view.job.is_active = !matches!(lifecycle_stage, JobLifecycleStage::Inactive);
    view
}

fn sample_profile() -> Profile {
    Profile {
        id: "profile-1".to_string(),
        name: "Jane Doe".to_string(),
        email: "jane@example.com".to_string(),
        location: Some("Kyiv".to_string()),
        raw_text: "Senior frontend engineer with React, TypeScript and design system experience"
            .to_string(),
        analysis: Some(ProfileAnalysis {
            summary: "Senior frontend engineer".to_string(),
            primary_role: RoleId::FrontendEngineer,
            seniority: "senior".to_string(),
            skills: vec!["react".to_string(), "typescript".to_string()],
            keywords: vec!["frontend".to_string(), "design system".to_string()],
        }),
        years_of_experience: None,
        salary_min: None,
        salary_max: None,
        salary_currency: "USD".to_string(),
        languages: vec![],
        preferred_locations: vec![],
        experience: vec![],
        work_mode_preference: "any".to_string(),
        preferred_language: None,
        search_preferences: None,
        created_at: "2026-04-14T08:00:00Z".to_string(),
        updated_at: "2026-04-14T08:00:00Z".to_string(),
        skills_updated_at: Some("2026-04-14T08:00:00Z".to_string()),
        portfolio_url: None,
        github_url: None,
        linkedin_url: None,
    }
}

fn profile_match_state(job_view: JobView) -> AppState {
    AppState::for_services(
        ProfilesService::for_tests(ProfilesServiceStub::default().with_profile(sample_profile())),
        JobsService::for_tests(JobsServiceStub::default().with_job_view(job_view)),
        ApplicationsService::for_tests(ApplicationsServiceStub::default()),
        ResumesService::for_tests(ResumesServiceStub::default()),
    )
}

#[tokio::test]
async fn returns_service_unavailable_when_database_is_missing() {
    let result = get_job_by_id(
        State(AppState::without_database()),
        Path("job-123".to_string()),
        Query(JobContextQuery::default()),
    )
    .await;

    let response = match result {
        Ok(_) => panic!("handler should fail without a configured database"),
        Err(error) => error.into_response(),
    };

    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
}

#[tokio::test]
async fn returns_not_found_for_unknown_job() {
    let state = AppState::for_services(
        ProfilesService::for_tests(ProfilesServiceStub::default()),
        JobsService::for_tests(JobsServiceStub::default()),
        ApplicationsService::for_tests(ApplicationsServiceStub::default()),
        ResumesService::for_tests(ResumesServiceStub::default()),
    );
    let result = get_job_by_id(
        State(state),
        Path("missing-job".to_string()),
        Query(JobContextQuery::default()),
    )
    .await;

    let response = match result {
        Ok(_) => panic!("handler should return not found for unknown job"),
        Err(error) => error.into_response(),
    };

    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    let body = body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("response body should be readable");
    let payload: Value = serde_json::from_slice(&body).expect("response body should be valid json");

    assert_eq!(payload["code"], json!("job_not_found"));
}

#[tokio::test]
async fn rejects_invalid_recent_jobs_limit() {
    let result = get_recent_jobs(
        State(AppState::without_database()),
        Query(RecentJobsQuery {
            limit: Some(0),
            lifecycle: None,
            source: None,
            profile_id: None,
        }),
    )
    .await;

    let response = match result {
        Ok(Json(_)) => panic!("handler should reject invalid limit"),
        Err(error) => error.into_response(),
    };

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("response body should be readable");
    let payload: Value = serde_json::from_slice(&body).expect("response body should be valid json");

    assert_eq!(payload["code"], json!("invalid_limit"));
}

#[tokio::test]
async fn profile_job_match_returns_canonical_fit_diagnostics() {
    let frontend_job = JobView {
        job: Job {
            title: "Senior Front-end React Developer".to_string(),
            description_text: "Ship frontend design system features with React and TypeScript. Partner with product and design on accessibility and performance improvements. Own component architecture, testing quality, and release readiness across a shared UI platform used by multiple product teams. Drive performance budgets, documentation standards, and cross-team adoption for reusable components, tokens, and frontend platform tooling.".to_string(),
            ..sample_job_view("job-frontend-1").job
        },
        ..sample_job_view("job-frontend-1")
    };
    let state = AppState::for_services(
        ProfilesService::for_tests(ProfilesServiceStub::default().with_profile(sample_profile())),
        JobsService::for_tests(JobsServiceStub::default().with_job_view(frontend_job)),
        ApplicationsService::for_tests(ApplicationsServiceStub::default()),
        ResumesService::for_tests(ResumesServiceStub::default()),
    );

    let Json(response) = get_profile_job_match(
        State(state),
        None,
        Path(("profile-1".to_string(), "job-frontend-1".to_string())),
    )
    .await
    .expect("profile match should succeed");

    assert!(response.score > 0);
    assert!(response.matched_skills.contains(&"react".to_string()));
    assert!(
        response
            .positive_reasons
            .iter()
            .any(|reason| reason.contains("Matched"))
    );
    assert_eq!(response.description_quality, "strong");
}

#[tokio::test]
async fn bulk_profile_job_match_supports_dashboard_sorting() {
    let strong_job = JobView {
        job: Job {
            title: "Senior Front-end React Developer".to_string(),
            description_text: "Ship frontend design system features with React and TypeScript. Collaborate with product, accessibility, and platform teams to improve shared components, design tokens, and performance budgets across multiple customer-facing surfaces.".to_string(),
            ..sample_job_view("job-frontend-strong").job
        },
        ..sample_job_view("job-frontend-strong")
    };
    let weak_job = JobView {
        job: Job {
            title: "Senior UI Engineer".to_string(),
            description_text: "Improve shared product experiences and collaborate with design."
                .to_string(),
            ..sample_job_view("job-frontend-weak").job
        },
        ..sample_job_view("job-frontend-weak")
    };
    let state = AppState::for_services(
        ProfilesService::for_tests(ProfilesServiceStub::default().with_profile(sample_profile())),
        JobsService::for_tests(
            JobsServiceStub::default()
                .with_job_view(strong_job)
                .with_job_view(weak_job),
        ),
        ApplicationsService::for_tests(ApplicationsServiceStub::default()),
        ResumesService::for_tests(ResumesServiceStub::default()),
    );

    let Json(response) = bulk_profile_job_match(
        State(state),
        None,
        Path("profile-1".to_string()),
        ApiJson(BulkProfileJobMatchRequest {
            job_ids: vec![
                "job-frontend-weak".to_string(),
                "job-frontend-strong".to_string(),
            ],
        }),
    )
    .await
    .expect("bulk profile match should succeed");

    assert_eq!(response.results.len(), 2);
    assert_eq!(response.results[0].job_id, "job-frontend-strong");
    assert!(response.results[0].score > response.results[1].score);
    assert!(response.meta.low_evidence_jobs <= response.meta.returned_jobs);
}

#[tokio::test]
async fn bulk_profile_job_match_uses_deterministic_runtime_when_requested() {
    let state = profile_match_state(sample_job_view("job-runtime-deterministic"))
        .with_reranker_runtime_mode(RerankerRuntimeMode::Deterministic);

    let Json(response) = bulk_profile_job_match(
        State(state),
        None,
        Path("profile-1".to_string()),
        ApiJson(BulkProfileJobMatchRequest {
            job_ids: vec!["job-runtime-deterministic".to_string()],
        }),
    )
    .await
    .expect("bulk profile match should succeed");

    assert_eq!(response.meta.reranker_mode_requested, "deterministic");
    assert_eq!(response.meta.reranker_mode_active, "deterministic");
    assert_eq!(response.meta.reranker_fallback_reason, None);
    assert_eq!(
        response.results[0].score_breakdown.reranker_mode,
        "deterministic"
    );
}

#[tokio::test]
async fn bulk_profile_job_match_falls_back_from_learned_to_deterministic() {
    let state = profile_match_state(sample_job_view("job-runtime-learned-fallback"))
        .with_user_events_service(UserEventsService::for_tests(
            UserEventsServiceStub::default().with_database_disabled(),
        ));

    let Json(response) = bulk_profile_job_match(
        State(state),
        None,
        Path("profile-1".to_string()),
        ApiJson(BulkProfileJobMatchRequest {
            job_ids: vec!["job-runtime-learned-fallback".to_string()],
        }),
    )
    .await
    .expect("bulk profile match should succeed");

    assert_eq!(response.meta.reranker_mode_requested, "learned");
    assert_eq!(response.meta.reranker_mode_active, "deterministic");
    assert!(
        response
            .meta
            .reranker_fallback_reason
            .as_deref()
            .is_some_and(|reason| reason.contains("Learned reranker unavailable"))
    );
    assert_eq!(
        response.results[0].score_breakdown.reranker_mode,
        "fallback"
    );
    assert!(
        response.results[0]
            .reasons
            .iter()
            .any(|reason| reason.contains("Learned reranker unavailable"))
    );
}

#[tokio::test]
async fn bulk_profile_job_match_falls_back_from_trained_to_learned() {
    let state = profile_match_state(sample_job_view("job-runtime-trained-to-learned"))
        .with_trained_reranker(true, None)
        .with_reranker_runtime_mode(RerankerRuntimeMode::Trained);

    let Json(response) = bulk_profile_job_match(
        State(state),
        None,
        Path("profile-1".to_string()),
        ApiJson(BulkProfileJobMatchRequest {
            job_ids: vec!["job-runtime-trained-to-learned".to_string()],
        }),
    )
    .await
    .expect("bulk profile match should succeed");

    assert_eq!(response.meta.reranker_mode_requested, "trained");
    assert_eq!(response.meta.reranker_mode_active, "learned");
    assert!(
        response
            .meta
            .reranker_fallback_reason
            .as_deref()
            .is_some_and(|reason| reason.contains("fell back to learned reranker"))
    );
    assert_eq!(response.results[0].score_breakdown.reranker_mode, "learned");
}

#[tokio::test]
async fn bulk_profile_job_match_falls_back_from_trained_to_deterministic_without_safe_layer() {
    let state = profile_match_state(sample_job_view("job-runtime-trained-to-deterministic"))
        .with_learned_reranker_enabled(false)
        .with_trained_reranker(true, None)
        .with_reranker_runtime_mode(RerankerRuntimeMode::Trained);

    let Json(response) = bulk_profile_job_match(
        State(state),
        None,
        Path("profile-1".to_string()),
        ApiJson(BulkProfileJobMatchRequest {
            job_ids: vec!["job-runtime-trained-to-deterministic".to_string()],
        }),
    )
    .await
    .expect("bulk profile match should succeed");

    assert_eq!(response.meta.reranker_mode_requested, "trained");
    assert_eq!(response.meta.reranker_mode_active, "deterministic");
    assert!(
        response
            .meta
            .reranker_fallback_reason
            .as_deref()
            .is_some_and(|reason| reason.contains("kept deterministic ranking"))
    );
    assert_eq!(
        response.results[0].score_breakdown.reranker_mode,
        "fallback"
    );
    assert!(
        response.results[0]
            .reasons
            .iter()
            .any(|reason| reason.contains("Trained reranker unavailable"))
    );
}

#[tokio::test]
async fn returns_job_feed_summary_and_lifecycle_fields() {
    let state = AppState::for_services(
        ProfilesService::for_tests(ProfilesServiceStub::default()),
        JobsService::for_tests(
            JobsServiceStub::default()
                .with_job_view(sample_job_view("job-123"))
                .with_feed_summary(JobFeedSummary {
                    total_jobs: 3,
                    active_jobs: 2,
                    inactive_jobs: 1,
                    reactivated_jobs: 1,
                    last_ingested_at: None,
                }),
        ),
        ApplicationsService::for_tests(ApplicationsServiceStub::default()),
        ResumesService::for_tests(ResumesServiceStub::default()),
    );

    let response = get_recent_jobs(
        State(state),
        Query(RecentJobsQuery {
            limit: Some(20),
            lifecycle: None,
            source: None,
            profile_id: None,
        }),
    )
    .await
    .expect("recent jobs should succeed")
    .into_response();

    let body = body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("response body should be readable");
    let payload: Value = serde_json::from_slice(&body).expect("response body should be valid JSON");

    assert_eq!(payload["summary"]["reactivated_jobs"], json!(1));
    assert_eq!(payload["jobs"][0]["lifecycle_stage"], json!("reactivated"));
    assert_eq!(payload["jobs"][0]["location"], json!("Remote, Europe"));
    assert_eq!(
        payload["jobs"][0]["primary_variant"]["source"],
        json!("mock_source")
    );
    assert_eq!(
        payload["jobs"][0]["presentation"]["salary_label"],
        json!("5,000-6,500 USD")
    );
    assert_eq!(
        payload["jobs"][0]["presentation"]["lifecycle_primary_label"],
        json!("Reactivated 2026-04-16")
    );
    assert_eq!(
        payload["jobs"][0]["presentation"]["lifecycle_secondary_label"],
        json!("Last confirmed active 2026-04-16")
    );
}

async fn recent_jobs_payload_for(job_view: JobView) -> Value {
    let lifecycle_stage = job_view.lifecycle_stage.clone();
    let state = AppState::for_services(
        ProfilesService::for_tests(ProfilesServiceStub::default()),
        JobsService::for_tests(
            JobsServiceStub::default()
                .with_job_view(job_view)
                .with_feed_summary(JobFeedSummary {
                    total_jobs: 1,
                    active_jobs: if matches!(lifecycle_stage, JobLifecycleStage::Inactive) {
                        0
                    } else {
                        1
                    },
                    inactive_jobs: if matches!(lifecycle_stage, JobLifecycleStage::Inactive) {
                        1
                    } else {
                        0
                    },
                    reactivated_jobs: if matches!(lifecycle_stage, JobLifecycleStage::Reactivated) {
                        1
                    } else {
                        0
                    },
                    last_ingested_at: None,
                }),
        ),
        ApplicationsService::for_tests(ApplicationsServiceStub::default()),
        ResumesService::for_tests(ResumesServiceStub::default()),
    );

    let response = get_recent_jobs(
        State(state),
        Query(RecentJobsQuery {
            limit: Some(20),
            lifecycle: None,
            source: None,
            profile_id: None,
        }),
    )
    .await
    .expect("recent jobs should succeed")
    .into_response();

    let body = body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("response body should be readable");

    serde_json::from_slice(&body).expect("response body should be valid JSON")
}

#[tokio::test]
async fn recent_jobs_active_with_posted_at_returns_posted_and_last_confirmed_labels() {
    let payload = recent_jobs_payload_for(job_view_with_lifecycle(
        "job-active-posted",
        Some("2026-04-15T08:00:00Z"),
        "2026-04-15T08:00:00Z",
        "2026-04-22T09:00:00Z",
        None,
        None,
        JobLifecycleStage::Active,
    ))
    .await;

    assert_eq!(
        payload["jobs"][0]["presentation"]["lifecycle_primary_label"],
        json!("Posted 2026-04-15")
    );
    assert_eq!(
        payload["jobs"][0]["presentation"]["lifecycle_secondary_label"],
        json!("Last confirmed active 2026-04-22")
    );
}

#[tokio::test]
async fn recent_jobs_active_without_posted_at_uses_seen_since_fallback() {
    let payload = recent_jobs_payload_for(job_view_with_lifecycle(
        "job-active-seen",
        None,
        "2026-04-15T08:00:00Z",
        "2026-04-22T09:00:00Z",
        None,
        None,
        JobLifecycleStage::Active,
    ))
    .await;

    assert_eq!(
        payload["jobs"][0]["presentation"]["lifecycle_primary_label"],
        json!("Seen since 2026-04-15")
    );
    assert_eq!(
        payload["jobs"][0]["presentation"]["lifecycle_secondary_label"],
        json!("Last confirmed active 2026-04-22")
    );
}

#[tokio::test]
async fn recent_jobs_inactive_returns_inactive_since_label() {
    let payload = recent_jobs_payload_for(job_view_with_lifecycle(
        "job-inactive",
        Some("2026-04-15T08:00:00Z"),
        "2026-04-15T08:00:00Z",
        "2026-04-20T09:00:00Z",
        Some("2026-04-20T09:00:00Z"),
        None,
        JobLifecycleStage::Inactive,
    ))
    .await;

    assert_eq!(
        payload["jobs"][0]["presentation"]["lifecycle_primary_label"],
        json!("Posted 2026-04-15")
    );
    assert_eq!(
        payload["jobs"][0]["presentation"]["lifecycle_secondary_label"],
        json!("Inactive since 2026-04-20")
    );
}

#[tokio::test]
async fn recent_jobs_reactivated_returns_reactivated_and_last_confirmed_labels() {
    let payload = recent_jobs_payload_for(job_view_with_lifecycle(
        "job-reactivated",
        Some("2026-04-15T08:00:00Z"),
        "2026-04-15T08:00:00Z",
        "2026-04-22T09:00:00Z",
        None,
        Some("2026-04-22T09:00:00Z"),
        JobLifecycleStage::Reactivated,
    ))
    .await;

    assert_eq!(
        payload["jobs"][0]["presentation"]["lifecycle_primary_label"],
        json!("Reactivated 2026-04-22")
    );
    assert_eq!(
        payload["jobs"][0]["presentation"]["lifecycle_secondary_label"],
        json!("Last confirmed active 2026-04-22")
    );
}

#[tokio::test]
async fn returns_ml_lifecycle_payload() {
    let state = AppState::for_services(
        ProfilesService::for_tests(ProfilesServiceStub::default()),
        JobsService::for_tests(
            JobsServiceStub::default().with_job_view(sample_job_view("job-123")),
        ),
        ApplicationsService::for_tests(ApplicationsServiceStub::default()),
        ResumesService::for_tests(ResumesServiceStub::default()),
    );

    let response = get_ml_job_lifecycle(State(state), Path("job-123".to_string()))
        .await
        .expect("ML lifecycle route should succeed")
        .into_response();

    let body = body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("response body should be readable");
    let payload: Value = serde_json::from_slice(&body).expect("response body should be valid JSON");

    assert_eq!(payload["id"], json!("job-123"));
    assert_eq!(payload["lifecycle_stage"], json!("reactivated"));
    assert_eq!(payload["location"], json!("Remote, Europe"));
    assert_eq!(
        payload["primary_variant"]["source_url"],
        json!("https://mock-source.example/jobs/platform-001")
    );
    assert_eq!(payload["presentation"]["location_label"], json!("Europe"));
}

#[test]
fn recent_jobs_query_accepts_known_source() {
    let uri: Uri = "/api/v1/jobs/recent?source=djinni"
        .parse()
        .expect("uri should parse");
    let Query(query) =
        Query::<RecentJobsQuery>::try_from_uri(&uri).expect("query should deserialize");

    assert_eq!(query.source, Some(SourceId::Djinni));
}

#[test]
fn recent_jobs_query_rejects_unknown_source() {
    let uri: Uri = "/api/v1/jobs/recent?source=linkedin"
        .parse()
        .expect("uri should parse");

    let result = Query::<RecentJobsQuery>::try_from_uri(&uri);

    assert!(result.is_err());
}
