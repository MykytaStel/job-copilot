use axum::extract::{Path, Query, State};
use serde::Deserialize;
use serde_json::json;

use crate::api::dto::applications::{
    ActivityResponse, ApplicationContactResponse, ApplicationDetailResponse, ApplicationResponse,
    ContactResponse, ContactsResponse, CreateActivityRequest, CreateApplicationContactRequest,
    CreateApplicationRequest, CreateContactRequest, CreateNoteRequest, NoteResponse, OfferResponse,
    RecentApplicationsResponse, UpdateApplicationRequest, UpsertOfferRequest,
};
use crate::api::error::{ApiError, ApiJson};
use crate::api::routes::events::{load_job_event_metadata, log_user_event_softly};
use crate::api::routes::feedback::ensure_profile_exists;
use crate::domain::user_event::model::{CreateUserEvent, UserEventType};
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct RecentApplicationsQuery {
    pub limit: Option<i64>,
}

pub async fn get_application_by_id(
    State(state): State<AppState>,
    Path(application_id): Path<String>,
) -> Result<axum::Json<ApplicationDetailResponse>, ApiError> {
    let Some(application) = state
        .applications_service
        .get_detail_by_id(&application_id)
        .await
        .map_err(|error| ApiError::from_repository(error, "applications_query_failed"))?
    else {
        return Err(ApiError::not_found(
            "application_not_found",
            format!("Application '{application_id}' was not found"),
        ));
    };

    Ok(axum::Json(ApplicationDetailResponse::from(application)))
}

pub async fn get_recent_applications(
    State(state): State<AppState>,
    Query(query): Query<RecentApplicationsQuery>,
) -> Result<axum::Json<RecentApplicationsResponse>, ApiError> {
    let limit = query.limit.unwrap_or(20);

    if !(1..=100).contains(&limit) {
        return Err(ApiError::invalid_limit(limit));
    }

    let applications = state
        .applications_service
        .list_recent(limit)
        .await
        .map_err(|error| ApiError::from_repository(error, "applications_query_failed"))?;

    Ok(axum::Json(RecentApplicationsResponse {
        applications: applications
            .into_iter()
            .map(ApplicationResponse::from)
            .collect(),
    }))
}

pub async fn create_application(
    State(state): State<AppState>,
    ApiJson(payload): ApiJson<CreateApplicationRequest>,
) -> Result<(axum::http::StatusCode, axum::Json<ApplicationResponse>), ApiError> {
    let payload = payload.validate()?;
    let profile_id = payload.profile_id.clone();

    let Some(_) = state
        .jobs_service
        .get_by_id(&payload.application.job_id)
        .await
        .map_err(|error| ApiError::from_repository(error, "applications_query_failed"))?
    else {
        return Err(ApiError::not_found(
            "job_not_found",
            format!("Job '{}' was not found", payload.application.job_id),
        ));
    };

    if let Some(profile_id) = profile_id.as_deref() {
        ensure_profile_exists(&state, profile_id).await?;
    }

    let event_metadata = if profile_id.is_some() {
        Some(load_job_event_metadata(&state, &payload.application.job_id).await?)
    } else {
        None
    };

    let mut application = state
        .applications_service
        .create(payload.application)
        .await
        .map_err(|error| ApiError::from_repository(error, "applications_query_failed"))?;

    if let Some(active_resume) = state
        .resumes_service
        .get_active()
        .await
        .map_err(|error| ApiError::from_repository(error, "applications_query_failed"))?
        && let Some(updated) = state
            .applications_service
            .attach_resume(&application.id, &active_resume.id)
            .await
            .map_err(|error| ApiError::from_repository(error, "applications_query_failed"))?
    {
        application = updated;
    }

    if let Some(profile_id) = profile_id {
        let metadata = event_metadata.unwrap_or_default();
        log_user_event_softly(
            &state,
            CreateUserEvent {
                profile_id,
                event_type: UserEventType::ApplicationCreated,
                job_id: Some(application.job_id.clone()),
                company_name: metadata.company_name,
                source: metadata.source,
                role_family: metadata.role_family,
                payload_json: Some(json!({
                    "application_id": application.id,
                    "status": application.status,
                })),
            },
        )
        .await;
    }

    Ok((
        axum::http::StatusCode::CREATED,
        axum::Json(ApplicationResponse::from(application)),
    ))
}

pub async fn patch_application(
    State(state): State<AppState>,
    Path(application_id): Path<String>,
    ApiJson(payload): ApiJson<UpdateApplicationRequest>,
) -> Result<axum::Json<ApplicationResponse>, ApiError> {
    let update = payload.validate()?;
    let new_status = update.status.clone();

    let Some(application) = state
        .applications_service
        .update(&application_id, update)
        .await
        .map_err(|error| ApiError::from_repository(error, "applications_query_failed"))?
    else {
        return Err(ApiError::not_found(
            "application_not_found",
            format!("Application '{application_id}' was not found"),
        ));
    };

    // Fire-and-forget: create a follow-up reminder task if status changed.
    if let Some(ref status) = new_status {
        state
            .followup_service
            .on_status_change(&application_id, status)
            .await;
    }

    Ok(axum::Json(ApplicationResponse::from(application)))
}

