use axum::Extension;
use axum::extract::State;
use serde::Serialize;

use crate::api::dto::profile::PersistedProfileAnalysisResponse;
use crate::api::dto::search_profile::SearchPreferencesResponse;
use crate::api::error::ApiError;
use crate::api::middleware::auth::AuthUser;
use crate::domain::application::model::{
    Activity, Application, ApplicationContact, ApplicationDetail, ApplicationNote, Contact, Offer,
    Task,
};
use crate::domain::feedback::model::{CompanyFeedbackRecord, JobFeedbackRecord};
use crate::domain::job::model::Job;
use crate::domain::profile::model::{LanguageProficiency, Profile};
use crate::state::AppState;

#[derive(Debug, Serialize)]
pub struct UserDataExportResponse {
    pub profile: ExportProfileResponse,
    pub feedback: ExportFeedbackResponse,
    pub companies: ExportCompaniesResponse,
    pub applications: Vec<ExportApplicationResponse>,
}

#[derive(Debug, Serialize)]
pub struct ExportProfileResponse {
    pub name: String,
    pub email: String,
    pub location: Option<String>,
    pub raw_text: String,
    pub years_of_experience: Option<i32>,
    pub salary_min: Option<i32>,
    pub salary_max: Option<i32>,
    pub salary_currency: String,
    pub languages: Vec<LanguageProficiency>,
    pub work_mode_preference: String,
    pub preferred_language: Option<String>,
    pub search_preferences: Option<SearchPreferencesResponse>,
    pub analysis: Option<PersistedProfileAnalysisResponse>,
    pub created_at: String,
    pub updated_at: String,
    pub skills_updated_at: Option<String>,
    pub portfolio_url: Option<String>,
    pub github_url: Option<String>,
    pub linkedin_url: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ExportFeedbackResponse {
    pub saved: Vec<ExportJobFeedbackResponse>,
    pub hidden: Vec<ExportJobFeedbackResponse>,
    pub bad_fit: Vec<ExportJobFeedbackResponse>,
}

#[derive(Debug, Serialize)]
pub struct ExportJobFeedbackResponse {
    pub job: Option<ExportJobSummaryResponse>,
    pub updated_at: String,
}

#[derive(Debug, Serialize)]
pub struct ExportJobSummaryResponse {
    pub title: String,
    pub company_name: String,
    pub location: Option<String>,
    pub remote_type: Option<String>,
    pub seniority: Option<String>,
    pub salary_min: Option<i32>,
    pub salary_max: Option<i32>,
    pub salary_currency: Option<String>,
    pub posted_at: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ExportCompaniesResponse {
    pub whitelist: Vec<String>,
    pub blacklist: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct ExportApplicationResponse {
    pub job: Option<ExportJobSummaryResponse>,
    pub status: String,
    pub applied_at: Option<String>,
    pub due_date: Option<String>,
    pub outcome: Option<String>,
    pub outcome_date: Option<String>,
    pub rejection_stage: Option<String>,
    pub updated_at: String,
    pub resume: Option<ExportResumeResponse>,
    pub offer: Option<ExportOfferResponse>,
    pub notes: Vec<ExportNoteResponse>,
    pub contacts: Vec<ExportApplicationContactResponse>,
    pub activities: Vec<ExportActivityResponse>,
    pub tasks: Vec<ExportTaskResponse>,
}

#[derive(Debug, Serialize)]
pub struct ExportResumeResponse {
    pub version: i32,
    pub filename: String,
    pub raw_text: String,
    pub is_active: bool,
    pub uploaded_at: String,
}

#[derive(Debug, Serialize)]
pub struct ExportOfferResponse {
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
pub struct ExportNoteResponse {
    pub content: String,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
pub struct ExportApplicationContactResponse {
    pub contact: ExportContactResponse,
    pub relationship: String,
}

#[derive(Debug, Serialize)]
pub struct ExportContactResponse {
    pub name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub linkedin_url: Option<String>,
    pub company: Option<String>,
    pub role: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
pub struct ExportActivityResponse {
    pub activity_type: String,
    pub description: String,
    pub happened_at: String,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
pub struct ExportTaskResponse {
    pub title: String,
    pub remind_at: Option<String>,
    pub done: bool,
    pub created_at: String,
}

pub async fn export_user_data(
    State(state): State<AppState>,
    auth: Option<Extension<AuthUser>>,
) -> Result<axum::Json<UserDataExportResponse>, ApiError> {
    let auth = auth.ok_or_else(|| {
        ApiError::unauthorized(
            "missing_auth",
            "Authentication is required to export user data",
        )
    })?;
    let profile_id = auth.profile_id.clone();

    let Some(profile) = state
        .profile_records
        .get_by_id(&profile_id)
        .await
        .map_err(|error| ApiError::from_repository(error, "profiles_query_failed"))?
    else {
        return Err(ApiError::not_found(
            "profile_not_found",
            format!("Profile '{profile_id}' was not found"),
        ));
    };

    let job_feedback = state
        .feedback_service
        .list_job_feedback(&profile_id)
        .await
        .map_err(|error| ApiError::from_repository(error, "feedback_query_failed"))?;
    let company_feedback = state
        .feedback_service
        .list_company_feedback(&profile_id)
        .await
        .map_err(|error| ApiError::from_repository(error, "feedback_query_failed"))?;
    let applications = state
        .applications_service
        .list_by_profile(&profile_id)
        .await
        .map_err(|error| ApiError::from_repository(error, "applications_query_failed"))?;

    let feedback = build_feedback_export(&state, job_feedback).await?;
    let companies = build_company_export(company_feedback);
    let applications = build_applications_export(&state, applications).await?;

    Ok(axum::Json(UserDataExportResponse {
        profile: ExportProfileResponse::from(profile),
        feedback,
        companies,
        applications,
    }))
}

async fn build_feedback_export(
    state: &AppState,
    records: Vec<JobFeedbackRecord>,
) -> Result<ExportFeedbackResponse, ApiError> {
    let mut saved = Vec::new();
    let mut hidden = Vec::new();
    let mut bad_fit = Vec::new();

    for record in records {
        let item = ExportJobFeedbackResponse {
            job: load_export_job_summary(state, &record.job_id).await?,
            updated_at: record.updated_at,
        };

        if record.saved {
            saved.push(item.clone());
        }
        if record.hidden {
            hidden.push(item.clone());
        }
        if record.bad_fit {
            bad_fit.push(item);
        }
    }

    Ok(ExportFeedbackResponse {
        saved,
        hidden,
        bad_fit,
    })
}

fn build_company_export(records: Vec<CompanyFeedbackRecord>) -> ExportCompaniesResponse {
    let mut whitelist = Vec::new();
    let mut blacklist = Vec::new();

    for record in records {
        match record.status {
            crate::domain::feedback::model::CompanyFeedbackStatus::Whitelist => {
                whitelist.push(record.company_name)
            }
            crate::domain::feedback::model::CompanyFeedbackStatus::Blacklist => {
                blacklist.push(record.company_name)
            }
        }
    }

    ExportCompaniesResponse {
        whitelist,
        blacklist,
    }
}

async fn build_applications_export(
    state: &AppState,
    applications: Vec<Application>,
) -> Result<Vec<ExportApplicationResponse>, ApiError> {
    let mut export = Vec::with_capacity(applications.len());

    for application in applications {
        let detail = state
            .applications_service
            .get_detail_by_id(&application.id)
            .await
            .map_err(|error| ApiError::from_repository(error, "applications_query_failed"))?;

        export.push(match detail {
            Some(detail) => ExportApplicationResponse::from(detail),
            None => ExportApplicationResponse::from(application),
        });
    }

    Ok(export)
}

async fn load_export_job_summary(
    state: &AppState,
    job_id: &str,
) -> Result<Option<ExportJobSummaryResponse>, ApiError> {
    state
        .jobs_service
        .get_by_id(job_id)
        .await
        .map(|job| job.map(ExportJobSummaryResponse::from))
        .map_err(|error| ApiError::from_repository(error, "jobs_query_failed"))
}

impl Clone for ExportJobFeedbackResponse {
    fn clone(&self) -> Self {
        Self {
            job: self.job.clone(),
            updated_at: self.updated_at.clone(),
        }
    }
}

impl Clone for ExportJobSummaryResponse {
    fn clone(&self) -> Self {
        Self {
            title: self.title.clone(),
            company_name: self.company_name.clone(),
            location: self.location.clone(),
            remote_type: self.remote_type.clone(),
            seniority: self.seniority.clone(),
            salary_min: self.salary_min,
            salary_max: self.salary_max,
            salary_currency: self.salary_currency.clone(),
            posted_at: self.posted_at.clone(),
        }
    }
}

impl From<Profile> for ExportProfileResponse {
    fn from(profile: Profile) -> Self {
        Self {
            name: profile.name,
            email: profile.email,
            location: profile.location,
            raw_text: profile.raw_text,
            years_of_experience: profile.years_of_experience,
            salary_min: profile.salary_min,
            salary_max: profile.salary_max,
            salary_currency: profile.salary_currency,
            languages: profile.languages,
            work_mode_preference: profile.work_mode_preference,
            preferred_language: profile.preferred_language,
            search_preferences: profile
                .search_preferences
                .map(SearchPreferencesResponse::from),
            analysis: profile.analysis.map(PersistedProfileAnalysisResponse::from),
            created_at: profile.created_at,
            updated_at: profile.updated_at,
            skills_updated_at: profile.skills_updated_at,
            portfolio_url: profile.portfolio_url,
            github_url: profile.github_url,
            linkedin_url: profile.linkedin_url,
        }
    }
}

impl From<Job> for ExportJobSummaryResponse {
    fn from(job: Job) -> Self {
        Self {
            title: job.title,
            company_name: job.company_name,
            location: job.location,
            remote_type: job.remote_type,
            seniority: job.seniority,
            salary_min: job.salary_min,
            salary_max: job.salary_max,
            salary_currency: job.salary_currency,
            posted_at: job.posted_at,
        }
    }
}

impl From<Application> for ExportApplicationResponse {
    fn from(application: Application) -> Self {
        Self {
            job: None,
            status: application.status,
            applied_at: application.applied_at,
            due_date: application.due_date,
            outcome: application
                .outcome
                .map(|outcome| outcome.as_str().to_string()),
            outcome_date: application.outcome_date,
            rejection_stage: application.rejection_stage,
            updated_at: application.updated_at,
            resume: None,
            offer: None,
            notes: Vec::new(),
            contacts: Vec::new(),
            activities: Vec::new(),
            tasks: Vec::new(),
        }
    }
}

impl From<ApplicationDetail> for ExportApplicationResponse {
    fn from(detail: ApplicationDetail) -> Self {
        Self {
            job: Some(ExportJobSummaryResponse::from(detail.job)),
            status: detail.application.status,
            applied_at: detail.application.applied_at,
            due_date: detail.application.due_date,
            outcome: detail
                .application
                .outcome
                .map(|outcome| outcome.as_str().to_string()),
            outcome_date: detail.application.outcome_date,
            rejection_stage: detail.application.rejection_stage,
            updated_at: detail.application.updated_at,
            resume: detail.resume.map(|resume| ExportResumeResponse {
                version: resume.version,
                filename: resume.filename,
                raw_text: resume.raw_text,
                is_active: resume.is_active,
                uploaded_at: resume.uploaded_at,
            }),
            offer: detail.offer.map(ExportOfferResponse::from),
            notes: detail
                .notes
                .into_iter()
                .map(ExportNoteResponse::from)
                .collect(),
            contacts: detail
                .contacts
                .into_iter()
                .map(ExportApplicationContactResponse::from)
                .collect(),
            activities: detail
                .activities
                .into_iter()
                .map(ExportActivityResponse::from)
                .collect(),
            tasks: detail
                .tasks
                .into_iter()
                .map(ExportTaskResponse::from)
                .collect(),
        }
    }
}

impl From<Offer> for ExportOfferResponse {
    fn from(offer: Offer) -> Self {
        Self {
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

impl From<ApplicationNote> for ExportNoteResponse {
    fn from(note: ApplicationNote) -> Self {
        Self {
            content: note.content,
            created_at: note.created_at,
        }
    }
}

impl From<ApplicationContact> for ExportApplicationContactResponse {
    fn from(contact: ApplicationContact) -> Self {
        Self {
            contact: ExportContactResponse::from(contact.contact),
            relationship: contact.relationship,
        }
    }
}

impl From<Contact> for ExportContactResponse {
    fn from(contact: Contact) -> Self {
        Self {
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

impl From<Activity> for ExportActivityResponse {
    fn from(activity: Activity) -> Self {
        Self {
            activity_type: activity.activity_type,
            description: activity.description,
            happened_at: activity.happened_at,
            created_at: activity.created_at,
        }
    }
}

impl From<Task> for ExportTaskResponse {
    fn from(task: Task) -> Self {
        Self {
            title: task.title,
            remind_at: task.remind_at,
            done: task.done,
            created_at: task.created_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use axum::Extension;
    use axum::extract::State;
    use axum::http::StatusCode;
    use axum::response::IntoResponse;
    use serde_json::json;

    use crate::api::middleware::auth::AuthUser;
    use crate::domain::application::model::Application;
    use crate::domain::feedback::model::{
        CompanyFeedbackRecord, CompanyFeedbackStatus, JobFeedbackRecord,
    };
    use crate::domain::job::model::Job;
    use crate::domain::profile::model::{LanguageLevel, LanguageProficiency, Profile};
    use crate::services::applications::{ApplicationsService, ApplicationsServiceStub};
    use crate::services::feedback::{FeedbackService, FeedbackServiceStub};
    use crate::services::jobs::{JobsService, JobsServiceStub};
    use crate::services::profiles::{ProfilesService, ProfilesServiceStub};
    use crate::services::resumes::{ResumesService, ResumesServiceStub};
    use crate::state::AppState;

    use super::export_user_data;

    fn sample_profile() -> Profile {
        Profile {
            id: "profile-1".to_string(),
            name: "Jane Doe".to_string(),
            email: "jane@example.com".to_string(),
            location: Some("Kyiv".to_string()),
            raw_text: "Senior backend engineer".to_string(),
            analysis: None,
            years_of_experience: Some(7),
            salary_min: None,
            salary_max: None,
            salary_currency: "USD".to_string(),
            languages: vec![LanguageProficiency {
                language: "English".to_string(),
                level: LanguageLevel::C1,
            }],
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

    fn sample_job() -> Job {
        Job {
            id: "job-1".to_string(),
            title: "Senior Backend Developer".to_string(),
            company_name: "NovaLedger".to_string(),
            location: Some("Kyiv".to_string()),
            remote_type: Some("remote".to_string()),
            seniority: Some("senior".to_string()),
            description_text: "Rust and Postgres".to_string(),
            salary_min: Some(5000),
            salary_max: Some(7000),
            salary_currency: Some("USD".to_string()),
            language: None,
            posted_at: Some("2026-04-10T00:00:00Z".to_string()),
            last_seen_at: "2026-04-14T00:00:00Z".to_string(),
            is_active: true,
        }
    }

    fn sample_application() -> Application {
        Application {
            id: "application-1".to_string(),
            job_id: "job-1".to_string(),
            resume_id: None,
            status: "applied".to_string(),
            applied_at: Some("2026-04-12T00:00:00Z".to_string()),
            due_date: None,
            outcome: None,
            outcome_date: None,
            rejection_stage: None,
            updated_at: "2026-04-12T00:00:00Z".to_string(),
        }
    }

    fn test_state() -> AppState {
        AppState::for_services(
            ProfilesService::for_tests(
                ProfilesServiceStub::default().with_profile(sample_profile()),
            ),
            JobsService::for_tests(JobsServiceStub::default().with_job(sample_job())),
            ApplicationsService::for_tests(
                ApplicationsServiceStub::default().with_application(sample_application()),
            ),
            ResumesService::for_tests(ResumesServiceStub::default()),
        )
        .with_feedback_service(FeedbackService::for_tests(
            FeedbackServiceStub::default()
                .with_job_feedback(JobFeedbackRecord {
                    profile_id: "profile-1".to_string(),
                    job_id: "job-1".to_string(),
                    saved: true,
                    hidden: true,
                    bad_fit: false,
                    salary_signal: None,
                    interest_rating: None,
                    work_mode_signal: None,
                    legitimacy_signal: None,
                    created_at: "2026-04-14T00:00:00Z".to_string(),
                    updated_at: "2026-04-14T00:00:00Z".to_string(),
                })
                .with_company_feedback(CompanyFeedbackRecord {
                    profile_id: "profile-1".to_string(),
                    company_name: "NovaLedger".to_string(),
                    normalized_company_name: "novaledger".to_string(),
                    status: CompanyFeedbackStatus::Blacklist,
                    created_at: "2026-04-14T00:00:00Z".to_string(),
                    updated_at: "2026-04-14T00:00:00Z".to_string(),
                }),
        ))
    }

    #[tokio::test]
    async fn export_requires_auth() {
        let error = export_user_data(State(test_state()), None)
            .await
            .expect_err("export should require auth");

        assert_eq!(error.into_response().status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn export_returns_profile_scoped_user_data_without_internal_ids() {
        let response = export_user_data(
            State(test_state()),
            Some(Extension(AuthUser {
                profile_id: "profile-1".to_string(),
            })),
        )
        .await
        .expect("export should succeed");

        let payload = serde_json::to_value(response.0).expect("export should serialize");

        assert_eq!(payload["profile"]["name"], json!("Jane Doe"));
        assert!(payload["profile"].get("id").is_none());
        assert_eq!(
            payload["feedback"]["saved"][0]["job"]["title"],
            json!("Senior Backend Developer")
        );
        assert_eq!(
            payload["feedback"]["hidden"].as_array().map(Vec::len),
            Some(1)
        );
        assert_eq!(
            payload["feedback"]["bad_fit"].as_array().map(Vec::len),
            Some(0)
        );
        assert_eq!(payload["companies"]["blacklist"], json!(["NovaLedger"]));
        assert_eq!(payload["applications"][0]["status"], json!("applied"));
        assert!(payload["applications"][0].get("id").is_none());
        assert!(payload["applications"][0].get("job_id").is_none());
    }
}
