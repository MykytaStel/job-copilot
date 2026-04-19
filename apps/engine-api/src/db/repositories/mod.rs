mod activities;
mod applications;
mod feedback;
mod fit_scores;
mod jobs;
mod notifications;
mod profiles;
mod resumes;
mod tasks;
mod user_events;

use std::error::Error;
use std::fmt;

pub use activities::ActivitiesRepository;
pub use applications::ApplicationsRepository;
pub use feedback::FeedbackRepository;
pub use fit_scores::FitScoresRepository;
pub use jobs::JobsRepository;
pub use notifications::NotificationsRepository;
pub use profiles::ProfilesRepository;
pub use resumes::ResumesRepository;
pub use tasks::TasksRepository;
pub use user_events::UserEventsRepository;

#[derive(Debug)]
pub enum RepositoryError {
    DatabaseDisabled,
    Sqlx(sqlx::Error),
    Json(serde_json::Error),
    InvalidData { message: String },
}

impl fmt::Display for RepositoryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DatabaseDisabled => formatter.write_str("database is not configured"),
            Self::Sqlx(error) => write!(formatter, "database query failed: {error}"),
            Self::Json(error) => write!(formatter, "json conversion failed: {error}"),
            Self::InvalidData { message } => formatter.write_str(message),
        }
    }
}

impl Error for RepositoryError {}

impl From<sqlx::Error> for RepositoryError {
    fn from(error: sqlx::Error) -> Self {
        Self::Sqlx(error)
    }
}

impl From<serde_json::Error> for RepositoryError {
    fn from(error: serde_json::Error) -> Self {
        Self::Json(error)
    }
}
