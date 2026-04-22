use serde_json::Value;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum UserEventType {
    JobImpression,
    JobOpened,
    JobSaved,
    JobUnsaved,
    JobHidden,
    JobUnhidden,
    JobBadFit,
    JobBadFitRemoved,
    CompanyWhitelisted,
    CompanyBlacklisted,
    SearchRun,
    FitExplanationRequested,
    ApplicationCoachRequested,
    CoverLetterDraftRequested,
    InterviewPrepRequested,
    ApplicationCreated,
    JobScrolledToBottom,
    JobReturned,
    JobShared,
}

impl UserEventType {
    pub const ALL: [Self; 19] = [
        Self::JobImpression,
        Self::JobOpened,
        Self::JobSaved,
        Self::JobUnsaved,
        Self::JobHidden,
        Self::JobUnhidden,
        Self::JobBadFit,
        Self::JobBadFitRemoved,
        Self::CompanyWhitelisted,
        Self::CompanyBlacklisted,
        Self::SearchRun,
        Self::FitExplanationRequested,
        Self::ApplicationCoachRequested,
        Self::CoverLetterDraftRequested,
        Self::InterviewPrepRequested,
        Self::ApplicationCreated,
        Self::JobScrolledToBottom,
        Self::JobReturned,
        Self::JobShared,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::JobImpression => "job_impression",
            Self::JobOpened => "job_opened",
            Self::JobSaved => "job_saved",
            Self::JobUnsaved => "job_unsaved",
            Self::JobHidden => "job_hidden",
            Self::JobUnhidden => "job_unhidden",
            Self::JobBadFit => "job_bad_fit",
            Self::JobBadFitRemoved => "job_bad_fit_removed",
            Self::CompanyWhitelisted => "company_whitelisted",
            Self::CompanyBlacklisted => "company_blacklisted",
            Self::SearchRun => "search_run",
            Self::FitExplanationRequested => "fit_explanation_requested",
            Self::ApplicationCoachRequested => "application_coach_requested",
            Self::CoverLetterDraftRequested => "cover_letter_draft_requested",
            Self::InterviewPrepRequested => "interview_prep_requested",
            Self::ApplicationCreated => "application_created",
            Self::JobScrolledToBottom => "job_scrolled_to_bottom",
            Self::JobReturned => "job_returned",
            Self::JobShared => "job_shared",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value.trim() {
            "job_impression" => Some(Self::JobImpression),
            "job_opened" => Some(Self::JobOpened),
            "job_saved" => Some(Self::JobSaved),
            "job_unsaved" => Some(Self::JobUnsaved),
            "job_hidden" => Some(Self::JobHidden),
            "job_unhidden" => Some(Self::JobUnhidden),
            "job_bad_fit" => Some(Self::JobBadFit),
            "job_bad_fit_removed" => Some(Self::JobBadFitRemoved),
            "company_whitelisted" => Some(Self::CompanyWhitelisted),
            "company_blacklisted" => Some(Self::CompanyBlacklisted),
            "search_run" => Some(Self::SearchRun),
            "fit_explanation_requested" => Some(Self::FitExplanationRequested),
            "application_coach_requested" => Some(Self::ApplicationCoachRequested),
            "cover_letter_draft_requested" => Some(Self::CoverLetterDraftRequested),
            "interview_prep_requested" => Some(Self::InterviewPrepRequested),
            "application_created" => Some(Self::ApplicationCreated),
            "job_scrolled_to_bottom" => Some(Self::JobScrolledToBottom),
            "job_returned" => Some(Self::JobReturned),
            "job_shared" => Some(Self::JobShared),
            _ => None,
        }
    }

    pub fn allowed_values() -> Vec<&'static str> {
        Self::ALL.into_iter().map(Self::as_str).collect()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CreateUserEvent {
    pub profile_id: String,
    pub event_type: UserEventType,
    pub job_id: Option<String>,
    pub company_name: Option<String>,
    pub source: Option<String>,
    pub role_family: Option<String>,
    pub payload_json: Option<Value>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct UserEventRecord {
    pub id: String,
    pub profile_id: String,
    pub event_type: UserEventType,
    pub job_id: Option<String>,
    pub company_name: Option<String>,
    pub source: Option<String>,
    pub role_family: Option<String>,
    pub payload_json: Option<Value>,
    pub created_at: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct UserEventSummary {
    pub save_count: usize,
    pub hide_count: usize,
    pub bad_fit_count: usize,
    pub search_run_count: usize,
    pub fit_explanation_requested_count: usize,
    pub application_coach_requested_count: usize,
    pub cover_letter_draft_requested_count: usize,
    pub interview_prep_requested_count: usize,
}

impl UserEventSummary {
    pub fn from_events<'a>(events: impl IntoIterator<Item = &'a UserEventRecord>) -> Self {
        let mut summary = Self::default();

        for event in events {
            match event.event_type {
                UserEventType::JobSaved => summary.save_count += 1,
                UserEventType::JobHidden => summary.hide_count += 1,
                UserEventType::JobBadFit => summary.bad_fit_count += 1,
                UserEventType::SearchRun => summary.search_run_count += 1,
                UserEventType::FitExplanationRequested => {
                    summary.fit_explanation_requested_count += 1;
                }
                UserEventType::ApplicationCoachRequested => {
                    summary.application_coach_requested_count += 1;
                }
                UserEventType::CoverLetterDraftRequested => {
                    summary.cover_letter_draft_requested_count += 1;
                }
                UserEventType::InterviewPrepRequested => {
                    summary.interview_prep_requested_count += 1;
                }
                _ => {}
            }
        }

        summary
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{UserEventRecord, UserEventSummary, UserEventType};

    #[test]
    fn parses_all_known_event_types() {
        for event_type in UserEventType::ALL {
            assert_eq!(
                UserEventType::parse(event_type.as_str()),
                Some(event_type),
                "event type should round-trip through string form"
            );
        }
    }

    #[test]
    fn summary_counts_only_requested_learning_signals() {
        let events = vec![
            UserEventRecord {
                id: "evt-1".to_string(),
                profile_id: "profile-1".to_string(),
                event_type: UserEventType::JobSaved,
                job_id: Some("job-1".to_string()),
                company_name: Some("NovaLedger".to_string()),
                source: Some("djinni".to_string()),
                role_family: None,
                payload_json: None,
                created_at: "2026-04-15T00:00:00Z".to_string(),
            },
            UserEventRecord {
                id: "evt-2".to_string(),
                profile_id: "profile-1".to_string(),
                event_type: UserEventType::SearchRun,
                job_id: None,
                company_name: None,
                source: None,
                role_family: Some("engineering".to_string()),
                payload_json: Some(json!({ "returned_jobs": 12 })),
                created_at: "2026-04-15T00:00:01Z".to_string(),
            },
            UserEventRecord {
                id: "evt-3".to_string(),
                profile_id: "profile-1".to_string(),
                event_type: UserEventType::ApplicationCreated,
                job_id: Some("job-1".to_string()),
                company_name: Some("NovaLedger".to_string()),
                source: Some("djinni".to_string()),
                role_family: None,
                payload_json: None,
                created_at: "2026-04-15T00:00:02Z".to_string(),
            },
        ];

        let summary = UserEventSummary::from_events(events.iter());

        assert_eq!(summary.save_count, 1);
        assert_eq!(summary.search_run_count, 1);
        assert_eq!(summary.hide_count, 0);
        assert_eq!(summary.bad_fit_count, 0);
        assert_eq!(summary.fit_explanation_requested_count, 0);
    }
}
