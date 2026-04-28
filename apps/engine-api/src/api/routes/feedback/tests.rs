use axum::Extension;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{Json, body};
use serde_json::{Value, json};

use crate::api::dto::feedback::UpdateCompanyFeedbackRequest;
use crate::api::error::ApiJson;
use crate::api::middleware::auth::AuthUser;
use crate::domain::feedback::model::{
    CompanyFeedbackRecord, CompanyFeedbackStatus, JobFeedbackFlags, JobFeedbackRecord,
};
use crate::domain::job::model::{Job, JobLifecycleStage, JobSourceVariant, JobView};
use crate::domain::profile::model::Profile;
use crate::domain::user_event::model::UserEventType;
use crate::services::applications::{ApplicationsService, ApplicationsServiceStub};
use crate::services::feedback::{FeedbackService, FeedbackServiceStub};
use crate::services::jobs::{JobsService, JobsServiceStub};
use crate::services::profiles::{ProfilesService, ProfilesServiceStub};
use crate::services::resumes::{ResumesService, ResumesServiceStub};
use crate::services::user_events::{UserEventsService, UserEventsServiceStub};
use crate::state::AppState;

use super::{
    add_company_blacklist, clear_all_hidden_jobs, hide_job, list_feedback, mark_job_bad_fit,
    save_job, unhide_job, unmark_job_bad_fit, unsave_job,
};

fn sample_profile() -> Profile {
    Profile {
        id: "profile-1".to_string(),
        name: "Jane Doe".to_string(),
        email: "jane@example.com".to_string(),
        location: Some("Kyiv".to_string()),
        raw_text: "Senior backend engineer".to_string(),
        analysis: None,
        years_of_experience: None,
        salary_min: None,
        salary_max: None,
        salary_currency: "USD".to_string(),
        languages: vec![],
        preferred_work_mode: None,
        preferred_language: None,
        search_preferences: None,
        created_at: "2026-04-14T00:00:00Z".to_string(),
        updated_at: "2026-04-14T00:00:00Z".to_string(),
        skills_updated_at: None,
    }
}

fn sample_job(job_id: &str, company_name: &str) -> Job {
    Job {
        id: job_id.to_string(),
        title: "Senior Backend Developer".to_string(),
        company_name: company_name.to_string(),
        location: None,
        remote_type: Some("remote".to_string()),
        seniority: Some("senior".to_string()),
        description_text: "Rust and Postgres".to_string(),
        salary_min: None,
        salary_max: None,
        salary_currency: None,
        language: None,
        posted_at: None,
        last_seen_at: "2026-04-14T00:00:00Z".to_string(),
        is_active: true,
    }
}

fn sample_job_view(job_id: &str, company_name: &str) -> JobView {
    JobView {
        job: sample_job(job_id, company_name),
        first_seen_at: "2026-04-12T00:00:00Z".to_string(),
        inactivated_at: None,
        reactivated_at: None,
        lifecycle_stage: JobLifecycleStage::Active,
        primary_variant: Some(JobSourceVariant {
            source: "djinni".to_string(),
            source_job_id: format!("djinni-{job_id}"),
            source_url: format!("https://djinni.co/jobs/{job_id}"),
            raw_payload: None,
            fetched_at: "2026-04-14T00:00:00Z".to_string(),
            last_seen_at: "2026-04-14T00:00:00Z".to_string(),
            is_active: true,
            inactivated_at: None,
        }),
    }
}

fn test_state() -> AppState {
    test_state_with_feedback(FeedbackService::for_tests(FeedbackServiceStub::default()))
}

fn test_state_with_feedback(feedback_service: FeedbackService) -> AppState {
    AppState::for_services(
        ProfilesService::for_tests(ProfilesServiceStub::default().with_profile(sample_profile())),
        JobsService::for_tests(
            JobsServiceStub::default()
                .with_job(sample_job("job-1", "NovaLedger"))
                .with_job_view(sample_job_view("job-1", "NovaLedger")),
        ),
        ApplicationsService::for_tests(ApplicationsServiceStub::default()),
        ResumesService::for_tests(ResumesServiceStub::default()),
    )
    .with_feedback_service(feedback_service)
}

