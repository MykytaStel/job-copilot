use crate::domain::job::model::Job;
use crate::domain::resume::model::ResumeVersion;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ApplicationOutcome {
    PhoneScreen,
    TechnicalInterview,
    FinalInterview,
    OfferReceived,
    Rejected,
    Ghosted,
    Withdrew,
}

impl ApplicationOutcome {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::PhoneScreen => "phone_screen",
            Self::TechnicalInterview => "technical_interview",
            Self::FinalInterview => "final_interview",
            Self::OfferReceived => "offer_received",
            Self::Rejected => "rejected",
            Self::Ghosted => "ghosted",
            Self::Withdrew => "withdrew",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value.trim() {
            "phone_screen" => Some(Self::PhoneScreen),
            "technical_interview" => Some(Self::TechnicalInterview),
            "final_interview" => Some(Self::FinalInterview),
            "offer_received" => Some(Self::OfferReceived),
            "rejected" => Some(Self::Rejected),
            "ghosted" => Some(Self::Ghosted),
            "withdrew" => Some(Self::Withdrew),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Application {
    pub id: String,
    pub job_id: String,
    pub resume_id: Option<String>,
    pub status: String,
    pub applied_at: Option<String>,
    pub due_date: Option<String>,
    pub outcome: Option<ApplicationOutcome>,
    pub outcome_date: Option<String>,
    pub rejection_stage: Option<String>,
    pub updated_at: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ApplicationDetail {
    pub application: Application,
    pub job: Job,
    pub resume: Option<ResumeVersion>,
    pub offer: Option<Offer>,
    pub notes: Vec<ApplicationNote>,
    pub contacts: Vec<ApplicationContact>,
    pub activities: Vec<Activity>,
    pub tasks: Vec<Task>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CreateApplication {
    pub profile_id: Option<String>,
    pub job_id: String,
    pub status: String,
    pub applied_at: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct UpdateApplication {
    pub status: Option<String>,
    pub due_date: Option<Option<String>>,
    pub outcome: Option<Option<ApplicationOutcome>>,
    pub outcome_date: Option<Option<String>>,
    pub rejection_stage: Option<Option<String>>,
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
pub struct CreateContact {
    pub name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub linkedin_url: Option<String>,
    pub company: Option<String>,
    pub role: Option<String>,
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
pub struct CreateApplicationContact {
    pub application_id: String,
    pub contact_id: String,
    pub relationship: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Offer {
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UpsertOffer {
    pub application_id: String,
    pub status: String,
    pub compensation_min: Option<i32>,
    pub compensation_max: Option<i32>,
    pub compensation_currency: Option<String>,
    pub starts_at: Option<String>,
    pub notes: Option<String>,
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
