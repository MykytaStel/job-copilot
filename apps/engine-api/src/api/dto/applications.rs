use serde::{Deserialize, Deserializer, Serialize};
use serde_json::json;

use crate::api::dto::jobs::JobResponse;
use crate::api::dto::resumes::ResumeVersionResponse;
use crate::api::error::ApiError;
use crate::domain::application::model::{
    Activity, Application, ApplicationContact, ApplicationDetail, ApplicationNote, Contact,
    CreateActivity, CreateApplication, CreateApplicationContact, CreateContact, CreateNote, Offer,
    Task, UpdateApplication, UpsertOffer,
};

#[derive(Default, Deserialize)]
pub struct CreateApplicationRequest {
    pub job_id: String,
    pub status: String,
    pub applied_at: Option<String>,
}

#[derive(Default, Deserialize)]
pub struct UpdateApplicationRequest {
    pub status: Option<String>,
    #[serde(default, deserialize_with = "deserialize_patch_due_date")]
    pub due_date: Option<Option<String>>,
}

#[derive(Deserialize)]
pub struct CreateActivityRequest {
    pub activity_type: String,
    pub description: String,
    pub happened_at: String,
}

#[derive(Deserialize)]
pub struct CreateNoteRequest {
    pub content: String,
}

#[derive(Deserialize)]
pub struct CreateContactRequest {
    pub name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub linkedin_url: Option<String>,
    pub company: Option<String>,
    pub role: Option<String>,
}

#[derive(Deserialize)]
pub struct CreateApplicationContactRequest {
    pub contact_id: String,
    pub relationship: String,
}

