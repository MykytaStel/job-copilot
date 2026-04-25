use axum::Json;
use axum::extract::{Path, Query, State};
use axum::response::IntoResponse;
use axum::{body, http::StatusCode};
use serde_json::{Value, json};

use crate::api::error::ApiJson;
use crate::domain::application::model::{Application, Contact};
use crate::domain::job::model::Job;
use crate::domain::profile::model::Profile;
use crate::domain::user_event::model::UserEventType;
use crate::services::applications::{ApplicationsService, ApplicationsServiceStub};
use crate::services::jobs::{JobsService, JobsServiceStub};
use crate::services::profiles::{ProfilesService, ProfilesServiceStub};
use crate::services::resumes::{ResumesService, ResumesServiceStub};
use crate::services::user_events::{UserEventsService, UserEventsServiceStub};
use crate::state::AppState;

use super::{
    ContactsQuery, RecentApplicationsQuery, add_application_contact, create_application,
    create_contact, create_note, get_application_by_id, get_recent_applications, list_contacts,
    patch_application, upsert_offer,
};

fn sample_job() -> Job {
    Job {
        id: "job-1".to_string(),
        title: "Backend Rust Engineer".to_string(),
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

fn sample_application() -> Application {
    Application {
        id: "application-1".to_string(),
        job_id: "job-1".to_string(),
        resume_id: None,
        status: "saved".to_string(),
        applied_at: None,
        due_date: None,
        outcome: None,
        outcome_date: None,
        rejection_stage: None,
        updated_at: "2026-04-11T00:00:00Z".to_string(),
    }
}

fn sample_profile() -> Profile {
    Profile {
        id: "profile-1".to_string(),
        name: "Jane Doe".to_string(),
        email: "jane@example.com".to_string(),
        location: Some("Kyiv".to_string()),
        raw_text: "Senior backend engineer with Rust".to_string(),
        analysis: None,
        years_of_experience: None,
        salary_min: None,
        salary_max: None,
        salary_currency: "USD".to_string(),
        languages: vec![],
        preferred_work_mode: None,
        search_preferences: None,
        created_at: "2026-04-11T00:00:00Z".to_string(),
        updated_at: "2026-04-11T00:00:00Z".to_string(),
        skills_updated_at: None,
    }
}

fn sample_contact() -> Contact {
    Contact {
        id: "contact-1".to_string(),
        name: "Jane Recruiter".to_string(),
        email: Some("jane@example.com".to_string()),
        phone: None,
        linkedin_url: None,
        company: Some("NovaLedger".to_string()),
        role: Some("Recruiter".to_string()),
        created_at: "2026-04-11T00:00:00Z".to_string(),
    }
}

#[tokio::test]
async fn returns_service_unavailable_when_database_is_missing() {
    let result = get_application_by_id(
        State(AppState::without_database()),
        Path("application-123".to_string()),
    )
    .await;

    let response = match result {
        Ok(_) => panic!("handler should fail without a configured database"),
        Err(error) => error.into_response(),
    };

    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
}

#[tokio::test]
async fn returns_not_found_for_unknown_application() {
    let state = AppState::for_services(
        ProfilesService::for_tests(ProfilesServiceStub::default()),
        JobsService::for_tests(JobsServiceStub::default().with_job(sample_job())),
        ApplicationsService::for_tests(ApplicationsServiceStub::default()),
        ResumesService::for_tests(ResumesServiceStub::default()),
    );
    let result = get_application_by_id(State(state), Path("missing-application".to_string())).await;

    let response = match result {
        Ok(_) => panic!("handler should return not found for unknown application"),
        Err(error) => error.into_response(),
    };

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn rejects_invalid_recent_applications_limit() {
    let result = get_recent_applications(
        State(AppState::without_database()),
        None,
        Query(RecentApplicationsQuery { limit: Some(0) }),
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
async fn rejects_invalid_patch_payload() {
    let state = AppState::for_services(
        ProfilesService::for_tests(ProfilesServiceStub::default()),
        JobsService::for_tests(JobsServiceStub::default()),
        ApplicationsService::for_tests(ApplicationsServiceStub::default()),
        ResumesService::for_tests(ResumesServiceStub::default()),
    );

    let response = patch_application(
        State(state),
        Path("application-1".to_string()),
        ApiJson(crate::api::dto::applications::UpdateApplicationRequest::default()),
    )
    .await
    .expect_err("handler should reject empty patch")
    .into_response();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn creates_application() {
    let state = AppState::for_services(
        ProfilesService::for_tests(ProfilesServiceStub::default()),
        JobsService::for_tests(JobsServiceStub::default().with_job(sample_job())),
        ApplicationsService::for_tests(ApplicationsServiceStub::default()),
        ResumesService::for_tests(ResumesServiceStub::default()),
    );

    let result = create_application(
        State(state),
        ApiJson(crate::api::dto::applications::CreateApplicationRequest {
            job_id: "job-1".to_string(),
            status: "saved".to_string(),
            applied_at: None,
            profile_id: None,
        }),
    )
    .await
    .expect("handler should create application");

    assert_eq!(result.0, StatusCode::CREATED);
}

#[tokio::test]
async fn creates_profile_scoped_application_event_when_profile_id_is_provided() {
    let state = AppState::for_services(
        ProfilesService::for_tests(ProfilesServiceStub::default().with_profile(sample_profile())),
        JobsService::for_tests(JobsServiceStub::default().with_job(sample_job())),
        ApplicationsService::for_tests(ApplicationsServiceStub::default()),
        ResumesService::for_tests(ResumesServiceStub::default()),
    )
    .with_user_events_service(UserEventsService::for_tests(
        UserEventsServiceStub::default(),
    ));

    let (status, Json(application)) = create_application(
        State(state.clone()),
        ApiJson(crate::api::dto::applications::CreateApplicationRequest {
            job_id: "job-1".to_string(),
            status: "saved".to_string(),
            applied_at: None,
            profile_id: Some("profile-1".to_string()),
        }),
    )
    .await
    .expect("handler should create application");

    assert_eq!(status, StatusCode::CREATED);

    let events = state
        .user_events_service
        .list_by_profile("profile-1")
        .await
        .expect("event query should succeed");
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].event_type, UserEventType::ApplicationCreated);
    assert_eq!(events[0].job_id.as_deref(), Some("job-1"));
    assert_eq!(
        events[0]
            .payload_json
            .as_ref()
            .and_then(|payload| payload.get("application_id"))
            .and_then(|value| value.as_str()),
        Some(application.id.as_str())
    );
}

#[tokio::test]
async fn creates_contact() {
    let state = AppState::for_services(
        ProfilesService::for_tests(ProfilesServiceStub::default()),
        JobsService::for_tests(JobsServiceStub::default()),
        ApplicationsService::for_tests(ApplicationsServiceStub::default()),
        ResumesService::for_tests(ResumesServiceStub::default()),
    );

    let (status, Json(contact)) = create_contact(
        State(state),
        ApiJson(crate::api::dto::applications::CreateContactRequest {
            name: "Jane Recruiter".to_string(),
            email: Some("jane@example.com".to_string()),
            phone: None,
            linkedin_url: None,
            company: Some("NovaLedger".to_string()),
            role: Some("Recruiter".to_string()),
        }),
    )
    .await
    .expect("handler should create contact");

    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(contact.name, "Jane Recruiter");
}

#[tokio::test]
async fn lists_contacts() {
    let state = AppState::for_services(
        ProfilesService::for_tests(ProfilesServiceStub::default()),
        JobsService::for_tests(JobsServiceStub::default()),
        ApplicationsService::for_tests(
            ApplicationsServiceStub::default().with_contact(sample_contact()),
        ),
        ResumesService::for_tests(ResumesServiceStub::default()),
    );

    let Json(response) = list_contacts(State(state), Query(ContactsQuery { offset: None }))
        .await
        .expect("handler should list contacts");

    assert_eq!(response.contacts.len(), 1);
    assert_eq!(response.contacts[0].id, "contact-1");
    assert_eq!(response.total, 1);
    assert!(response.next_cursor.is_none());
}

#[tokio::test]
async fn creates_note_for_existing_application() {
    let state = AppState::for_services(
        ProfilesService::for_tests(ProfilesServiceStub::default()),
        JobsService::for_tests(JobsServiceStub::default()),
        ApplicationsService::for_tests(
            ApplicationsServiceStub::default().with_application(sample_application()),
        ),
        ResumesService::for_tests(ResumesServiceStub::default()),
    );

    let (status, Json(note)) = create_note(
        State(state),
        Path("application-1".to_string()),
        ApiJson(crate::api::dto::applications::CreateNoteRequest {
            content: "Follow up on Friday".to_string(),
        }),
    )
    .await
    .expect("handler should create note");

    assert_eq!(status, StatusCode::CREATED);
}

#[tokio::test]
async fn links_contact_to_application() {
    let state = AppState::for_services(
        ProfilesService::for_tests(ProfilesServiceStub::default()),
        JobsService::for_tests(JobsServiceStub::default()),
        ApplicationsService::for_tests(
            ApplicationsServiceStub::default()
                .with_application(sample_application())
                .with_contact(sample_contact()),
        ),
        ResumesService::for_tests(ResumesServiceStub::default()),
    );

    let (status, Json(link)) = add_application_contact(
        State(state),
        Path("application-1".to_string()),
        ApiJson(
            crate::api::dto::applications::CreateApplicationContactRequest {
                contact_id: "contact-1".to_string(),
                relationship: "recruiter".to_string(),
            },
        ),
    )
    .await
    .expect("handler should link contact");

    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(link.contact.id, "contact-1");
}

#[tokio::test]
async fn rejects_unknown_contact_when_linking_application_contact() {
    let state = AppState::for_services(
        ProfilesService::for_tests(ProfilesServiceStub::default()),
        JobsService::for_tests(JobsServiceStub::default()),
        ApplicationsService::for_tests(
            ApplicationsServiceStub::default().with_application(sample_application()),
        ),
        ResumesService::for_tests(ResumesServiceStub::default()),
    );

    let response = add_application_contact(
        State(state),
        Path("application-1".to_string()),
        ApiJson(
            crate::api::dto::applications::CreateApplicationContactRequest {
                contact_id: "missing-contact".to_string(),
                relationship: "recruiter".to_string(),
            },
        ),
    )
    .await
    .expect_err("handler should reject unknown contact")
    .into_response();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn upserts_offer_for_existing_application() {
    let state = AppState::for_services(
        ProfilesService::for_tests(ProfilesServiceStub::default()),
        JobsService::for_tests(JobsServiceStub::default()),
        ApplicationsService::for_tests(
            ApplicationsServiceStub::default().with_application(sample_application()),
        ),
        ResumesService::for_tests(ResumesServiceStub::default()),
    );

    let Json(offer) = upsert_offer(
        State(state),
        Path("application-1".to_string()),
        ApiJson(crate::api::dto::applications::UpsertOfferRequest {
            status: "received".to_string(),
            compensation_min: Some(5000),
            compensation_max: Some(6500),
            compensation_currency: Some("USD".to_string()),
            starts_at: Some("2026-05-01".to_string()),
            notes: Some("Includes bonus".to_string()),
        }),
    )
    .await
    .expect("handler should upsert offer");

    assert_eq!(offer.status, "received");
}
