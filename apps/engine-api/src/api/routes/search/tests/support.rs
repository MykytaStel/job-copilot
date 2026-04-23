use crate::api::dto::search::{
    RunSearchRequest, SearchProfileRequest, SearchRerankerComparisonRequest,
};
use crate::domain::job::model::{Job, JobLifecycleStage, JobSourceVariant, JobView};
use crate::domain::profile::model::Profile;
use crate::domain::search::profile::{TargetRegion, WorkMode};
use crate::domain::user_event::model::{UserEventRecord, UserEventType};
use crate::services::applications::{ApplicationsService, ApplicationsServiceStub};
use crate::services::jobs::{JobsService, JobsServiceStub};
use crate::services::profiles::{ProfilesService, ProfilesServiceStub};
use crate::services::ranking::runtime::RerankerRuntimeMode;
use crate::services::resumes::{ResumesService, ResumesServiceStub};
use crate::services::trained_reranker::TrainedRerankerModel;
use crate::services::user_events::{UserEventsService, UserEventsServiceStub};
use crate::state::AppState;

pub(super) fn sample_job(id: &str, title: &str) -> Job {
    Job {
        id: id.to_string(),
        title: title.to_string(),
        company_name: "NovaLedger".to_string(),
        location: None,
        remote_type: Some("remote".to_string()),
        seniority: Some("senior".to_string()),
        description_text: "Rust and Postgres".to_string(),
        salary_min: None,
        salary_max: None,
        salary_currency: None,
        posted_at: None,
        last_seen_at: "2026-04-11T00:00:00Z".to_string(),
        is_active: true,
    }
}

pub(super) fn sample_application_search_hit(
    id: &str,
    job_id: &str,
    job_title: &str,
    company_name: &str,
) -> crate::domain::search::global::ApplicationSearchHit {
    crate::domain::search::global::ApplicationSearchHit {
        id: id.to_string(),
        job_id: job_id.to_string(),
        resume_id: None,
        status: "saved".to_string(),
        applied_at: None,
        due_date: None,
        updated_at: "2026-04-14T00:00:00Z".to_string(),
        job_title: job_title.to_string(),
        company_name: company_name.to_string(),
    }
}

pub(super) fn sample_profile() -> Profile {
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
        search_preferences: None,
        created_at: "2026-04-14T00:00:00Z".to_string(),
        updated_at: "2026-04-14T00:00:00Z".to_string(),
        skills_updated_at: None,
    }
}

pub(super) fn sample_job_view(
    id: &str,
    title: &str,
    description_text: &str,
    remote_type: Option<&str>,
    source: &str,
) -> JobView {
    let source_url = match source {
        "djinni" => format!("https://djinni.co/jobs/{id}-sample-role/"),
        "work_ua" => format!("https://www.work.ua/jobs/{id}/"),
        "robota_ua" => format!("https://robota.ua/vacancy/{id}"),
        _ => format!("https://example.com/{id}"),
    };

    JobView {
        job: Job {
            id: id.to_string(),
            title: title.to_string(),
            company_name: "NovaLedger".to_string(),
            location: None,
            remote_type: remote_type.map(str::to_string),
            seniority: Some("senior".to_string()),
            description_text: description_text.to_string(),
            salary_min: None,
            salary_max: None,
            salary_currency: None,
            posted_at: Some("2026-04-12T09:00:00Z".to_string()),
            last_seen_at: "2026-04-14T09:00:00Z".to_string(),
            is_active: true,
        },
        first_seen_at: "2026-04-12T09:00:00Z".to_string(),
        inactivated_at: None,
        reactivated_at: None,
        lifecycle_stage: JobLifecycleStage::Active,
        primary_variant: Some(JobSourceVariant {
            source: source.to_string(),
            source_job_id: format!("{id}-source"),
            source_url,
            raw_payload: None,
            fetched_at: "2026-04-14T09:00:00Z".to_string(),
            last_seen_at: "2026-04-14T09:00:00Z".to_string(),
            is_active: true,
            inactivated_at: None,
        }),
    }
}

