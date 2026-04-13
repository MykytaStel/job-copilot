use crate::domain::job::model::Job;
use crate::domain::resume::model::ResumeVersion;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Application {
    pub id: String,
    pub job_id: String,
    pub resume_id: Option<String>,
    pub status: String,
    pub applied_at: Option<String>,
    pub due_date: Option<String>,
    pub updated_at: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ApplicationDetail {
    pub application: Application,
    pub job: Job,
    pub resume: Option<ResumeVersion>,
    pub notes: Vec<ApplicationNote>,
    pub contacts: Vec<ApplicationContact>,
    pub activities: Vec<Activity>,
    pub tasks: Vec<Task>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CreateApplication {
    pub job_id: String,
    pub status: String,
    pub applied_at: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct UpdateApplication {
    pub status: Option<String>,
    pub due_date: Option<Option<String>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CreateActivity {
    pub application_id: String,
    pub activity_type: String,
    pub description: String,
    pub happened_at: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CreateTask {
    pub application_id: String,
    pub title: String,
    /// Days from now when the reminder should fire (stored as `NOW() + N days` in DB).
    pub remind_in_days: i32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ApplicationNote {
    pub id: String,
    pub application_id: String,
    pub content: String,
    pub created_at: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CreateNote {
    pub application_id: String,
    pub content: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Contact {
    pub id: String,
    pub name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub linkedin_url: Option<String>,
    pub company: Option<String>,
    pub role: Option<String>,
    pub created_at: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ApplicationContact {
    pub id: String,
    pub application_id: String,
    pub contact: Contact,
    pub relationship: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Activity {
    pub id: String,
    pub application_id: String,
    pub activity_type: String,
    pub description: String,
    pub happened_at: String,
    pub created_at: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Task {
    pub id: String,
    pub application_id: String,
    pub title: String,
    pub remind_at: Option<String>,
    pub done: bool,
    pub created_at: String,
}