#[derive(Deserialize)]
pub struct UpsertOfferRequest {
    pub status: String,
    pub compensation_min: Option<i32>,
    pub compensation_max: Option<i32>,
    pub compensation_currency: Option<String>,
    pub starts_at: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct NoteResponse {
    pub id: String,
    pub application_id: String,
    pub content: String,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
pub struct ApplicationResponse {
    pub id: String,
    pub job_id: String,
    pub resume_id: Option<String>,
    pub status: String,
    pub applied_at: Option<String>,
    pub due_date: Option<String>,
    pub updated_at: String,
}

#[derive(Debug, Serialize)]
pub struct ApplicationNoteResponse {
    pub id: String,
    pub application_id: String,
    pub content: String,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
pub struct ContactResponse {
    pub id: String,
    pub name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub linkedin_url: Option<String>,
    pub company: Option<String>,
    pub role: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
pub struct ApplicationContactResponse {
    pub id: String,
    pub application_id: String,
    pub contact: ContactResponse,
    pub relationship: String,
}

#[derive(Debug, Serialize)]
pub struct OfferResponse {
    pub id: String,
    pub application_id: String,
    pub status: String,
    pub compensation_min: Option<i32>,
    pub compensation_max: Option<i32>,
    pub compensation_currency: Option<String>,
    pub starts_at: Option<String>,
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize)]
pub struct ActivityResponse {
    pub id: String,
    pub application_id: String,
    pub activity_type: String,
    pub description: String,
    pub happened_at: String,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
pub struct TaskResponse {
    pub id: String,
    pub application_id: String,
    pub title: String,
    pub remind_at: Option<String>,
    pub done: bool,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
pub struct ApplicationDetailResponse {
    #[serde(flatten)]
    pub application: ApplicationResponse,
    pub job: JobResponse,
    pub resume: Option<ResumeVersionResponse>,
    pub offer: Option<OfferResponse>,
    pub notes: Vec<ApplicationNoteResponse>,
    pub contacts: Vec<ApplicationContactResponse>,
    pub activities: Vec<ActivityResponse>,
    pub tasks: Vec<TaskResponse>,
}

#[derive(Debug, Serialize)]
pub struct RecentApplicationsResponse {
    pub applications: Vec<ApplicationResponse>,
}

#[derive(Debug, Serialize)]
pub struct ContactsResponse {
    pub contacts: Vec<ContactResponse>,
}

impl CreateApplicationRequest {
    pub fn validate(self) -> Result<CreateApplication, ApiError> {
        Ok(CreateApplication {
            job_id: validate_required("job_id", self.job_id)?,
            status: validate_status(self.status)?,
            applied_at: self.applied_at,
        })
    }
}

impl CreateContactRequest {
    pub fn validate(self) -> Result<CreateContact, ApiError> {
        Ok(CreateContact {
            name: validate_required("name", self.name)?,
            email: validate_optional_email(self.email)?,
            phone: validate_optional_trimmed(self.phone),
            linkedin_url: validate_optional_trimmed(self.linkedin_url),
            company: validate_optional_trimmed(self.company),
            role: validate_optional_trimmed(self.role),
        })
    }
}

impl CreateApplicationContactRequest {
    pub fn validate(self, application_id: &str) -> Result<CreateApplicationContact, ApiError> {
        Ok(CreateApplicationContact {
            application_id: application_id.to_string(),
            contact_id: validate_required("contact_id", self.contact_id)?,
            relationship: validate_relationship(self.relationship)?,
        })
    }
}

impl UpdateApplicationRequest {
    pub fn validate(self) -> Result<UpdateApplication, ApiError> {
        if self.status.is_none() && self.due_date.is_none() {
            return Err(ApiError::bad_request_with_details(
                "empty_application_patch",
                "PATCH /applications/:id requires at least one field",
                json!({
                    "allowed_fields": ["status", "due_date"]
                }),
            ));
        }

        Ok(UpdateApplication {
            status: self.status.map(validate_status).transpose()?,
            due_date: self.due_date,
        })
    }
}

impl From<Application> for ApplicationResponse {
    fn from(application: Application) -> Self {
        Self {
            id: application.id,
            job_id: application.job_id,
            resume_id: application.resume_id,
            status: application.status,
            applied_at: application.applied_at,
            due_date: application.due_date,
            updated_at: application.updated_at,
        }
    }
}

impl From<ApplicationNote> for ApplicationNoteResponse {
    fn from(note: ApplicationNote) -> Self {
        Self {
            id: note.id,
            application_id: note.application_id,
            content: note.content,
            created_at: note.created_at,
        }
    }
}

impl From<Contact> for ContactResponse {
    fn from(contact: Contact) -> Self {
        Self {
            id: contact.id,
            name: contact.name,
            email: contact.email,
            phone: contact.phone,
            linkedin_url: contact.linkedin_url,
            company: contact.company,
            role: contact.role,
            created_at: contact.created_at,
        }
    }
}

impl From<ApplicationContact> for ApplicationContactResponse {
    fn from(contact: ApplicationContact) -> Self {
        Self {
            id: contact.id,
            application_id: contact.application_id,
            contact: ContactResponse::from(contact.contact),
            relationship: contact.relationship,
        }
    }
}

impl From<Offer> for OfferResponse {
    fn from(offer: Offer) -> Self {
        Self {
            id: offer.id,
            application_id: offer.application_id,
            status: offer.status,
            compensation_min: offer.compensation_min,
            compensation_max: offer.compensation_max,
            compensation_currency: offer.compensation_currency,
            starts_at: offer.starts_at,
            notes: offer.notes,
            created_at: offer.created_at,
            updated_at: offer.updated_at,
        }
    }
}

impl CreateNoteRequest {
    pub fn validate(self, application_id: &str) -> Result<CreateNote, ApiError> {
        let content = validate_required("content", self.content)?;
        Ok(CreateNote {
            application_id: application_id.to_string(),
            content,
        })
    }
}

impl UpsertOfferRequest {
    pub fn validate(self, application_id: &str) -> Result<UpsertOffer, ApiError> {
        if let (Some(min), Some(max)) = (self.compensation_min, self.compensation_max) {
            if min > max {
                return Err(ApiError::bad_request_with_details(
                    "invalid_offer_input",
                    "Field 'compensation_min' must be less than or equal to 'compensation_max'",
                    json!({
                        "field": "compensation_min",
                        "compensation_min": min,
                        "compensation_max": max,
                    }),
                ));
            }
        }

        Ok(UpsertOffer {
            application_id: application_id.to_string(),
            status: validate_offer_status(self.status)?,
            compensation_min: self.compensation_min,
            compensation_max: self.compensation_max,
            compensation_currency: validate_optional_trimmed(self.compensation_currency),
            starts_at: validate_optional_trimmed(self.starts_at),
            notes: validate_optional_trimmed(self.notes),
        })
    }
}

impl From<ApplicationNote> for NoteResponse {
    fn from(note: ApplicationNote) -> Self {
        Self {
            id: note.id,
            application_id: note.application_id,
            content: note.content,
            created_at: note.created_at,
        }
    }
}

impl From<Activity> for ActivityResponse {
    fn from(activity: Activity) -> Self {
        Self {
            id: activity.id,
            application_id: activity.application_id,
            activity_type: activity.activity_type,
            description: activity.description,
            happened_at: activity.happened_at,
            created_at: activity.created_at,
        }
    }
}

impl From<Task> for TaskResponse {
    fn from(task: Task) -> Self {
        Self {
            id: task.id,
            application_id: task.application_id,
            title: task.title,
            remind_at: task.remind_at,
            done: task.done,
            created_at: task.created_at,
        }
    }
}

impl From<ApplicationDetail> for ApplicationDetailResponse {
    fn from(detail: ApplicationDetail) -> Self {
        Self {
            application: ApplicationResponse::from(detail.application),
            job: JobResponse::from(detail.job),
            resume: detail.resume.map(ResumeVersionResponse::from),
            offer: detail.offer.map(OfferResponse::from),
            notes: detail
                .notes
                .into_iter()
                .map(ApplicationNoteResponse::from)
                .collect(),
            contacts: detail
                .contacts
                .into_iter()
                .map(ApplicationContactResponse::from)
                .collect(),
            activities: detail
                .activities
                .into_iter()
                .map(ActivityResponse::from)
                .collect(),
            tasks: detail.tasks.into_iter().map(TaskResponse::from).collect(),
        }
    }
}

impl CreateActivityRequest {
    pub fn validate(self, application_id: &str) -> Result<CreateActivity, ApiError> {
        let activity_type = validate_required("activity_type", self.activity_type)?;
        let description = validate_required("description", self.description)?;
        let happened_at = validate_required("happened_at", self.happened_at)?;

        if !matches!(
            activity_type.as_str(),
            "interview" | "call" | "email" | "offer" | "rejection" | "other"
        ) {
            return Err(ApiError::bad_request_with_details(
                "invalid_activity_type",
                "Unsupported activity type",
                json!({
                    "field": "activity_type",
                    "allowed_values": ["interview", "call", "email", "offer", "rejection", "other"],
                    "received": activity_type,
                }),
            ));
        }

        Ok(CreateActivity {
            application_id: application_id.to_string(),
            activity_type,
            description,
            happened_at,
        })
    }
}

fn validate_required(field: &'static str, value: String) -> Result<String, ApiError> {
    let value = value.trim().to_string();
    if value.is_empty() {
        return Err(ApiError::bad_request_with_details(
            "invalid_application_input",
            format!("Field '{field}' must not be empty"),
            json!({ "field": field }),
        ));
    }

    Ok(value)
}

fn validate_optional_trimmed(value: Option<String>) -> Option<String> {
    value.and_then(|value| {
        let value = value.trim().to_string();
        (!value.is_empty()).then_some(value)
    })
}

fn deserialize_patch_due_date<'de, D>(
    deserializer: D,
) -> Result<Option<Option<String>>, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(Some(Option::<String>::deserialize(deserializer)?))
}

fn validate_optional_email(value: Option<String>) -> Result<Option<String>, ApiError> {
    validate_optional_trimmed(value)
        .map(validate_email)
        .transpose()
}

fn validate_email(value: String) -> Result<String, ApiError> {
    let value = validate_required("email", value)?;

    if !value.contains('@') {
        return Err(ApiError::bad_request_with_details(
            "invalid_application_input",
            "Field 'email' must contain '@'",
            json!({ "field": "email" }),
        ));
    }

    Ok(value)
}

fn validate_relationship(value: String) -> Result<String, ApiError> {
    let value = validate_required("relationship", value)?;

    if !matches!(
        value.as_str(),
        "recruiter" | "hiring_manager" | "interviewer" | "referrer" | "other"
    ) {
        return Err(ApiError::bad_request_with_details(
            "invalid_contact_relationship",
            "Unsupported contact relationship",
            json!({
                "field": "relationship",
                "allowed_values": ["recruiter", "hiring_manager", "interviewer", "referrer", "other"],
                "received": value,
            }),
        ));
    }

    Ok(value)
}

fn validate_offer_status(value: String) -> Result<String, ApiError> {
    let value = validate_required("status", value)?;

    if !matches!(
        value.as_str(),
        "draft" | "received" | "accepted" | "declined" | "expired"
    ) {
        return Err(ApiError::bad_request_with_details(
            "invalid_offer_status",
            "Unsupported offer status",
            json!({
                "field": "status",
                "allowed_values": ["draft", "received", "accepted", "declined", "expired"],
                "received": value,
            }),
        ));
    }

    Ok(value)
}

fn validate_status(value: String) -> Result<String, ApiError> {
    let value = validate_required("status", value)?;

    if !matches!(
        value.as_str(),
        "saved" | "applied" | "interview" | "offer" | "rejected"
    ) {
        return Err(ApiError::bad_request_with_details(
            "invalid_application_status",
            "Unsupported application status",
            json!({
                "field": "status",
                "allowed_values": ["saved", "applied", "interview", "offer", "rejected"],
                "received": value,
            }),
        ));
    }

    Ok(value)
}

#[cfg(test)]
mod tests {
    use axum::response::IntoResponse;

    use super::{
        CreateApplicationContactRequest, CreateContactRequest, UpdateApplicationRequest,
        UpsertOfferRequest,
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
    fn allows_clearing_due_date_with_null() {
        let payload: UpdateApplicationRequest =
            serde_json::from_str(r#"{"due_date":null}"#).expect("json payload should deserialize");

        let validated = payload.validate().expect("null due_date patch should be valid");

        assert_eq!(validated.status, None);
        assert_eq!(validated.due_date, Some(None));
    }

    #[test]
    fn allows_setting_due_date() {
        let payload: UpdateApplicationRequest =
            serde_json::from_str(r#"{"due_date":"2026-05-10T12:00:00Z"}"#)
                .expect("json payload should deserialize");

        let validated = payload.validate().expect("due_date patch should be valid");

        assert_eq!(validated.due_date, Some(Some("2026-05-10T12:00:00Z".to_string())));
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
}
