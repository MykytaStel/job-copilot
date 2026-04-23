use std::collections::BTreeMap;

use chrono::{DateTime, Utc};

use crate::domain::application::model::{Application, ApplicationOutcome};
use crate::domain::feedback::model::JobFeedbackState;
use crate::domain::profile::model::ProfileAnalysis;
use crate::domain::search::profile::{SearchProfile, SearchRoleCandidate};
use crate::domain::user_event::model::{UserEventRecord, UserEventType};

use super::ids::normalized_job_id;
use super::types::OutcomeSignals;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct EventSignals {
    pub viewed: bool,
    pub saved: bool,
    pub hidden: bool,
    pub bad_fit: bool,
    pub applied: bool,
    pub viewed_event_count: usize,
    pub saved_event_count: usize,
    pub applied_event_count: usize,
    pub dismissed_event_count: usize,
    pub scrolled_to_bottom: bool,
    pub returned_count: usize,
    pub first_viewed_at: Option<String>,
    pub first_applied_at: Option<String>,
    pub latest_event_at: Option<String>,
}

pub(super) fn search_profile_from_analysis(analysis: &ProfileAnalysis) -> SearchProfile {
    SearchProfile {
        primary_role: analysis.primary_role,
        primary_role_confidence: None,
        target_roles: Vec::new(),
        role_candidates: vec![SearchRoleCandidate {
            role: analysis.primary_role,
            confidence: 100,
        }],
        seniority: analysis.seniority.clone(),
        target_regions: Vec::new(),
        work_modes: Vec::new(),
        allowed_sources: Vec::new(),
        profile_skills: analysis.skills.clone(),
        profile_keywords: analysis.keywords.clone(),
        search_terms: vec![analysis.primary_role.search_label()],
        exclude_terms: Vec::new(),
    }
}

pub(crate) fn event_signals_by_job_id(
    events: &[UserEventRecord],
) -> BTreeMap<String, EventSignals> {
    let mut ordered_events = events
        .iter()
        .filter_map(|event| {
            normalized_job_id(event.job_id.as_deref()).map(|job_id| (job_id, event))
        })
        .collect::<Vec<_>>();
    ordered_events.sort_by(|(left_job_id, left_event), (right_job_id, right_event)| {
        left_job_id
            .cmp(right_job_id)
            .then_with(|| left_event.created_at.cmp(&right_event.created_at))
            .then_with(|| left_event.id.cmp(&right_event.id))
    });

    let mut signals_by_job_id = BTreeMap::<String, EventSignals>::new();

    for (job_id, event) in ordered_events {
        let signals = signals_by_job_id.entry(job_id).or_default();

        match event.event_type {
            UserEventType::JobOpened => {
                signals.viewed = true;
                signals.viewed_event_count += 1;
                if signals.first_viewed_at.is_none() {
                    signals.first_viewed_at = Some(event.created_at.clone());
                }
            }
            UserEventType::JobSaved => {
                signals.saved = true;
                signals.saved_event_count += 1;
            }
            UserEventType::JobUnsaved => signals.saved = false,
            UserEventType::JobHidden => {
                signals.hidden = true;
                signals.dismissed_event_count += 1;
            }
            UserEventType::JobUnhidden => signals.hidden = false,
            UserEventType::JobBadFit => {
                signals.bad_fit = true;
                signals.dismissed_event_count += 1;
            }
            UserEventType::JobBadFitRemoved => signals.bad_fit = false,
            UserEventType::ApplicationCreated => {
                signals.applied = true;
                signals.applied_event_count += 1;
                if signals.first_applied_at.is_none() {
                    signals.first_applied_at = Some(event.created_at.clone());
                }
            }
            UserEventType::JobScrolledToBottom => {
                signals.scrolled_to_bottom = true;
            }
            UserEventType::JobReturned => {
                signals.returned_count += 1;
            }
            _ => {}
        }

        signals.latest_event_at = Some(event.created_at.clone());
    }

    signals_by_job_id
}

