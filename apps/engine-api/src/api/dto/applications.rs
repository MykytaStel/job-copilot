use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::api::dto::jobs::JobResponse;
use crate::api::dto::resumes::ResumeVersionResponse;
use crate::api::error::ApiError;
use crate::domain::application::model::{
    Activity, Application, ApplicationContact, ApplicationDetail, ApplicationNote, Contact,
    CreateActivity, CreateApplication, CreateNote, Task, UpdateApplication,
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
    pub notes: Vec<ApplicationNoteResponse>,
    pub contacts: Vec<ApplicationContactResponse>,
    pub activities: Vec<ActivityResponse>,
    pub tasks: Vec<TaskResponse>,
}

#[derive(Debug, Serialize)]
pub struct RecentApplicationsResponse {
    pub applications: Vec<ApplicationResponse>,
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

impl CreateNoteRequest {
    pub fn validate(self, application_id: &str) -> Result<CreateNote, ApiError> {
        let content = validate_required("content", self.content)?;
        Ok(CreateNote {
            application_id: application_id.to_string(),
            content,
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