#[tokio::test]
async fn save_and_bad_fit_persist_in_feedback_overview() {
    let state = test_state();

    let _ = save_job(
        State(state.clone()),
        None,
        Path(("profile-1".to_string(), "job-1".to_string())),
    )
    .await
    .expect("save should succeed");
    let _ = mark_job_bad_fit(
        State(state.clone()),
        None,
        Path(("profile-1".to_string(), "job-1".to_string())),
    )
    .await
    .expect("bad fit should succeed");

    let response = list_feedback(State(state), None, Path("profile-1".to_string()))
        .await
        .expect("listing feedback should succeed")
        .into_response();

    let body = body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("feedback body should be readable");
    let payload: Value = serde_json::from_slice(&body).expect("feedback body should be valid JSON");

    assert_eq!(payload["jobs"].as_array().map(Vec::len), Some(1));
    assert_eq!(payload["jobs"][0]["saved"], json!(true));
    assert_eq!(payload["jobs"][0]["bad_fit"], json!(true));
}

#[tokio::test]
async fn add_company_blacklist_is_visible_in_feedback_overview() {
    let state = test_state();

    let _ = add_company_blacklist(
        State(state.clone()),
        None,
        Path("profile-1".to_string()),
        ApiJson(UpdateCompanyFeedbackRequest {
            company_name: "NovaLedger".to_string(),
        }),
    )
    .await
    .expect("blacklist should succeed");

    let response = list_feedback(State(state), None, Path("profile-1".to_string()))
        .await
        .expect("listing feedback should succeed")
        .into_response();

    let body = body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("feedback body should be readable");
    let payload: Value = serde_json::from_slice(&body).expect("feedback body should be valid JSON");

    assert_eq!(payload["companies"].as_array().map(Vec::len), Some(1));
    assert_eq!(payload["companies"][0]["status"], json!("blacklist"));
}

#[tokio::test]
async fn list_feedback_uses_existing_stub_records() {
    let state = test_state().with_feedback_service(FeedbackService::for_tests(
        FeedbackServiceStub::default()
            .with_job_feedback(JobFeedbackRecord {
                profile_id: "profile-1".to_string(),
                job_id: "job-1".to_string(),
                saved: true,
                hidden: false,
                bad_fit: false,
                salary_signal: None,
                interest_rating: None,
                work_mode_signal: None,
                legitimacy_signal: None,
                created_at: "2026-04-14T00:00:00Z".to_string(),
                updated_at: "2026-04-14T00:00:00Z".to_string(),
            })
            .with_company_feedback(CompanyFeedbackRecord {
                profile_id: "profile-1".to_string(),
                company_name: "NovaLedger".to_string(),
                normalized_company_name: "novaledger".to_string(),
                status: CompanyFeedbackStatus::Blacklist,
                created_at: "2026-04-14T00:00:00Z".to_string(),
                updated_at: "2026-04-14T00:00:00Z".to_string(),
            }),
    ));

    let Json(response) = list_feedback(State(state), None, Path("profile-1".to_string()))
        .await
        .expect("listing feedback should succeed");

    assert_eq!(response.jobs.len(), 1);
    assert_eq!(response.companies.len(), 1);
    assert!(response.jobs[0].saved);
    assert_eq!(response.companies[0].status, "blacklist");
}

#[tokio::test]
async fn unsave_job_clears_saved_flag() {
    let state = test_state().with_feedback_service(FeedbackService::for_tests(
        FeedbackServiceStub::default().with_job_feedback(JobFeedbackRecord {
            profile_id: "profile-1".to_string(),
            job_id: "job-1".to_string(),
            saved: true,
            hidden: false,
            bad_fit: false,
            salary_signal: None,
            interest_rating: None,
            work_mode_signal: None,
            legitimacy_signal: None,
            created_at: "2026-04-14T00:00:00Z".to_string(),
            updated_at: "2026-04-14T00:00:00Z".to_string(),
        }),
    ));

    unsave_job(
        State(state.clone()),
        None,
        Path(("profile-1".to_string(), "job-1".to_string())),
    )
    .await
    .expect("unsave should succeed");

    let Json(overview) = list_feedback(State(state), None, Path("profile-1".to_string()))
        .await
        .expect("listing feedback should succeed");

    assert_eq!(overview.jobs.len(), 1);
    assert!(
        !overview.jobs[0].saved,
        "saved should be cleared after unsave"
    );
}

