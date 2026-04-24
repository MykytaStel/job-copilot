use axum::response::IntoResponse;

use super::{
    CreateApplicationContactRequest, CreateApplicationRequest, CreateContactRequest,
    UpdateApplicationRequest, UpsertOfferRequest,
};

#[test]
fn rejects_empty_patch_payload() {
    let response = UpdateApplicationRequest::default()
        .validate()
        .expect_err("empty patch should be rejected")
        .into_response();

    assert_eq!(response.status(), axum::http::StatusCode::BAD_REQUEST);
}

#[test]
fn rejects_invalid_contact_relationship() {
    let response = CreateApplicationContactRequest {
        contact_id: "contact-1".to_string(),
        relationship: "friend".to_string(),
    }
    .validate("application-1")
    .expect_err("unsupported relationship should be rejected")
    .into_response();

    assert_eq!(response.status(), axum::http::StatusCode::BAD_REQUEST);
}

#[test]
fn carries_profile_scope_into_validated_application() {
    let validated = CreateApplicationRequest {
        job_id: "job-1".to_string(),
        status: "saved".to_string(),
        applied_at: None,
        profile_id: Some(" profile-1 ".to_string()),
    }
    .validate()
    .expect("application payload should validate");

    assert_eq!(
        validated.application.profile_id.as_deref(),
        Some("profile-1")
    );
}

#[test]
fn allows_clearing_due_date_with_null() {
    let payload: UpdateApplicationRequest =
        serde_json::from_str(r#"{"due_date":null}"#).expect("json payload should deserialize");

    let validated = payload
        .validate()
        .expect("null due_date patch should be valid");

    assert_eq!(validated.status, None);
    assert_eq!(validated.due_date, Some(None));
}

#[test]
fn allows_setting_due_date() {
    let payload: UpdateApplicationRequest =
        serde_json::from_str(r#"{"due_date":"2026-05-10T12:00:00Z"}"#)
            .expect("json payload should deserialize");

    let validated = payload.validate().expect("due_date patch should be valid");

    assert_eq!(
        validated.due_date,
        Some(Some("2026-05-10T12:00:00Z".to_string()))
    );
}

#[test]
fn trims_blank_optional_contact_fields() {
    let payload = CreateContactRequest {
        name: " Recruiter ".to_string(),
        email: Some(" recruiter@example.com ".to_string()),
        phone: Some("   ".to_string()),
        linkedin_url: Some(" https://linkedin.com/in/recruiter ".to_string()),
        company: None,
        role: Some(" Talent Partner ".to_string()),
    }
    .validate()
    .expect("contact payload should validate");

    assert_eq!(payload.name, "Recruiter");
    assert_eq!(payload.email.as_deref(), Some("recruiter@example.com"));
    assert_eq!(payload.phone, None);
    assert_eq!(
        payload.linkedin_url.as_deref(),
        Some("https://linkedin.com/in/recruiter")
    );
    assert_eq!(payload.role.as_deref(), Some("Talent Partner"));
}

#[test]
fn rejects_offer_range_when_min_exceeds_max() {
    let response = UpsertOfferRequest {
        status: "received".to_string(),
        compensation_min: Some(5000),
        compensation_max: Some(4000),
        compensation_currency: Some("USD".to_string()),
        starts_at: None,
        notes: None,
    }
    .validate("application-1")
    .expect_err("invalid offer range should be rejected")
    .into_response();

    assert_eq!(response.status(), axum::http::StatusCode::BAD_REQUEST);
}