pub async fn create_activity(
    State(state): State<AppState>,
    Path(application_id): Path<String>,
    ApiJson(payload): ApiJson<CreateActivityRequest>,
) -> Result<(axum::http::StatusCode, axum::Json<ActivityResponse>), ApiError> {
    // Verify the application exists first.
    let Some(_) = state
        .applications_service
        .get_by_id(&application_id)
        .await
        .map_err(|error| ApiError::from_repository(error, "activities_query_failed"))?
    else {
        return Err(ApiError::not_found(
            "application_not_found",
            format!("Application '{application_id}' was not found"),
        ));
    };

    let is_interview = payload.activity_type == "interview";
    let activity = state
        .activities_service
        .create(payload.validate(&application_id)?)
        .await
        .map_err(|error| ApiError::from_repository(error, "activities_query_failed"))?;

    // Fire-and-forget: create a thank-you note reminder after an interview activity.
    if is_interview {
        state
            .followup_service
            .on_interview_activity(&application_id)
            .await;
    }

    Ok((
        axum::http::StatusCode::CREATED,
        axum::Json(ActivityResponse::from(activity)),
    ))
}

pub async fn create_note(
    State(state): State<AppState>,
    Path(application_id): Path<String>,
    ApiJson(payload): ApiJson<CreateNoteRequest>,
) -> Result<(axum::http::StatusCode, axum::Json<NoteResponse>), ApiError> {
    // Verify the application exists first.
    let Some(_) = state
        .applications_service
        .get_by_id(&application_id)
        .await
        .map_err(|error| ApiError::from_repository(error, "notes_query_failed"))?
    else {
        return Err(ApiError::not_found(
            "application_not_found",
            format!("Application '{application_id}' was not found"),
        ));
    };

    let note = state
        .applications_service
        .create_note(payload.validate(&application_id)?)
        .await
        .map_err(|error| ApiError::from_repository(error, "notes_query_failed"))?;

    Ok((
        axum::http::StatusCode::CREATED,
        axum::Json(NoteResponse::from(note)),
    ))
}

pub async fn create_contact(
    State(state): State<AppState>,
    ApiJson(payload): ApiJson<CreateContactRequest>,
) -> Result<(axum::http::StatusCode, axum::Json<ContactResponse>), ApiError> {
    let contact = state
        .applications_service
        .create_contact(payload.validate()?)
        .await
        .map_err(|error| ApiError::from_repository(error, "contacts_query_failed"))?;

    Ok((
        axum::http::StatusCode::CREATED,
        axum::Json(ContactResponse::from(contact)),
    ))
}

pub async fn list_contacts(
    State(state): State<AppState>,
) -> Result<axum::Json<ContactsResponse>, ApiError> {
    let contacts = state
        .applications_service
        .list_contacts()
        .await
        .map_err(|error| ApiError::from_repository(error, "contacts_query_failed"))?;

    Ok(axum::Json(ContactsResponse {
        contacts: contacts.into_iter().map(ContactResponse::from).collect(),
    }))
}

pub async fn add_application_contact(
    State(state): State<AppState>,
    Path(application_id): Path<String>,
    ApiJson(payload): ApiJson<CreateApplicationContactRequest>,
) -> Result<
    (
        axum::http::StatusCode,
        axum::Json<ApplicationContactResponse>,
    ),
    ApiError,
> {
    let Some(_) = state
        .applications_service
        .get_by_id(&application_id)
        .await
        .map_err(|error| ApiError::from_repository(error, "contacts_query_failed"))?
    else {
        return Err(ApiError::not_found(
            "application_not_found",
            format!("Application '{application_id}' was not found"),
        ));
    };

    let contact_id = payload.contact_id.clone();
    let payload = payload.validate(&application_id)?;

    let Some(_) = state
        .applications_service
        .get_contact_by_id(&contact_id)
        .await
        .map_err(|error| ApiError::from_repository(error, "contacts_query_failed"))?
    else {
        return Err(ApiError::not_found(
            "contact_not_found",
            format!("Contact '{contact_id}' was not found"),
        ));
    };

    let contact = state
        .applications_service
        .attach_contact(payload)
        .await
        .map_err(|error| ApiError::from_repository(error, "contacts_query_failed"))?;

    Ok((
        axum::http::StatusCode::CREATED,
        axum::Json(ApplicationContactResponse::from(contact)),
    ))
}

pub async fn upsert_offer(
    State(state): State<AppState>,
    Path(application_id): Path<String>,
    ApiJson(payload): ApiJson<UpsertOfferRequest>,
) -> Result<axum::Json<OfferResponse>, ApiError> {
    let Some(_) = state
        .applications_service
        .get_by_id(&application_id)
        .await
        .map_err(|error| ApiError::from_repository(error, "offers_query_failed"))?
    else {
        return Err(ApiError::not_found(
            "application_not_found",
            format!("Application '{application_id}' was not found"),
        ));
    };

    let offer = state
        .applications_service
        .upsert_offer(payload.validate(&application_id)?)
        .await
        .map_err(|error| ApiError::from_repository(error, "offers_query_failed"))?;

    Ok(axum::Json(OfferResponse::from(offer)))
}

#[cfg(test)]
mod tests {
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
        RecentApplicationsQuery, add_application_contact, create_application, create_contact,
        create_note, get_application_by_id, get_recent_applications, list_contacts,
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
            salary_min_usd: None,
            salary_max_usd: None,
            preferred_work_mode: None,
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
        let result =
            get_application_by_id(State(state), Path("missing-application".to_string())).await;

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
        let payload: Value =
            serde_json::from_slice(&body).expect("response body should be valid json");

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
            ProfilesService::for_tests(
                ProfilesServiceStub::default().with_profile(sample_profile()),
            ),
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

        let Json(response) = list_contacts(State(state))
            .await
            .expect("handler should list contacts");

        assert_eq!(response.contacts.len(), 1);
        assert_eq!(response.contacts[0].id, "contact-1");
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
        assert_eq!(note.application_id, "application-1");
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
        assert_eq!(link.application_id, "application-1");
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

        assert_eq!(offer.application_id, "application-1");
        assert_eq!(offer.status, "received");
    }
}