#[tokio::test]
async fn unhide_job_clears_hidden_flag() {
    let state = test_state().with_feedback_service(FeedbackService::for_tests(
        FeedbackServiceStub::default().with_job_feedback(JobFeedbackRecord {
            profile_id: "profile-1".to_string(),
            job_id: "job-1".to_string(),
            saved: false,
            hidden: true,
            bad_fit: false,
            salary_signal: None,
            interest_rating: None,
            work_mode_signal: None,
            legitimacy_signal: None,
            created_at: "2026-04-14T00:00:00Z".to_string(),
            updated_at: "2026-04-14T00:00:00Z".to_string(),
        }),
    ));

    unhide_job(
        State(state.clone()),
        None,
        Path(("profile-1".to_string(), "job-1".to_string())),
    )
    .await
    .expect("unhide should succeed");

    let Json(overview) = list_feedback(State(state), None, Path("profile-1".to_string()))
        .await
        .expect("listing feedback should succeed");

    assert_eq!(overview.jobs.len(), 1);
    assert!(
        !overview.jobs[0].hidden,
        "hidden should be cleared after unhide"
    );
}

#[tokio::test]
async fn unmark_bad_fit_clears_bad_fit_flag() {
    let state = test_state().with_feedback_service(FeedbackService::for_tests(
        FeedbackServiceStub::default().with_job_feedback(JobFeedbackRecord {
            profile_id: "profile-1".to_string(),
            job_id: "job-1".to_string(),
            saved: false,
            hidden: false,
            bad_fit: true,
            salary_signal: None,
            interest_rating: None,
            work_mode_signal: None,
            legitimacy_signal: None,
            created_at: "2026-04-14T00:00:00Z".to_string(),
            updated_at: "2026-04-14T00:00:00Z".to_string(),
        }),
    ));

    unmark_job_bad_fit(
        State(state.clone()),
        None,
        Path(("profile-1".to_string(), "job-1".to_string())),
    )
    .await
    .expect("unmark bad fit should succeed");

    let Json(overview) = list_feedback(State(state), None, Path("profile-1".to_string()))
        .await
        .expect("listing feedback should succeed");

    assert_eq!(overview.jobs.len(), 1);
    assert!(
        !overview.jobs[0].bad_fit,
        "bad_fit should be cleared after unmark"
    );
}

#[tokio::test]
async fn undo_on_nonexistent_feedback_succeeds_idempotently() {
    let state = test_state();

    let result = unsave_job(
        State(state),
        None,
        Path(("profile-1".to_string(), "job-1".to_string())),
    )
    .await;

    assert!(
        result.is_ok(),
        "unsave on a job with no feedback should succeed"
    );
}

#[tokio::test]
async fn undo_preserves_other_flags() {
    let state = test_state().with_feedback_service(FeedbackService::for_tests(
        FeedbackServiceStub::default().with_job_feedback(JobFeedbackRecord {
            profile_id: "profile-1".to_string(),
            job_id: "job-1".to_string(),
            saved: true,
            hidden: true,
            bad_fit: true,
            salary_signal: None,
            interest_rating: None,
            work_mode_signal: None,
            legitimacy_signal: None,
            created_at: "2026-04-14T00:00:00Z".to_string(),
            updated_at: "2026-04-14T00:00:00Z".to_string(),
        }),
    ));

    unsave_job(
        State(state.clone()),
        None,
        Path(("profile-1".to_string(), "job-1".to_string())),
    )
    .await
    .expect("unsave should succeed");

    let Json(overview) = list_feedback(State(state), None, Path("profile-1".to_string()))
        .await
        .expect("listing feedback should succeed");

    assert!(!overview.jobs[0].saved, "saved should be cleared");
    assert!(overview.jobs[0].hidden, "hidden should be untouched");
    assert!(overview.jobs[0].bad_fit, "bad_fit should be untouched");
}

