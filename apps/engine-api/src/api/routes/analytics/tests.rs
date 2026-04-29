use axum::Json;
use axum::extract::{Path, State};
use serde_json::json;

use crate::domain::analytics::model::JobSourceCount;
use crate::domain::feedback::model::{
    CompanyFeedbackRecord, CompanyFeedbackStatus, JobFeedbackRecord,
};
use crate::domain::job::model::{Job, JobFeedSummary, JobLifecycleStage, JobView};
use crate::domain::profile::model::{Profile, ProfileAnalysis};
use crate::domain::role::RoleId;
use crate::domain::user_event::model::{UserEventRecord, UserEventType};
use crate::services::applications::{ApplicationsService, ApplicationsServiceStub};
use crate::services::feedback::{FeedbackService, FeedbackServiceStub};
use crate::services::jobs::{JobsService, JobsServiceStub};
use crate::services::profiles::{ProfilesService, ProfilesServiceStub};
use crate::services::resumes::{ResumesService, ResumesServiceStub};
use crate::services::user_events::{UserEventsService, UserEventsServiceStub};
use crate::state::AppState;

use super::{get_analytics_summary, get_funnel_summary, get_llm_context};

fn sample_profile_with_analysis() -> Profile {
    Profile {
        id: "profile-1".to_string(),
        name: "Jane".to_string(),
        email: "jane@example.com".to_string(),
        location: None,
        raw_text: "Senior Rust backend engineer".to_string(),
        analysis: Some(ProfileAnalysis {
            summary: "Senior backend engineer with Rust expertise".to_string(),
            primary_role: RoleId::BackendEngineer,
            seniority: "senior".to_string(),
            skills: vec!["rust".to_string(), "postgres".to_string()],
            keywords: vec!["backend".to_string(), "distributed".to_string()],
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
        created_at: "2026-04-14T00:00:00Z".to_string(),
        updated_at: "2026-04-14T00:00:00Z".to_string(),
        skills_updated_at: None,
        portfolio_url: None,
        github_url: None,
        linkedin_url: None,
    }
}

fn test_state() -> AppState {
    AppState::for_services(
        ProfilesService::for_tests(
            ProfilesServiceStub::default().with_profile(sample_profile_with_analysis()),
        ),
        JobsService::for_tests(
            JobsServiceStub::default()
                .with_job_view(sample_job_view(
                    "job-1",
                    "Senior Backend Developer",
                    "NovaLedger",
                ))
                .with_job_view(sample_job_view("job-2", "Legacy Support Engineer", "OldCo"))
                .with_feed_summary(JobFeedSummary {
                    total_jobs: 10,
                    active_jobs: 6,
                    inactive_jobs: 3,
                    reactivated_jobs: 1,
                    last_ingested_at: Some("2025-01-15T10:00:00".to_string()),
                })
                .with_jobs_by_source(vec![
                    JobSourceCount {
                        source: "djinni".to_string(),
                        count: 7,
                        last_seen: "2025-01-15T10:00:00".to_string(),
                    },
                    JobSourceCount {
                        source: "work_ua".to_string(),
                        count: 3,
                        last_seen: "2025-01-14T08:00:00".to_string(),
                    },
                ]),
        ),
        ApplicationsService::for_tests(ApplicationsServiceStub::default()),
        ResumesService::for_tests(ResumesServiceStub::default()),
    )
}

fn sample_job_view(id: &str, title: &str, company_name: &str) -> JobView {
    JobView {
        job: Job {
            id: id.to_string(),
            title: title.to_string(),
            company_name: company_name.to_string(),
            location: None,
            remote_type: Some("remote".to_string()),
            seniority: Some("senior".to_string()),
            description_text: "Rust and Postgres".to_string(),
            salary_min: None,
            salary_max: None,
            salary_currency: None,
            language: None,
            posted_at: Some("2026-04-12T09:00:00Z".to_string()),
            last_seen_at: "2026-04-14T09:00:00Z".to_string(),
            is_active: true,
        },
        first_seen_at: "2026-04-12T09:00:00Z".to_string(),
        inactivated_at: None,
        reactivated_at: None,
        lifecycle_stage: JobLifecycleStage::Active,
        primary_variant: None,
    }
}

fn with_feedback(state: AppState) -> AppState {
    state.with_feedback_service(FeedbackService::for_tests(
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
            .with_job_feedback(JobFeedbackRecord {
                profile_id: "profile-1".to_string(),
                job_id: "job-2".to_string(),
                saved: false,
                hidden: true,
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
                notes: String::new(),
                created_at: "2026-04-14T00:00:00Z".to_string(),
                updated_at: "2026-04-14T00:00:00Z".to_string(),
            })
            .with_company_feedback(CompanyFeedbackRecord {
                profile_id: "profile-1".to_string(),
                company_name: "BadCorp".to_string(),
                normalized_company_name: "badcorp".to_string(),
                status: CompanyFeedbackStatus::Blacklist,
                notes: String::new(),
                created_at: "2026-04-14T00:00:00Z".to_string(),
                updated_at: "2026-04-14T00:00:00Z".to_string(),
            }),
    ))
}

fn event(
    id: &str,
    event_type: UserEventType,
    job_id: &str,
    source: Option<&str>,
) -> UserEventRecord {
    UserEventRecord {
        id: id.to_string(),
        profile_id: "profile-1".to_string(),
        event_type,
        job_id: Some(job_id.to_string()),
        company_name: Some("NovaLedger".to_string()),
        source: source.map(str::to_string),
        role_family: Some("engineering".to_string()),
        payload_json: Some(json!({ "surface": "test" })),
        created_at: "2026-04-15T00:00:00Z".to_string(),
    }
}

fn with_funnel_events(state: AppState) -> AppState {
    state.with_user_events_service(UserEventsService::for_tests(
        UserEventsServiceStub::default()
            .with_event(event(
                "evt-1",
                UserEventType::JobImpression,
                "job-1",
                Some("djinni"),
            ))
            .with_event(event(
                "evt-2",
                UserEventType::JobImpression,
                "job-2",
                Some("djinni"),
            ))
            .with_event(event(
                "evt-3",
                UserEventType::JobImpression,
                "job-3",
                Some("work_ua"),
            ))
            .with_event(event(
                "evt-4",
                UserEventType::JobOpened,
                "job-1",
                Some("djinni"),
            ))
            .with_event(event(
                "evt-5",
                UserEventType::JobOpened,
                "job-2",
                Some("djinni"),
            ))
            .with_event(event(
                "evt-6",
                UserEventType::JobSaved,
                "job-1",
                Some("djinni"),
            ))
            .with_event(event(
                "evt-7",
                UserEventType::JobHidden,
                "job-3",
                Some("work_ua"),
            ))
            .with_event(event(
                "evt-8",
                UserEventType::JobBadFit,
                "job-3",
                Some("work_ua"),
            ))
            .with_event(event(
                "evt-9",
                UserEventType::ApplicationCreated,
                "job-1",
                Some("djinni"),
            ))
            .with_event(event(
                "evt-10",
                UserEventType::FitExplanationRequested,
                "job-1",
                Some("djinni"),
            ))
            .with_event(event(
                "evt-11",
                UserEventType::ApplicationCoachRequested,
                "job-1",
                Some("djinni"),
            ))
            .with_event(event(
                "evt-12",
                UserEventType::CoverLetterDraftRequested,
                "job-1",
                Some("djinni"),
            ))
            .with_event(event(
                "evt-13",
                UserEventType::InterviewPrepRequested,
                "job-1",
                Some("djinni"),
            )),
    ))
}

#[tokio::test]
async fn analytics_summary_feedback_counts_are_correct() {
    let state = with_feedback(test_state());

    let Json(summary) = get_analytics_summary(State(state), None, Path("profile-1".to_string()))
        .await
        .expect("analytics summary should succeed");

    assert_eq!(summary.feedback.saved_jobs_count, 1);
    assert_eq!(summary.feedback.hidden_jobs_count, 1);
    assert_eq!(summary.feedback.bad_fit_jobs_count, 1);
    assert_eq!(summary.feedback.whitelisted_companies_count, 1);
    assert_eq!(summary.feedback.blacklisted_companies_count, 1);
}

#[tokio::test]
async fn analytics_summary_source_aggregation_is_correct() {
    let state = test_state();

    let Json(summary) = get_analytics_summary(State(state), None, Path("profile-1".to_string()))
        .await
        .expect("analytics summary should succeed");

    assert_eq!(summary.jobs_by_source.len(), 2);
    assert_eq!(summary.jobs_by_source[0].source, "djinni");
    assert_eq!(summary.jobs_by_source[0].count, 7);
    assert_eq!(summary.jobs_by_source[1].source, "work_ua");
    assert_eq!(summary.jobs_by_source[1].count, 3);
}

#[tokio::test]
async fn analytics_summary_lifecycle_distribution_is_correct() {
    let state = test_state();

    let Json(summary) = get_analytics_summary(State(state), None, Path("profile-1".to_string()))
        .await
        .expect("analytics summary should succeed");

    assert_eq!(summary.jobs_by_lifecycle.total, 10);
    assert_eq!(summary.jobs_by_lifecycle.active, 6);
    assert_eq!(summary.jobs_by_lifecycle.inactive, 3);
    assert_eq!(summary.jobs_by_lifecycle.reactivated, 1);
}

#[tokio::test]
async fn analytics_summary_top_matched_come_from_profile_analysis() {
    let state = test_state();

    let Json(summary) = get_analytics_summary(State(state), None, Path("profile-1".to_string()))
        .await
        .expect("analytics summary should succeed");

    assert_eq!(summary.top_matched_roles, vec!["backend_engineer"]);
    assert_eq!(summary.top_matched_skills, vec!["rust", "postgres"]);
    assert_eq!(summary.top_matched_keywords, vec!["backend", "distributed"]);
}

#[tokio::test]
async fn funnel_summary_counts_impressions_and_actions() {
    let state = with_funnel_events(test_state());

    let Json(summary) = get_funnel_summary(State(state), None, Path("profile-1".to_string()))
        .await
        .expect("funnel summary should succeed");

    assert_eq!(summary.impression_count, 3);
    assert_eq!(summary.open_count, 2);
    assert_eq!(summary.save_count, 1);
    assert_eq!(summary.hide_count, 1);
    assert_eq!(summary.bad_fit_count, 1);
    assert_eq!(summary.application_created_count, 1);
    assert_eq!(summary.fit_explanation_requested_count, 1);
    assert_eq!(summary.application_coach_requested_count, 1);
    assert_eq!(summary.cover_letter_draft_requested_count, 1);
    assert_eq!(summary.interview_prep_requested_count, 1);
    assert_eq!(summary.impressions_by_source[0].source, "djinni");
    assert_eq!(summary.impressions_by_source[0].count, 2);
    assert_eq!(summary.applications_by_source[0].source, "djinni");
    assert_eq!(summary.applications_by_source[0].count, 1);
}

#[tokio::test]
async fn funnel_summary_derived_ratios_are_correct() {
    let state = with_funnel_events(test_state());

    let Json(summary) = get_funnel_summary(State(state), None, Path("profile-1".to_string()))
        .await
        .expect("funnel summary should succeed");

    assert!((summary.conversion_rates.open_rate_from_impressions - (2.0 / 3.0)).abs() < 1e-9);
    assert!((summary.conversion_rates.save_rate_from_opens - 0.5).abs() < 1e-9);
    assert!((summary.conversion_rates.application_rate_from_saves - 1.0).abs() < 1e-9);
}

#[tokio::test]
async fn funnel_summary_avoids_divide_by_zero() {
    let state = test_state().with_user_events_service(UserEventsService::for_tests(
        UserEventsServiceStub::default().with_event(event(
            "evt-1",
            UserEventType::JobSaved,
            "job-1",
            Some("djinni"),
        )),
    ));

    let Json(summary) = get_funnel_summary(State(state), None, Path("profile-1".to_string()))
        .await
        .expect("funnel summary should succeed");

    assert_eq!(summary.impression_count, 0);
    assert_eq!(summary.open_count, 0);
    assert_eq!(summary.conversion_rates.open_rate_from_impressions, 0.0);
    assert_eq!(summary.conversion_rates.save_rate_from_opens, 0.0);
    assert_eq!(summary.conversion_rates.application_rate_from_saves, 0.0);
}

#[tokio::test]
async fn llm_context_payload_shape_is_complete() {
    let state = with_feedback(test_state());

    let Json(ctx) = get_llm_context(State(state), None, Path("profile-1".to_string()))
        .await
        .expect("llm context should succeed");

    assert_eq!(ctx.profile_id, "profile-1");
    assert!(ctx.analyzed_profile.is_some());
    assert_eq!(ctx.profile_skills, vec!["rust", "postgres"]);
    assert_eq!(ctx.profile_keywords, vec!["backend", "distributed"]);
    assert_eq!(ctx.jobs_feed_summary.total, 10);
    assert_eq!(ctx.feedback_summary.saved_jobs_count, 1);
}

#[tokio::test]
async fn llm_context_positive_evidence_includes_saved_jobs_and_whitelisted_companies() {
    let state = with_feedback(test_state());

    let Json(ctx) = get_llm_context(State(state), None, Path("profile-1".to_string()))
        .await
        .expect("llm context should succeed");

    let saved = ctx
        .top_positive_evidence
        .iter()
        .find(|e| e.entry_type == "saved_job");
    let whitelisted = ctx
        .top_positive_evidence
        .iter()
        .find(|e| e.entry_type == "whitelisted_company");

    assert!(saved.is_some(), "should include saved job evidence");
    assert_eq!(
        saved.unwrap().label,
        "Senior Backend Developer at NovaLedger"
    );
    assert!(
        whitelisted.is_some(),
        "should include whitelisted company evidence"
    );
    assert_eq!(whitelisted.unwrap().label, "GoodCorp");
}

#[tokio::test]
async fn llm_context_negative_evidence_includes_bad_fit_jobs_and_blacklisted_companies() {
    let state = with_feedback(test_state());

    let Json(ctx) = get_llm_context(State(state), None, Path("profile-1".to_string()))
        .await
        .expect("llm context should succeed");

    let bad_fit = ctx
        .top_negative_evidence
        .iter()
        .find(|e| e.entry_type == "bad_fit_job");
    let blacklisted = ctx
        .top_negative_evidence
        .iter()
        .find(|e| e.entry_type == "blacklisted_company");

    assert!(bad_fit.is_some(), "should include bad fit job evidence");
    assert_eq!(bad_fit.unwrap().label, "Legacy Support Engineer at OldCo");
    assert!(
        blacklisted.is_some(),
        "should include blacklisted company evidence"
    );
    assert_eq!(blacklisted.unwrap().label, "BadCorp");
}

#[tokio::test]
async fn llm_context_analyzed_profile_fields_match_profile_analysis() {
    let state = test_state();

    let Json(ctx) = get_llm_context(State(state), None, Path("profile-1".to_string()))
        .await
        .expect("llm context should succeed");

    let analysis = ctx
        .analyzed_profile
        .expect("analyzed_profile should be present");
    assert_eq!(analysis.primary_role, "backend_engineer");
    assert_eq!(analysis.seniority, "senior");
    assert!(analysis.skills.contains(&"rust".to_string()));
}

#[tokio::test]
async fn ingestion_stats_returns_feed_totals_and_last_ingested_at() {
    use super::get_ingestion_stats;

    let state = test_state();

    let axum::Json(stats) = get_ingestion_stats(State(state))
        .await
        .expect("ingestion stats should succeed");

    assert_eq!(stats.total_jobs, 10);
    assert_eq!(stats.active_jobs, 6);
    assert_eq!(stats.inactive_jobs, 3);
    assert_eq!(
        stats.last_ingested_at.as_deref(),
        Some("2025-01-15T10:00:00")
    );
}

#[tokio::test]
async fn ingestion_stats_sources_include_count_and_last_seen() {
    use super::get_ingestion_stats;

    let state = test_state();

    let axum::Json(stats) = get_ingestion_stats(State(state))
        .await
        .expect("ingestion stats should succeed");

    assert_eq!(stats.sources.len(), 2);
    assert_eq!(stats.sources[0].source, "djinni");
    assert_eq!(stats.sources[0].count, 7);
    assert_eq!(stats.sources[0].last_seen, "2025-01-15T10:00:00");
    assert_eq!(stats.sources[1].source, "work_ua");
    assert_eq!(stats.sources[1].count, 3);
    assert_eq!(stats.sources[1].last_seen, "2025-01-14T08:00:00");
}

#[tokio::test]
async fn ingestion_stats_returns_none_when_no_jobs_ingested() {
    use super::get_ingestion_stats;

    let state = AppState::for_services(
        ProfilesService::for_tests(ProfilesServiceStub::default()),
        JobsService::for_tests(JobsServiceStub::default()),
        ApplicationsService::for_tests(ApplicationsServiceStub::default()),
        ResumesService::for_tests(ResumesServiceStub::default()),
    );

    let axum::Json(stats) = get_ingestion_stats(State(state))
        .await
        .expect("ingestion stats should succeed");

    assert_eq!(stats.total_jobs, 0);
    assert_eq!(stats.active_jobs, 0);
    assert_eq!(stats.inactive_jobs, 0);
    assert!(stats.last_ingested_at.is_none());
    assert!(stats.sources.is_empty());
}
