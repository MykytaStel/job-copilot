use std::collections::{BTreeMap, BTreeSet};

use serde_json::Value;

use crate::domain::application::model::{Application, ApplicationOutcome};
use crate::domain::feedback::model::JobFeedbackState;
use crate::domain::job::model::JobView;
use crate::domain::matching::JobFit;
use crate::domain::profile::model::{Profile, ProfileAnalysis};
use crate::domain::search::profile::{SearchProfile, SearchRoleCandidate};
use crate::domain::user_event::model::{UserEventRecord, UserEventType};
use crate::services::behavior::{BehaviorService, ProfileBehaviorAggregates};
use crate::services::funnel::ProfileFunnelAggregates;
use crate::services::learned_reranker::LearnedRerankerService;
use crate::services::search_ranking::SearchRankingService;

pub const OUTCOME_LABEL_POLICY_VERSION: &str = "outcome_label_v3";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OutcomeDataset {
    pub profile_id: String,
    pub label_policy_version: String,
    pub examples: Vec<OutcomeExample>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OutcomeExample {
    pub profile_id: String,
    pub job_id: String,
    pub title: String,
    pub company_name: String,
    pub source: Option<String>,
    pub role_family: Option<String>,
    pub label_observed_at: Option<String>,
    pub label: OutcomeLabel,
    pub label_score: u8,
    pub label_reasons: Vec<String>,
    pub signals: OutcomeSignals,
    pub ranking: OutcomeRankingFeatures,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OutcomeLabel {
    Positive,
    Medium,
    Negative,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct OutcomeSignals {
    pub viewed: bool,
    pub saved: bool,
    pub hidden: bool,
    pub bad_fit: bool,
    pub applied: bool,
    pub dismissed: bool,
    pub explicit_feedback: bool,
    pub explicit_saved: bool,
    pub explicit_hidden: bool,
    pub explicit_bad_fit: bool,
    pub viewed_event_count: usize,
    pub saved_event_count: usize,
    pub applied_event_count: usize,
    pub dismissed_event_count: usize,
    // Slice 1: application outcome
    pub outcome: Option<String>,
    pub reached_interview: bool,
    pub received_offer: bool,
    pub was_rejected: bool,
    pub was_ghosted: bool,
    // Slice 2: structured rejection/interest tags
    pub rejection_tags: Vec<String>,
    pub positive_tags: Vec<String>,
    pub has_salary_rejection: bool,
    pub has_remote_rejection: bool,
    pub has_tech_rejection: bool,
    // Slice 3: salary signal
    pub salary_signal: Option<String>,
    pub salary_below_expectation: bool,
    // Slice 4: interest rating
    pub interest_rating: Option<i8>,
    // Slice 5: work mode signal
    pub work_mode_deal_breaker: bool,
    // Slice 6: engagement depth
    pub scrolled_to_bottom: bool,
    pub returned_count: usize,
    // Slice 7: legitimacy
    pub legitimacy_suspicious: bool,
    pub legitimacy_spam: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OutcomeRankingFeatures {
    pub deterministic_score: u8,
    pub behavior_score_delta: i16,
    pub behavior_score: u8,
    pub learned_reranker_score_delta: i16,
    pub learned_reranker_score: u8,
    pub matched_roles: Vec<String>,
    pub matched_skills: Vec<String>,
    pub matched_keywords: Vec<String>,
    pub matched_role_count: usize,
    pub matched_skill_count: usize,
    pub matched_keyword_count: usize,
    pub fit_reasons: Vec<String>,
    pub behavior_reasons: Vec<String>,
    pub learned_reasons: Vec<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OutcomeDatasetError {
    ProfileAnalysisRequired,
}

#[derive(Clone, Default)]
pub struct OutcomeDatasetService;

impl OutcomeDatasetService {
    pub fn new() -> Self {
        Self
    }

    pub fn build(
        &self,
        profile: &Profile,
        events: &[UserEventRecord],
        jobs: Vec<(JobView, JobFeedbackState)>,
        applications_by_job_id: &BTreeMap<String, Application>,
        feedback_updated_at_by_job_id: &BTreeMap<String, String>,
        search_ranking_service: &SearchRankingService,
        behavior: &ProfileBehaviorAggregates,
        funnel: &ProfileFunnelAggregates,
    ) -> Result<OutcomeDataset, OutcomeDatasetError> {
        let Some(analysis) = profile.analysis.as_ref() else {
            return Err(OutcomeDatasetError::ProfileAnalysisRequired);
        };

        let search_profile = search_profile_from_analysis(analysis);
        let event_signals_by_job_id = event_signals_by_job_id(events);
        let behavior_service = BehaviorService::new();
        let learned_reranker = LearnedRerankerService::new();
        let mut examples = Vec::new();

        for (job, feedback) in jobs {
            let job_id = job.job.id.clone();
            let event_signals = event_signals_by_job_id
                .get(&job_id)
                .cloned()
                .unwrap_or_default();
            let application = applications_by_job_id.get(&job_id);
            let signals = normalize_signals(&feedback, &event_signals, application);
            let Some(label_assignment) = assign_label(&signals) else {
                continue;
            };
            let label_observed_at = resolve_label_observed_at(
                &signals,
                &event_signals,
                application,
                feedback_updated_at_by_job_id.get(&job_id).map(String::as_str),
            );

            let fit = search_ranking_service.score_job(&search_profile, &job);
            let source = job
                .primary_variant
                .as_ref()
                .map(|variant| variant.source.clone());
            let role_family = search_ranking_service.infer_role_family(&job);
            let behavior_adjustment =
                behavior_service.score_job(behavior, source.as_deref(), role_family.as_deref());
            let behavior_score =
                (fit.score as i16 + behavior_adjustment.score_delta).clamp(0, 100) as u8;
            let learned_score = learned_reranker.score_job(
                fit.score,
                source.as_deref(),
                role_family.as_deref(),
                behavior,
                funnel,
                &feedback,
            );
            let learned_reranker_score =
                (behavior_score as i16 + learned_score.score_delta).clamp(0, 100) as u8;
            let ranking = ranking_features(
                fit,
                behavior_adjustment.score_delta,
                behavior_score,
                behavior_adjustment.reasons,
                learned_score.score_delta,
                learned_reranker_score,
                learned_score.reasons,
            );

            examples.push(OutcomeExample {
                profile_id: profile.id.clone(),
                job_id,
                title: job.job.title,
                company_name: job.job.company_name,
                source,
                role_family,
                label_observed_at,
                label: label_assignment.label,
                label_score: label_assignment.label_score,
                label_reasons: label_assignment.reasons,
                signals,
                ranking,
            });
        }

        examples.sort_by(|left, right| left.job_id.cmp(&right.job_id));

        Ok(OutcomeDataset {
            profile_id: profile.id.clone(),
            label_policy_version: OUTCOME_LABEL_POLICY_VERSION.to_string(),
            examples,
        })
    }
}

impl OutcomeLabel {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Positive => "positive",
            Self::Medium => "medium",
            Self::Negative => "negative",
        }
    }
}

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
    pub latest_event_at: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct OutcomeLabelAssignment {
    label: OutcomeLabel,
    label_score: u8,
    reasons: Vec<String>,
}

pub fn outcome_job_ids(events: &[UserEventRecord], feedback_job_ids: &[String]) -> Vec<String> {
    let mut job_ids = BTreeSet::new();

    for event in events {
        if !matches!(
            event.event_type,
            UserEventType::JobOpened
                | UserEventType::JobSaved
                | UserEventType::JobHidden
                | UserEventType::JobBadFit
                | UserEventType::ApplicationCreated
        ) {
            continue;
        }

        if let Some(job_id) = normalized_job_id(event.job_id.as_deref()) {
            job_ids.insert(job_id);
        }
    }

    for job_id in feedback_job_ids {
        if let Some(job_id) = normalized_job_id(Some(job_id.as_str())) {
            job_ids.insert(job_id);
        }
    }

    job_ids.into_iter().collect()
}

pub(crate) fn application_ids_by_job_id(events: &[UserEventRecord]) -> BTreeMap<String, String> {
    let mut ordered_events = events
        .iter()
        .filter(|event| matches!(event.event_type, UserEventType::ApplicationCreated))
        .filter_map(|event| {
            let job_id = normalized_job_id(event.job_id.as_deref())?;
            let application_id = application_id_from_payload(event.payload_json.as_ref())?;
            Some((job_id, application_id, &event.created_at, &event.id))
        })
        .collect::<Vec<_>>();
    ordered_events.sort_by(
        |(left_job_id, _, left_created_at, left_id), (right_job_id, _, right_created_at, right_id)| {
            left_job_id
                .cmp(right_job_id)
                .then_with(|| left_created_at.cmp(right_created_at))
                .then_with(|| left_id.cmp(right_id))
        },
    );

    let mut application_ids = BTreeMap::new();
    for (job_id, application_id, _, _) in ordered_events {
        application_ids.insert(job_id, application_id);
    }

    application_ids
}

fn search_profile_from_analysis(analysis: &ProfileAnalysis) -> SearchProfile {
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

fn normalized_job_id(job_id: Option<&str>) -> Option<String> {
    job_id
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

pub(crate) fn event_signals_by_job_id(events: &[UserEventRecord]) -> BTreeMap<String, EventSignals> {
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
        .filter(|t| t.is_negative())
        .map(|t| t.as_str().to_string())
        .collect();
    let positive_tags: Vec<String> = feedback
        .tags
        .iter()
        .filter(|t| !t.is_negative())
        .map(|t| t.as_str().to_string())
        .collect();
    let has_salary_rejection = rejection_tags.contains(&"salary_too_low".to_string());
    let has_remote_rejection = rejection_tags.contains(&"not_remote".to_string());
    let has_tech_rejection = rejection_tags.contains(&"bad_tech_stack".to_string());

    let salary_signal = feedback.salary_signal.map(|s| s.as_str().to_string());
    let salary_below_expectation = feedback
        .salary_signal
        .is_some_and(|s| matches!(s, crate::domain::feedback::model::SalaryFeedbackSignal::BelowExpectation));

    let work_mode_deal_breaker = feedback
        .work_mode_signal
        .is_some_and(|s| matches!(s, crate::domain::feedback::model::WorkModeFeedbackSignal::DealBreaker));

    let legitimacy_suspicious = feedback
        .legitimacy_signal
        .is_some_and(|s| matches!(s, crate::domain::feedback::model::LegitimacySignal::Suspicious));
    let legitimacy_spam = feedback
        .legitimacy_signal
        .is_some_and(|s| matches!(s, crate::domain::feedback::model::LegitimacySignal::Spam));
    let outcome = application
        .and_then(|record| record.outcome)
        .map(ApplicationOutcome::as_str)
        .map(str::to_string);
    let reached_interview = application.is_some_and(application_reached_interview);
    let received_offer = application.is_some_and(|record| {
        matches!(record.outcome, Some(ApplicationOutcome::OfferReceived))
    });
    let was_rejected = application.is_some_and(|record| {
        matches!(record.outcome, Some(ApplicationOutcome::Rejected))
    });
    let was_ghosted = application.is_some_and(|record| {
        matches!(record.outcome, Some(ApplicationOutcome::Ghosted))
    });

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
        legitimacy_suspicious,
        legitimacy_spam,
    }
}

fn resolve_label_observed_at(
    signals: &OutcomeSignals,
    event_signals: &EventSignals,
    application: Option<&Application>,
    feedback_updated_at: Option<&str>,
) -> Option<String> {
    if signals.applied {
        if let Some(record) = application {
            if let Some(outcome_date) = record.outcome_date.as_ref() {
                return Some(outcome_date.clone());
            }
            return Some(record.updated_at.clone());
        }
    }

    if let Some(created_at) = event_signals.latest_event_at.as_ref() {
        return Some(created_at.clone());
    }

    feedback_updated_at.map(str::to_string)
}

fn assign_label(signals: &OutcomeSignals) -> Option<OutcomeLabelAssignment> {
    // Legitimacy spam/suspicious forces a negative regardless of other signals.
    if signals.legitimacy_spam || signals.legitimacy_suspicious {
        let mut reasons = vec!["dismissed".to_string(), "suspicious_posting".to_string()];
        if signals.legitimacy_spam {
            reasons.push("spam".to_string());
        }
        return Some(OutcomeLabelAssignment {
            label: OutcomeLabel::Negative,
            label_score: 0,
            reasons,
        });
    }

    // Work-mode deal-breaker forces negative regardless of save.
    if signals.work_mode_deal_breaker && !signals.applied {
        let mut reasons = vec!["dismissed".to_string(), "work_mode_deal_breaker".to_string()];
        if signals.dismissed {
            if signals.bad_fit {
                reasons.push("bad_fit".to_string());
            }
            if signals.hidden {
                reasons.push("hidden".to_string());
            }
        }
        return Some(OutcomeLabelAssignment {
            label: OutcomeLabel::Negative,
            label_score: 0,
            reasons,
        });
    }

    if signals.applied {
        let mut reasons = vec!["applied".to_string()];
        if signals.received_offer {
            reasons.push("offer_received".to_string());
        } else if signals.reached_interview {
            reasons.push("reached_interview".to_string());
        } else if signals.was_rejected {
            reasons.push("outcome_rejected".to_string());
        } else if signals.was_ghosted {
            reasons.push("outcome_ghosted".to_string());
        }
        return Some(OutcomeLabelAssignment {
            label: OutcomeLabel::Positive,
            label_score: 2,
            reasons,
        });
    }

    if signals.dismissed {
        let mut reasons = vec!["dismissed".to_string()];
        if signals.bad_fit {
            reasons.push("bad_fit".to_string());
        }
        if signals.hidden {
            reasons.push("hidden".to_string());
        }
        if signals.has_salary_rejection {
            reasons.push("salary_too_low".to_string());
        }
        if signals.salary_below_expectation && !reasons.contains(&"salary_too_low".to_string()) {
            reasons.push("salary_too_low".to_string());
        }

        return Some(OutcomeLabelAssignment {
            label: OutcomeLabel::Negative,
            label_score: 0,
            reasons,
        });
    }

    if signals.saved {
        let mut reasons = vec!["saved".to_string()];
        if signals.interest_rating == Some(2) {
            reasons.push("love_it".to_string());
        }
        return Some(OutcomeLabelAssignment {
            label: OutcomeLabel::Medium,
            label_score: 1,
            reasons,
        });
    }

    if signals.viewed {
        let mut reasons = vec!["viewed".to_string()];
        if signals.returned_count >= 2 {
            reasons.push("high_engagement".to_string());
        }
        return Some(OutcomeLabelAssignment {
            label: OutcomeLabel::Medium,
            label_score: 1,
            reasons,
        });
    }

    None
}

fn ranking_features(
    fit: JobFit,
    behavior_score_delta: i16,
    behavior_score: u8,
    behavior_reasons: Vec<String>,
    learned_reranker_score_delta: i16,
    learned_reranker_score: u8,
    learned_reasons: Vec<String>,
) -> OutcomeRankingFeatures {
    let matched_roles = fit
        .matched_roles
        .into_iter()
        .map(|role| role.to_string())
        .collect::<Vec<_>>();
    let matched_role_count = matched_roles.len();
    let matched_skill_count = fit.matched_skills.len();
    let matched_keyword_count = fit.matched_keywords.len();

    OutcomeRankingFeatures {
        deterministic_score: fit.score,
        behavior_score_delta,
        behavior_score,
        learned_reranker_score_delta,
        learned_reranker_score,
        matched_roles,
        matched_skills: fit.matched_skills,
        matched_keywords: fit.matched_keywords,
        matched_role_count,
        matched_skill_count,
        matched_keyword_count,
        fit_reasons: fit.reasons,
        behavior_reasons,
        learned_reasons,
    }
}

fn application_id_from_payload(payload: Option<&Value>) -> Option<String> {
    payload
        .and_then(|value| value.get("application_id"))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
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

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use crate::domain::application::model::{Application, ApplicationOutcome};
    use crate::domain::feedback::model::JobFeedbackState;
    use crate::domain::job::model::{Job, JobLifecycleStage, JobSourceVariant, JobView};
    use crate::domain::profile::model::{Profile, ProfileAnalysis};
    use crate::domain::role::RoleId;
    use crate::domain::user_event::model::{UserEventRecord, UserEventType};
    use crate::services::behavior::BehaviorService;
    use crate::services::funnel::FunnelService;
    use crate::services::search_ranking::SearchRankingService;

    use super::{OutcomeDatasetService, OutcomeLabel, outcome_job_ids};

    fn profile() -> Profile {
        Profile {
            id: "profile-1".to_string(),
            name: "Jane Doe".to_string(),
            email: "jane@example.com".to_string(),
            location: Some("Kyiv".to_string()),
            raw_text: "Senior backend engineer with Rust and Postgres".to_string(),
            analysis: Some(ProfileAnalysis {
                summary: "Senior backend engineer".to_string(),
                primary_role: RoleId::BackendEngineer,
                seniority: "senior".to_string(),
                skills: vec!["Rust".to_string(), "Postgres".to_string()],
                keywords: vec!["backend".to_string(), "platform".to_string()],
            }),
            years_of_experience: None,
            salary_min: None,
            salary_max: None,
            salary_currency: "USD".to_string(),
            languages: vec![],
            preferred_work_mode: None,
            search_preferences: None,
            created_at: "2026-04-14T00:00:00Z".to_string(),
            updated_at: "2026-04-14T00:00:00Z".to_string(),
            skills_updated_at: None,
        }
    }

    fn job_view(id: &str, title: &str, source: &str) -> JobView {
        JobView {
            job: Job {
                id: id.to_string(),
                title: title.to_string(),
                company_name: "NovaLedger".to_string(),
                location: None,
                remote_type: Some("remote".to_string()),
                seniority: Some("senior".to_string()),
                description_text: "Build backend Rust services on Postgres".to_string(),
                salary_min: None,
                salary_max: None,
                salary_currency: None,
                posted_at: Some("2026-04-14T00:00:00Z".to_string()),
                last_seen_at: "2026-04-15T00:00:00Z".to_string(),
                is_active: true,
            },
            first_seen_at: "2026-04-14T00:00:00Z".to_string(),
            inactivated_at: None,
            reactivated_at: None,
            lifecycle_stage: JobLifecycleStage::Active,
            primary_variant: Some(JobSourceVariant {
                source: source.to_string(),
                source_job_id: format!("{source}-{id}"),
                source_url: format!("https://example.com/{id}"),
                raw_payload: None,
                fetched_at: "2026-04-14T00:00:00Z".to_string(),
                last_seen_at: "2026-04-15T00:00:00Z".to_string(),
                is_active: true,
                inactivated_at: None,
            }),
        }
    }

    fn event(id: &str, job_id: &str, event_type: UserEventType) -> UserEventRecord {
        UserEventRecord {
            id: id.to_string(),
            profile_id: "profile-1".to_string(),
            event_type,
            job_id: Some(job_id.to_string()),
            company_name: Some("NovaLedger".to_string()),
            source: Some("djinni".to_string()),
            role_family: Some("engineering".to_string()),
            payload_json: None,
            created_at: "2026-04-15T00:00:00Z".to_string(),
        }
    }

    #[test]
    fn label_assignment_is_deterministic_and_prioritized() {
        let events = vec![
            event("evt-1", "job-positive", UserEventType::JobSaved),
            event("evt-2", "job-positive", UserEventType::JobBadFit),
            event("evt-3", "job-positive", UserEventType::ApplicationCreated),
            event("evt-4", "job-negative", UserEventType::JobSaved),
            event("evt-5", "job-negative", UserEventType::JobBadFit),
            event("evt-6", "job-medium", UserEventType::JobSaved),
            event("evt-7", "job-viewed", UserEventType::JobOpened),
        ];
        let behavior = BehaviorService::new().build_aggregates(events.iter());
        let funnel = FunnelService::new().build_aggregates(events.iter());
        let dataset = OutcomeDatasetService::new()
            .build(
                &profile(),
                &events,
                vec![
                    (
                        job_view("job-positive", "Senior Backend Engineer", "djinni"),
                        JobFeedbackState::default(),
                    ),
                    (
                        job_view("job-negative", "Senior Backend Engineer", "djinni"),
                        JobFeedbackState::default(),
                    ),
                    (
                        job_view("job-medium", "Senior Backend Engineer", "djinni"),
                        JobFeedbackState::default(),
                    ),
                    (
                        job_view("job-viewed", "Senior Backend Engineer", "djinni"),
                        JobFeedbackState::default(),
                    ),
                ],
                &BTreeMap::new(),
                &BTreeMap::new(),
                &SearchRankingService::new(),
                &behavior,
                &funnel,
            )
            .expect("dataset should build");

        let labels = dataset
            .examples
            .iter()
            .map(|example| (example.job_id.as_str(), example.label, example.label_score))
            .collect::<Vec<_>>();

        assert_eq!(
            labels,
            vec![
                ("job-medium", OutcomeLabel::Medium, 1),
                ("job-negative", OutcomeLabel::Negative, 0),
                ("job-positive", OutcomeLabel::Positive, 2),
                ("job-viewed", OutcomeLabel::Medium, 1),
            ]
        );

        let positive = dataset
            .examples
            .iter()
            .find(|example| example.job_id == "job-positive")
            .expect("positive example should exist");
        assert_eq!(positive.label_reasons, vec!["applied"]);
        assert!(positive.signals.dismissed);
    }

    #[test]
    fn reversal_events_clear_saved_and_dismissed_states_before_labeling() {
        let events = vec![
            event("evt-2", "job-viewed", UserEventType::JobSaved),
            event("evt-1", "job-viewed", UserEventType::JobOpened),
            event("evt-3", "job-viewed", UserEventType::JobUnsaved),
            event("evt-4", "job-cleared", UserEventType::JobHidden),
            event("evt-5", "job-cleared", UserEventType::JobUnhidden),
        ];
        let behavior = BehaviorService::new().build_aggregates(events.iter());
        let funnel = FunnelService::new().build_aggregates(events.iter());
        let dataset = OutcomeDatasetService::new()
            .build(
                &profile(),
                &events,
                vec![
                    (
                        job_view("job-viewed", "Senior Backend Engineer", "djinni"),
                        JobFeedbackState::default(),
                    ),
                    (
                        job_view("job-cleared", "Senior Backend Engineer", "djinni"),
                        JobFeedbackState::default(),
                    ),
                ],
                &BTreeMap::new(),
                &BTreeMap::new(),
                &SearchRankingService::new(),
                &behavior,
                &funnel,
            )
            .expect("dataset should build");

        assert_eq!(dataset.examples.len(), 1);
        assert_eq!(dataset.examples[0].job_id, "job-viewed");
        assert_eq!(dataset.examples[0].label_reasons, vec!["viewed"]);
        assert!(dataset.examples[0].signals.viewed);
        assert!(!dataset.examples[0].signals.saved);
        assert_eq!(dataset.examples[0].signals.saved_event_count, 1);
        assert_eq!(dataset.examples[0].signals.viewed_event_count, 1);
    }

    #[test]
    fn explicit_feedback_flags_drive_labels_without_matching_events() {
        let events = vec![event("evt-1", "job-explicit", UserEventType::JobOpened)];
        let behavior = BehaviorService::new().build_aggregates(events.iter());
        let funnel = FunnelService::new().build_aggregates(events.iter());
        let dataset = OutcomeDatasetService::new()
            .build(
                &profile(),
                &events,
                vec![(
                    job_view("job-explicit", "Senior Backend Engineer", "djinni"),
                    JobFeedbackState {
                        saved: false,
                        hidden: true,
                        bad_fit: false,
                        company_status: None,
                        salary_signal: None,
                        interest_rating: None,
                        work_mode_signal: None,
                        legitimacy_signal: None,
                        tags: Vec::new(),
                    },
                )],
                &BTreeMap::new(),
                &BTreeMap::new(),
                &SearchRankingService::new(),
                &behavior,
                &funnel,
            )
            .expect("dataset should build");

        assert_eq!(dataset.examples.len(), 1);
        assert_eq!(dataset.examples[0].label, OutcomeLabel::Negative);
        assert_eq!(
            dataset.examples[0].label_reasons,
            vec!["dismissed", "hidden"]
        );
        assert!(dataset.examples[0].signals.explicit_feedback);
        assert!(dataset.examples[0].signals.explicit_hidden);
        assert!(dataset.examples[0].signals.dismissed);
    }

    #[test]
    fn outcome_job_ids_include_viewed_events_plus_feedback_job_ids() {
        let events = vec![
            event("evt-1", "job-saved", UserEventType::JobSaved),
            event("evt-2", "job-opened", UserEventType::JobOpened),
            event("evt-3", "job-applied", UserEventType::ApplicationCreated),
        ];

        let job_ids = outcome_job_ids(&events, &["job-feedback".to_string()]);

        assert_eq!(
            job_ids,
            vec!["job-applied", "job-feedback", "job-opened", "job-saved"]
        );
    }

    #[test]
    fn empty_labelable_inputs_return_empty_dataset() {
        let events = Vec::new();
        let behavior = BehaviorService::new().build_aggregates(events.iter());
        let funnel = FunnelService::new().build_aggregates(events.iter());
        let dataset = OutcomeDatasetService::new()
            .build(
                &profile(),
                &events,
                Vec::new(),
                &BTreeMap::new(),
                &BTreeMap::new(),
                &SearchRankingService::new(),
                &behavior,
                &funnel,
            )
            .expect("empty dataset should build");

        assert!(dataset.examples.is_empty());
    }

    #[test]
    fn application_outcomes_enrich_signals_and_label_timestamp() {
        let events = vec![event("evt-1", "job-positive", UserEventType::ApplicationCreated)];
        let behavior = BehaviorService::new().build_aggregates(events.iter());
        let funnel = FunnelService::new().build_aggregates(events.iter());
        let dataset = OutcomeDatasetService::new()
            .build(
                &profile(),
                &events,
                vec![(
                    job_view("job-positive", "Senior Backend Engineer", "djinni"),
                    JobFeedbackState::default(),
                )],
                &BTreeMap::from([(
                    "job-positive".to_string(),
                    Application {
                        id: "app-1".to_string(),
                        job_id: "job-positive".to_string(),
                        resume_id: None,
                        status: "offer".to_string(),
                        applied_at: Some("2026-04-15T00:00:00Z".to_string()),
                        due_date: None,
                        outcome: Some(ApplicationOutcome::OfferReceived),
                        outcome_date: Some("2026-04-20T00:00:00Z".to_string()),
                        rejection_stage: None,
                        updated_at: "2026-04-20T00:00:00Z".to_string(),
                    },
                )]),
                &BTreeMap::new(),
                &SearchRankingService::new(),
                &behavior,
                &funnel,
            )
            .expect("dataset should build");

        assert_eq!(dataset.examples.len(), 1);
        assert_eq!(
            dataset.examples[0].label_observed_at.as_deref(),
            Some("2026-04-20T00:00:00Z")
        );
        assert_eq!(dataset.examples[0].signals.outcome.as_deref(), Some("offer_received"));
        assert!(dataset.examples[0].signals.received_offer);
        assert!(dataset.examples[0].signals.reached_interview);
    }
}