#[tokio::test]
async fn feedback_overview_summary_counts_are_correct() {
    let state = test_state().with_feedback_service(FeedbackService::for_tests(
        FeedbackServiceStub::default()
            .with_job_feedback(JobFeedbackRecord {
                profile_id: "profile-1".to_string(),
                job_id: "job-1".to_string(),
                saved: true,
                hidden: false,
                bad_fit: true,
                salary_signal: None,
                interest_rating: None,
                work_mode_signal: None,
                legitimacy_signal: None,
                created_at: "2026-04-14T00:00:00Z".to_string(),
                updated_at: "2026-04-14T00:00:00Z".to_string(),
            })
            .with_company_feedback(CompanyFeedbackRecord {
                profile_id: "profile-1".to_string(),
                company_name: "GoodCorp".to_string(),
                normalized_company_name: "goodcorp".to_string(),
                status: CompanyFeedbackStatus::Whitelist,
                created_at: "2026-04-14T00:00:00Z".to_string(),
                updated_at: "2026-04-14T00:00:00Z".to_string(),
            })
            .with_company_feedback(CompanyFeedbackRecord {
                profile_id: "profile-1".to_string(),
                company_name: "BadCorp".to_string(),
                normalized_company_name: "badcorp".to_string(),
                status: CompanyFeedbackStatus::Blacklist,
                created_at: "2026-04-14T00:00:00Z".to_string(),
                updated_at: "2026-04-14T00:00:00Z".to_string(),
            }),
    ));

    let Json(overview) = list_feedback(State(state), None, Path("profile-1".to_string()))
        .await
        .expect("listing feedback should succeed");

    assert_eq!(overview.summary.saved_jobs_count, 1);
    assert_eq!(overview.summary.hidden_jobs_count, 0);
    assert_eq!(overview.summary.bad_fit_jobs_count, 1);
    assert_eq!(overview.summary.whitelisted_companies_count, 1);
    assert_eq!(overview.summary.blacklisted_companies_count, 1);
}

#[tokio::test]
async fn feedback_actions_create_expected_user_events() {
    let state = test_state();

    let _ = save_job(
        State(state.clone()),
        None,
        Path(("profile-1".to_string(), "job-1".to_string())),
    )
    .await
    .expect("save should succeed");
    unsave_job(
        State(state.clone()),
        None,
        Path(("profile-1".to_string(), "job-1".to_string())),
    )
    .await
    .expect("unsave should succeed");
    let _ = hide_job(
        State(state.clone()),
        None,
        Path(("profile-1".to_string(), "job-1".to_string())),
    )
    .await
    .expect("hide should succeed");
    unhide_job(
        State(state.clone()),
        None,
        Path(("profile-1".to_string(), "job-1".to_string())),
    )
    .await
    .expect("unhide should succeed");
    let _ = mark_job_bad_fit(
        State(state.clone()),
        None,
        Path(("profile-1".to_string(), "job-1".to_string())),
    )
    .await
    .expect("bad fit should succeed");
    unmark_job_bad_fit(
        State(state.clone()),
        None,
        Path(("profile-1".to_string(), "job-1".to_string())),
    )
    .await
    .expect("remove bad fit should succeed");

    let events = state
        .user_events_service
        .list_by_profile("profile-1")
        .await
        .expect("events should be queryable");
    let event_types: Vec<UserEventType> =
        events.into_iter().map(|event| event.event_type).collect();

    assert!(event_types.contains(&UserEventType::JobSaved));
    assert!(event_types.contains(&UserEventType::JobUnsaved));
    assert!(event_types.contains(&UserEventType::JobHidden));
    assert!(event_types.contains(&UserEventType::JobUnhidden));
    assert!(event_types.contains(&UserEventType::JobBadFit));
    assert!(event_types.contains(&UserEventType::JobBadFitRemoved));
}

#[tokio::test]
async fn company_blacklist_creates_user_event() {
    let state = test_state();

    let _ = add_company_blacklist(
        State(state.clone()),
        None,
        Path("profile-1".to_string()),
        ApiJson(UpdateCompanyFeedbackRequest {
            company_name: "NovaLedger".to_string(),
        }),
    )
    .await
    .expect("blacklist should succeed");

    let events = state
        .user_events_service
        .list_by_profile("profile-1")
        .await
        .expect("events should be queryable");

    assert!(
        events
            .iter()
            .any(|event| event.event_type == UserEventType::CompanyBlacklisted),
        "company blacklist action should emit an immutable user event"
    );
}