pub(super) fn user_event(
    id: &str,
    event_type: UserEventType,
    source: Option<&str>,
    role_family: Option<&str>,
) -> UserEventRecord {
    UserEventRecord {
        id: id.to_string(),
        profile_id: "profile-1".to_string(),
        event_type,
        job_id: Some(format!("job-{id}")),
        company_name: Some("NovaLedger".to_string()),
        source: source.map(str::to_string),
        role_family: role_family.map(str::to_string),
        payload_json: None,
        created_at: "2026-04-15T00:00:00Z".to_string(),
    }
}

pub(super) fn trained_reranker_model() -> TrainedRerankerModel {
    TrainedRerankerModel::from_json_str(
        r#"{
              "artifact_version": "trained_reranker_v3",
              "model_type": "logistic_regression",
              "label_policy_version": "outcome_label_v3",
              "feature_names": ["matched_skill_count"],
              "feature_transforms": {},
              "weights": { "matched_skill_count": 20.0 },
              "intercept": -4.0,
              "max_score_delta": 8,
              "training": {
                "example_count": 2,
                "positive_count": 1,
                "medium_count": 0,
                "negative_count": 1,
                "epochs": 10,
                "learning_rate": 0.1,
                "l2": 0.0,
                "loss": 0.5
              }
            }"#,
    )
    .expect("test artifact should load")
}

pub(super) fn outcome_signal_trained_reranker_model() -> TrainedRerankerModel {
    TrainedRerankerModel::from_json_str(
        r#"{
              "artifact_version": "trained_reranker_v3",
              "model_type": "logistic_regression",
              "label_policy_version": "outcome_label_v3",
              "feature_names": ["outcome_received_offer"],
              "weights": { "outcome_received_offer": 10.0 },
              "intercept": -4.0,
              "max_score_delta": 8
            }"#,
    )
    .expect("signal-aware test artifact should load")
}

pub(super) fn reranker_comparison_request(
    comparison: Option<SearchRerankerComparisonRequest>,
) -> RunSearchRequest {
    RunSearchRequest {
        profile_id: Some("profile-1".to_string()),
        search_profile: SearchProfileRequest {
            primary_role: "backend_engineer".to_string(),
            primary_role_confidence: Some(95),
            target_roles: vec![],
            role_candidates: vec![],
            seniority: "senior".to_string(),
            target_regions: vec![TargetRegion::EuRemote],
            work_modes: vec![WorkMode::Remote],
            allowed_sources: vec!["djinni".to_string(), "work_ua".to_string()],
            profile_skills: vec!["rust".to_string()],
            profile_keywords: vec!["backend".to_string()],
            search_terms: vec!["rust".to_string()],
            exclude_terms: vec![],
        },
        limit: Some(10),
        reranker_comparison: comparison,
    }
}

pub(super) fn reranker_comparison_state() -> AppState {
    let shared_desc = "Remote EU role working with Rust and Postgres backend systems";
    let events = UserEventsServiceStub::default()
        .with_event(user_event(
            "evt-1",
            UserEventType::JobSaved,
            Some("djinni"),
            Some("engineering"),
        ))
        .with_event(user_event(
            "evt-2",
            UserEventType::JobSaved,
            Some("djinni"),
            Some("engineering"),
        ))
        .with_event(user_event(
            "evt-3",
            UserEventType::ApplicationCreated,
            Some("djinni"),
            Some("engineering"),
        ));

    AppState::for_services(
        ProfilesService::for_tests(ProfilesServiceStub::default().with_profile(sample_profile())),
        JobsService::for_tests(
            JobsServiceStub::default()
                .with_job_view(sample_job_view(
                    "job-1",
                    "Senior Backend Developer",
                    shared_desc,
                    Some("remote"),
                    "work_ua",
                ))
                .with_job_view(sample_job_view(
                    "job-2",
                    "Senior Backend Developer",
                    shared_desc,
                    Some("remote"),
                    "djinni",
                )),
        ),
        ApplicationsService::for_tests(ApplicationsServiceStub::default()),
        ResumesService::for_tests(ResumesServiceStub::default()),
    )
    .with_user_events_service(UserEventsService::for_tests(events))
    .with_reranker_runtime_mode(RerankerRuntimeMode::Deterministic)
}
