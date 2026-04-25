use serde::Serialize;

use crate::api::dto::jobs::JobResponse;
use crate::api::dto::resumes::ResumeVersionResponse;
use crate::domain::application::model::{
    Activity, Application, ApplicationContact, ApplicationDetail, ApplicationNote, Contact, Offer,
    Task,
};

#[derive(Debug, Serialize)]
pub struct NoteResponse {
    pub id: String,
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
    pub outcome: Option<String>,
    pub outcome_date: Option<String>,
    pub rejection_stage: Option<String>,
    pub updated_at: String,
}

#[derive(Debug, Serialize)]
pub struct ApplicationNoteResponse {
    pub id: String,
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
    pub contact: ContactResponse,
    pub relationship: String,
}

#[derive(Debug, Serialize)]
pub struct OfferResponse {
    pub id: String,
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
    pub activity_type: String,
    pub description: String,
    pub happened_at: String,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
pub struct TaskResponse {
    pub id: String,
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
    pub total: i64,
    pub next_cursor: Option<i64>,
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
            outcome: application.outcome.map(|o| o.as_str().to_string()),
            outcome_date: application.outcome_date,
            rejection_stage: application.rejection_stage,
            updated_at: application.updated_at,
        }
    }
}

impl From<ApplicationNote> for ApplicationNoteResponse {
    fn from(note: ApplicationNote) -> Self {
        Self {
            id: note.id,
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
            contact: ContactResponse::from(contact.contact),
            relationship: contact.relationship,
        }
    }
}

impl From<Offer> for OfferResponse {
    fn from(offer: Offer) -> Self {
        Self {
            id: offer.id,
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

impl From<ApplicationNote> for NoteResponse {
    fn from(note: ApplicationNote) -> Self {
        Self {
            id: note.id,
            content: note.content,
            created_at: note.created_at,
        }
    }
}

impl From<Activity> for ActivityResponse {
    fn from(activity: Activity) -> Self {
        Self {
            id: activity.id,
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