#[tokio::test]
async fn save_job_still_succeeds_when_event_logging_fails_softly() {
    let state = test_state().with_user_events_service(UserEventsService::for_tests(
        UserEventsServiceStub::default().with_database_disabled(),
    ));

    let result = save_job(
        State(state.clone()),
        None,
        Path(("profile-1".to_string(), "job-1".to_string())),
    )
    .await;

    assert!(
        result.is_ok(),
        "feedback write should not fail when event logging is unavailable"
    );
}

#[tokio::test]
async fn non_owner_gets_forbidden_on_feedback_list() {
    let state = test_state();

    let response = list_feedback(
        State(state),
        Some(Extension(AuthUser {
            profile_id: "other-profile".to_string(),
        })),
        Path("profile-1".to_string()),
    )
    .await
    .expect_err("non-owner should be rejected")
    .into_response();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn missing_profile_returns_not_found_on_feedback_list() {
    let state = AppState::for_services(
        ProfilesService::for_tests(ProfilesServiceStub::default()),
        JobsService::for_tests(JobsServiceStub::default()),
        ApplicationsService::for_tests(ApplicationsServiceStub::default()),
        ResumesService::for_tests(ResumesServiceStub::default()),
    );

    let response = list_feedback(State(state), None, Path("nonexistent-profile".to_string()))
        .await
        .expect_err("missing profile should return 404")
        .into_response();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn clear_all_hidden_jobs_clears_only_current_profile_hidden_flags() {
    let feedback_stub = FeedbackServiceStub::default();

    feedback_stub
        .upsert_job_feedback(
            "profile-1",
            "job-hidden-1",
            JobFeedbackFlags {
                hidden: true,
                ..Default::default()
            },
        )
        .expect("seed hidden feedback");

    feedback_stub
        .upsert_job_feedback(
            "profile-1",
            "job-saved",
            JobFeedbackFlags {
                saved: true,
                hidden: true,
                ..Default::default()
            },
        )
        .expect("seed saved hidden feedback");

    feedback_stub
        .upsert_job_feedback(
            "profile-2",
            "job-other-profile",
            JobFeedbackFlags {
                hidden: true,
                ..Default::default()
            },
        )
        .expect("seed other profile feedback");

    let state = test_state_with_feedback(FeedbackService::for_tests(feedback_stub));

    let status = clear_all_hidden_jobs(
        State(state.clone()),
        Some(Extension(AuthUser {
            profile_id: "profile-1".to_string(),
        })),
        Path("profile-1".to_string()),
    )
    .await
    .expect("clear hidden jobs should succeed");

    assert_eq!(status, StatusCode::NO_CONTENT);

    let profile_1_feedback = state
        .feedback_service
        .list_job_feedback("profile-1")
        .await
        .expect("profile feedback should be readable");

    assert!(profile_1_feedback.iter().all(|record| !record.hidden));
    assert!(
        profile_1_feedback
            .iter()
            .any(|record| record.job_id == "job-saved" && record.saved)
    );

    let profile_2_feedback = state
        .feedback_service
        .list_job_feedback("profile-2")
        .await
        .expect("other profile feedback should be readable");

    assert!(
        profile_2_feedback
            .iter()
            .any(|record| record.job_id == "job-other-profile" && record.hidden)
    );
}

#[tokio::test]
async fn clear_all_hidden_jobs_rejects_profile_mismatch() {
    let state =
        test_state_with_feedback(FeedbackService::for_tests(FeedbackServiceStub::default()));

    let error = clear_all_hidden_jobs(
        State(state),
        Some(Extension(AuthUser {
            profile_id: "profile-1".to_string(),
        })),
        Path("profile-2".to_string()),
    )
    .await
    .expect_err("profile mismatch should be rejected");

    assert_eq!(error.into_response().status(), StatusCode::FORBIDDEN);
}