pub(crate) fn normalize_signals(
    feedback: &JobFeedbackState,
    event_signals: &EventSignals,
    application: Option<&Application>,
) -> OutcomeSignals {
    let saved = feedback.saved || event_signals.saved;
    let hidden = feedback.hidden || event_signals.hidden;
    let bad_fit = feedback.bad_fit || event_signals.bad_fit;

    let rejection_tags: Vec<String> = feedback
        .tags
        .iter()
        .filter(|tag| tag.is_negative())
        .map(|tag| tag.as_str().to_string())
        .collect();
    let positive_tags: Vec<String> = feedback
        .tags
        .iter()
        .filter(|tag| !tag.is_negative())
        .map(|tag| tag.as_str().to_string())
        .collect();
    let has_salary_rejection = rejection_tags.contains(&"salary_too_low".to_string());
    let has_remote_rejection = rejection_tags.contains(&"not_remote".to_string());
    let has_tech_rejection = rejection_tags.contains(&"bad_tech_stack".to_string());

    let salary_signal = feedback
        .salary_signal
        .map(|signal| signal.as_str().to_string());
    let salary_below_expectation = feedback.salary_signal.is_some_and(|signal| {
        matches!(
            signal,
            crate::domain::feedback::model::SalaryFeedbackSignal::BelowExpectation
        )
    });

    let work_mode_deal_breaker = feedback.work_mode_signal.is_some_and(|signal| {
        matches!(
            signal,
            crate::domain::feedback::model::WorkModeFeedbackSignal::DealBreaker
        )
    });

    let legitimacy_suspicious = feedback.legitimacy_signal.is_some_and(|signal| {
        matches!(
            signal,
            crate::domain::feedback::model::LegitimacySignal::Suspicious
        )
    });
    let legitimacy_spam = feedback.legitimacy_signal.is_some_and(|signal| {
        matches!(
            signal,
            crate::domain::feedback::model::LegitimacySignal::Spam
        )
    });
    let outcome = application
        .and_then(|record| record.outcome)
        .map(ApplicationOutcome::as_str)
        .map(str::to_string);
    let reached_interview = application.is_some_and(application_reached_interview);
    let received_offer = application
        .is_some_and(|record| matches!(record.outcome, Some(ApplicationOutcome::OfferReceived)));
    let was_rejected = application
        .is_some_and(|record| matches!(record.outcome, Some(ApplicationOutcome::Rejected)));
    let was_ghosted = application
        .is_some_and(|record| matches!(record.outcome, Some(ApplicationOutcome::Ghosted)));
    let time_to_apply_days = resolve_time_to_apply_days(event_signals, application);

    OutcomeSignals {
        viewed: event_signals.viewed,
        saved,
        hidden,
        bad_fit,
        applied: event_signals.applied,
        dismissed: hidden || bad_fit,
        explicit_feedback: feedback.saved || feedback.hidden || feedback.bad_fit,
        explicit_saved: feedback.saved,
        explicit_hidden: feedback.hidden,
        explicit_bad_fit: feedback.bad_fit,
        viewed_event_count: event_signals.viewed_event_count,
        saved_event_count: event_signals.saved_event_count,
        applied_event_count: event_signals.applied_event_count,
        dismissed_event_count: event_signals.dismissed_event_count,
        outcome,
        reached_interview,
        received_offer,
        was_rejected,
        was_ghosted,
        rejection_tags,
        positive_tags,
        has_salary_rejection,
        has_remote_rejection,
        has_tech_rejection,
        salary_signal,
        salary_below_expectation,
        interest_rating: feedback.interest_rating,
        work_mode_deal_breaker,
        scrolled_to_bottom: event_signals.scrolled_to_bottom,
        returned_count: event_signals.returned_count,
        time_to_apply_days,
        legitimacy_suspicious,
        legitimacy_spam,
    }
}

fn resolve_time_to_apply_days(
    event_signals: &EventSignals,
    application: Option<&Application>,
) -> Option<u32> {
    let first_viewed_at = parse_timestamp(event_signals.first_viewed_at.as_deref())?;
    let applied_at = application
        .and_then(|record| record.applied_at.as_deref())
        .and_then(|value| parse_timestamp(Some(value)))
        .or_else(|| parse_timestamp(event_signals.first_applied_at.as_deref()))?;

    let duration = applied_at.signed_duration_since(first_viewed_at);
    let days = duration.num_days().max(0);
    u32::try_from(days).ok()
}

fn parse_timestamp(value: Option<&str>) -> Option<DateTime<Utc>> {
    let value = value?.trim();
    if value.is_empty() {
        return None;
    }

    DateTime::parse_from_rfc3339(value)
        .ok()
        .map(|timestamp| timestamp.with_timezone(&Utc))
}

fn application_reached_interview(application: &Application) -> bool {
    matches!(
        application.outcome,
        Some(
            ApplicationOutcome::PhoneScreen
                | ApplicationOutcome::TechnicalInterview
                | ApplicationOutcome::FinalInterview
                | ApplicationOutcome::OfferReceived
        )
    )
}
