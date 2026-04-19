use std::collections::{BTreeMap, BTreeSet};

use crate::domain::feedback::model::JobFeedbackState;
use crate::domain::job::model::JobView;
use crate::domain::matching::JobFit;
use crate::domain::profile::model::{Profile, ProfileAnalysis};
use crate::domain::search::profile::{SearchProfile, SearchRoleCandidate};
use crate::domain::user_event::model::{UserEventRecord, UserEventType};
use crate::services::behavior::{BehaviorService, ProfileBehaviorAggregates};
use crate::services::funnel::ProfileFunnelAggregates;
use crate::services::learned_reranker::LearnedRerankerService;
use crate::services::matching::SearchMatchingService;

pub const OUTCOME_LABEL_POLICY_VERSION: &str = "outcome_label_v2";

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
        matching_service: &SearchMatchingService,
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
            let signals = normalize_signals(&feedback, event_signals);
            let Some(label_assignment) = assign_label(&signals) else {
                continue;
            };

            let fit = matching_service.score_job(&search_profile, &job);
            let source = job
                .primary_variant
                .as_ref()
                .map(|variant| variant.source.clone());
            let role_family = matching_service.infer_role_family(&job);
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
struct EventSignals {
    viewed: bool,
    saved: bool,
    hidden: bool,
    bad_fit: bool,
    applied: bool,
    viewed_event_count: usize,
    saved_event_count: usize,
    applied_event_count: usize,
    dismissed_event_count: usize,
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

fn event_signals_by_job_id(events: &[UserEventRecord]) -> BTreeMap<String, EventSignals> {
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
            _ => {}
        }
    }

    signals_by_job_id
}

fn normalize_signals(feedback: &JobFeedbackState, event_signals: EventSignals) -> OutcomeSignals {
    let saved = feedback.saved || event_signals.saved;
    let hidden = feedback.hidden || event_signals.hidden;
    let bad_fit = feedback.bad_fit || event_signals.bad_fit;

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
    }
}

fn assign_label(signals: &OutcomeSignals) -> Option<OutcomeLabelAssignment> {
    if signals.applied {
        return Some(OutcomeLabelAssignment {
            label: OutcomeLabel::Positive,
            label_score: 2,
            reasons: vec!["applied".to_string()],
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

        return Some(OutcomeLabelAssignment {
            label: OutcomeLabel::Negative,
            label_score: 0,
            reasons,
        });
    }

    if signals.saved {
        return Some(OutcomeLabelAssignment {
            label: OutcomeLabel::Medium,
            label_score: 1,
            reasons: vec!["saved".to_string()],
        });
    }

    if signals.viewed {
        return Some(OutcomeLabelAssignment {
            label: OutcomeLabel::Medium,
            label_score: 1,
            reasons: vec!["viewed".to_string()],
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

#[cfg(test)]
mod tests {
    use crate::domain::feedback::model::JobFeedbackState;
    use crate::domain::job::model::{Job, JobLifecycleStage, JobSourceVariant, JobView};
    use crate::domain::profile::model::{Profile, ProfileAnalysis};
    use crate::domain::role::RoleId;
    use crate::domain::user_event::model::{UserEventRecord, UserEventType};
    use crate::services::behavior::BehaviorService;
    use crate::services::funnel::FunnelService;
    use crate::services::matching::SearchMatchingService;

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
                &SearchMatchingService::new(),
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
                &SearchMatchingService::new(),
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
                    },
                )],
                &SearchMatchingService::new(),
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
                &SearchMatchingService::new(),
                &behavior,
                &funnel,
            )
            .expect("empty dataset should build");

        assert!(dataset.examples.is_empty());
    }
}
