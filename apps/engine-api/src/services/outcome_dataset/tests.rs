use std::collections::BTreeMap;

use crate::domain::application::model::{Application, ApplicationOutcome};
use crate::domain::feedback::model::JobFeedbackState;
use crate::domain::job::model::{Job, JobLifecycleStage, JobSourceVariant, JobView};
use crate::domain::profile::model::{Profile, ProfileAnalysis};
use crate::domain::role::RoleId;
use crate::domain::user_event::model::{UserEventRecord, UserEventType};
use crate::services::behavior::BehaviorService;
use crate::services::funnel::FunnelService;
use crate::services::search_ranking::SearchRankingService;

use super::{OutcomeDatasetService, OutcomeLabel, outcome_job_ids};

fn profile() -> Profile {
    Profile {
        id: "profile-1".to_string(),
        name: "Jane Doe".to_string(),
        email: "jane@example.com".to_string(),
        location: Some("Kyiv".to_string()),
        raw_text: "Senior backend engineer with Rust and Postgres".to_string(),
        analysis: Some(ProfileAnalysis {
            summary: "Senior backend engineer".to_string(),
            primary_role: RoleId::BackendEngineer,
            seniority: "senior".to_string(),
            skills: vec!["Rust".to_string(), "Postgres".to_string()],
            keywords: vec!["backend".to_string(), "platform".to_string()],
        }),
        years_of_experience: None,
        salary_min: None,
        salary_max: None,
        salary_currency: "USD".to_string(),
        languages: vec![],
        preferred_locations: vec![],
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

fn job_view(id: &str, title: &str, source: &str) -> JobView {
    JobView {
        job: Job {
            id: id.to_string(),
            title: title.to_string(),
            company_name: "NovaLedger".to_string(),
            location: None,
            remote_type: Some("remote".to_string()),
            seniority: Some("senior".to_string()),
            description_text: "Build backend Rust services on Postgres".to_string(),
            salary_min: None,
            salary_max: None,
            salary_currency: None,
            language: None,
            posted_at: Some("2026-04-14T00:00:00Z".to_string()),
            last_seen_at: "2026-04-15T00:00:00Z".to_string(),
            is_active: true,
        },
        first_seen_at: "2026-04-14T00:00:00Z".to_string(),
        inactivated_at: None,
        reactivated_at: None,
        lifecycle_stage: JobLifecycleStage::Active,
        primary_variant: Some(JobSourceVariant {
            source: source.to_string(),
            source_job_id: format!("{source}-{id}"),
            source_url: format!("https://example.com/{id}"),
            raw_payload: None,
            fetched_at: "2026-04-14T00:00:00Z".to_string(),
            last_seen_at: "2026-04-15T00:00:00Z".to_string(),
            is_active: true,
            inactivated_at: None,
        }),
    }
}

fn event(id: &str, job_id: &str, event_type: UserEventType) -> UserEventRecord {
    UserEventRecord {
        id: id.to_string(),
        profile_id: "profile-1".to_string(),
        event_type,
        job_id: Some(job_id.to_string()),
        company_name: Some("NovaLedger".to_string()),
        source: Some("djinni".to_string()),
        role_family: Some("engineering".to_string()),
        payload_json: None,
        created_at: "2026-04-15T00:00:00Z".to_string(),
    }
}

#[test]
fn label_assignment_is_deterministic_and_prioritized() {
    let events = vec![
        event("evt-1", "job-positive", UserEventType::JobSaved),
        event("evt-2", "job-positive", UserEventType::JobBadFit),
        event("evt-3", "job-positive", UserEventType::ApplicationCreated),
        event("evt-4", "job-negative", UserEventType::JobSaved),
        event("evt-5", "job-negative", UserEventType::JobBadFit),
        event("evt-6", "job-medium", UserEventType::JobSaved),
        event("evt-7", "job-viewed", UserEventType::JobOpened),
    ];
    let behavior = BehaviorService::new().build_aggregates(events.iter());
    let funnel = FunnelService::new().build_aggregates(events.iter());
    let dataset = OutcomeDatasetService::new()
        .build(
            &profile(),
            &events,
            vec![
                (
                    job_view("job-positive", "Senior Backend Engineer", "djinni"),
                    JobFeedbackState::default(),
                ),
                (
                    job_view("job-negative", "Senior Backend Engineer", "djinni"),
                    JobFeedbackState::default(),
                ),
                (
                    job_view("job-medium", "Senior Backend Engineer", "djinni"),
                    JobFeedbackState::default(),
                ),
                (
                    job_view("job-viewed", "Senior Backend Engineer", "djinni"),
                    JobFeedbackState::default(),
                ),
            ],
            &BTreeMap::new(),
            &BTreeMap::new(),
            &SearchRankingService::new(),
            &behavior,
            &funnel,
        )
        .expect("dataset should build");

    let labels = dataset
        .examples
        .iter()
        .map(|example| (example.job_id.as_str(), example.label, example.label_score))
        .collect::<Vec<_>>();

    assert_eq!(
        labels,
        vec![
            ("job-medium", OutcomeLabel::Medium, 1),
            ("job-negative", OutcomeLabel::Negative, 0),
            ("job-positive", OutcomeLabel::Positive, 2),
            ("job-viewed", OutcomeLabel::Medium, 1),
        ]
    );

    let positive = dataset
        .examples
        .iter()
        .find(|example| example.job_id == "job-positive")
        .expect("positive example should exist");
    assert_eq!(positive.label_reasons, vec!["applied"]);
    assert!(positive.signals.dismissed);
}

#[test]
fn reversal_events_clear_saved_and_dismissed_states_before_labeling() {
    let events = vec![
        event("evt-2", "job-viewed", UserEventType::JobSaved),
        event("evt-1", "job-viewed", UserEventType::JobOpened),
        event("evt-3", "job-viewed", UserEventType::JobUnsaved),
        event("evt-4", "job-cleared", UserEventType::JobHidden),
        event("evt-5", "job-cleared", UserEventType::JobUnhidden),
    ];
    let behavior = BehaviorService::new().build_aggregates(events.iter());
    let funnel = FunnelService::new().build_aggregates(events.iter());
    let dataset = OutcomeDatasetService::new()
        .build(
            &profile(),
            &events,
            vec![
                (
                    job_view("job-viewed", "Senior Backend Engineer", "djinni"),
                    JobFeedbackState::default(),
                ),
                (
                    job_view("job-cleared", "Senior Backend Engineer", "djinni"),
                    JobFeedbackState::default(),
                ),
            ],
            &BTreeMap::new(),
            &BTreeMap::new(),
            &SearchRankingService::new(),
            &behavior,
            &funnel,
        )
        .expect("dataset should build");

    assert_eq!(dataset.examples.len(), 1);
    assert_eq!(dataset.examples[0].job_id, "job-viewed");
    assert_eq!(dataset.examples[0].label_reasons, vec!["viewed"]);
    assert!(dataset.examples[0].signals.viewed);
    assert!(!dataset.examples[0].signals.saved);
    assert_eq!(dataset.examples[0].signals.saved_event_count, 1);
    assert_eq!(dataset.examples[0].signals.viewed_event_count, 1);
}

#[test]
fn explicit_feedback_flags_drive_labels_without_matching_events() {
    let events = vec![event("evt-1", "job-explicit", UserEventType::JobOpened)];
    let behavior = BehaviorService::new().build_aggregates(events.iter());
    let funnel = FunnelService::new().build_aggregates(events.iter());
    let dataset = OutcomeDatasetService::new()
        .build(
            &profile(),
            &events,
            vec![(
                job_view("job-explicit", "Senior Backend Engineer", "djinni"),
                JobFeedbackState {
                    saved: false,
                    hidden: true,
                    bad_fit: false,
                    company_status: None,
                    salary_signal: None,
                    interest_rating: None,
                    work_mode_signal: None,
                    legitimacy_signal: None,
                    tags: Vec::new(),
                },
            )],
            &BTreeMap::new(),
            &BTreeMap::new(),
            &SearchRankingService::new(),
            &behavior,
            &funnel,
        )
        .expect("dataset should build");

    assert_eq!(dataset.examples.len(), 1);
    assert_eq!(dataset.examples[0].label, OutcomeLabel::Negative);
    assert_eq!(
        dataset.examples[0].label_reasons,
        vec!["dismissed", "hidden"]
    );
    assert!(dataset.examples[0].signals.explicit_feedback);
    assert!(dataset.examples[0].signals.explicit_hidden);
    assert!(dataset.examples[0].signals.dismissed);
}

#[test]
fn outcome_job_ids_include_viewed_events_plus_feedback_job_ids() {
    let events = vec![
        event("evt-1", "job-saved", UserEventType::JobSaved),
        event("evt-2", "job-opened", UserEventType::JobOpened),
        event("evt-3", "job-applied", UserEventType::ApplicationCreated),
    ];

    let job_ids = outcome_job_ids(&events, &["job-feedback".to_string()]);

    assert_eq!(
        job_ids,
        vec!["job-applied", "job-feedback", "job-opened", "job-saved"]
    );
}

#[test]
fn empty_labelable_inputs_return_empty_dataset() {
    let events = Vec::new();
    let behavior = BehaviorService::new().build_aggregates(events.iter());
    let funnel = FunnelService::new().build_aggregates(events.iter());
    let dataset = OutcomeDatasetService::new()
        .build(
            &profile(),
            &events,
            Vec::new(),
            &BTreeMap::new(),
            &BTreeMap::new(),
            &SearchRankingService::new(),
            &behavior,
            &funnel,
        )
        .expect("empty dataset should build");

    assert!(dataset.examples.is_empty());
}

#[test]
fn application_outcomes_enrich_signals_and_label_timestamp() {
    let events = vec![
        UserEventRecord {
            id: "evt-0".to_string(),
            profile_id: "profile-1".to_string(),
            event_type: UserEventType::JobOpened,
            job_id: Some("job-positive".to_string()),
            company_name: Some("NovaLedger".to_string()),
            source: Some("djinni".to_string()),
            role_family: Some("engineering".to_string()),
            payload_json: None,
            created_at: "2026-04-10T00:00:00Z".to_string(),
        },
        event("evt-1", "job-positive", UserEventType::ApplicationCreated),
    ];
    let behavior = BehaviorService::new().build_aggregates(events.iter());
    let funnel = FunnelService::new().build_aggregates(events.iter());
    let dataset = OutcomeDatasetService::new()
        .build(
            &profile(),
            &events,
            vec![(
                job_view("job-positive", "Senior Backend Engineer", "djinni"),
                JobFeedbackState::default(),
            )],
            &BTreeMap::from([(
                "job-positive".to_string(),
                Application {
                    id: "app-1".to_string(),
                    job_id: "job-positive".to_string(),
                    resume_id: None,
                    status: "offer".to_string(),
                    applied_at: Some("2026-04-15T00:00:00Z".to_string()),
                    due_date: None,
                    outcome: Some(ApplicationOutcome::OfferReceived),
                    outcome_date: Some("2026-04-20T00:00:00Z".to_string()),
                    rejection_stage: None,
                    updated_at: "2026-04-20T00:00:00Z".to_string(),
                },
            )]),
            &BTreeMap::new(),
            &SearchRankingService::new(),
            &behavior,
            &funnel,
        )
        .expect("dataset should build");

    assert_eq!(dataset.examples.len(), 1);
    assert_eq!(
        dataset.examples[0].label_observed_at.as_deref(),
        Some("2026-04-20T00:00:00Z")
    );
    assert_eq!(
        dataset.examples[0].signals.outcome.as_deref(),
        Some("offer_received")
    );
    assert!(dataset.examples[0].signals.received_offer);
    assert!(dataset.examples[0].signals.reached_interview);
    assert_eq!(dataset.examples[0].signals.time_to_apply_days, Some(5));
}
