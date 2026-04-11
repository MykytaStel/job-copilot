use crate::db::repositories::TasksRepository;
use crate::domain::application::model::CreateTask;

/// Auto-creates follow-up reminder tasks after key application status changes.
///
/// All methods are fire-and-forget: errors are swallowed so they never fail
/// the primary operation that triggered them.
#[derive(Clone)]
pub struct FollowUpService {
    tasks_repository: TasksRepository,
}

impl FollowUpService {
    pub fn new(tasks_repository: TasksRepository) -> Self {
        Self { tasks_repository }
    }

    /// Called after an application status changes.
    /// When the new status is "applied", creates a 7-day follow-up reminder
    /// if one does not already exist.
    pub async fn on_status_change(&self, application_id: &str, new_status: &str) {
        if new_status != "applied" {
            return;
        }

        const TITLE: &str = "Follow up on application";

        let already_exists = self
            .tasks_repository
            .has_followup_task(application_id, TITLE)
            .await
            .unwrap_or(true); // if DB unavailable, skip silently

        if !already_exists {
            let _ = self
                .tasks_repository
                .create(&CreateTask {
                    application_id: application_id.to_string(),
                    title: TITLE.to_string(),
                    remind_in_days: 7,
                })
                .await;
        }
    }

    /// Called after an interview activity is logged for an application.
    /// Creates a 1-day "send thank-you note" reminder if one does not already exist.
    pub async fn on_interview_activity(&self, application_id: &str) {
        const TITLE: &str = "Send thank-you note";

        let already_exists = self
            .tasks_repository
            .has_followup_task(application_id, TITLE)
            .await
            .unwrap_or(true); // if DB unavailable, skip silently

        if !already_exists {
            let _ = self
                .tasks_repository
                .create(&CreateTask {
                    application_id: application_id.to_string(),
                    title: TITLE.to_string(),
                    remind_in_days: 1,
                })
                .await;
        }
    }
}
