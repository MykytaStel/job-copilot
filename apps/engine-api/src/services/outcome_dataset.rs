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

pub const OUTCOME_LABEL_POLICY_VERSION: &str = "outcome_label_v1";

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
    pub saved: bool,
    pub hidden: bool,
    pub bad_fit: bool,
    pub application_created: bool,
    pub saved_count: usize,
    pub hidden_count: usize,
    pub bad_fit_count: usize,
    pub application_created_count: usize,
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
        let event_counts_by_job_id = event_counts_by_job_id(events);
        let behavior_service = BehaviorService::new();
        let learned_reranker = LearnedRerankerService::new();
        let mut examples = Vec::new();

        for (job, feedback) in jobs {
            let job_id = job.job.id.clone();
            let event_counts = event_counts_by_job_id
                .get(&job_id)
                .cloned()
                .unwrap_or_default();
            let signals = OutcomeSignals {
                saved: feedback.saved || event_counts.saved_count > 0,
                hidden: feedback.hidden || event_counts.hidden_count > 0,
                bad_fit: feedback.bad_fit || event_counts.bad_fit_count > 0,
                application_created: event_counts.application_created_count > 0,
                saved_count: event_counts.saved_count,
                hidden_count: event_counts.hidden_count,
                bad_fit_count: event_counts.bad_fit_count,
                application_created_count: event_counts.application_created_count,
            };
            let Some((label, label_reasons)) = assign_label(&signals) else {
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
                label,
                label_score: label.score(),
                label_reasons,
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

    pub fn score(self) -> u8 {
        match self {
            Self::Positive => 2,
            Self::Medium => 1,
            Self::Negative => 0,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct EventCounts {
    saved_count: usize,
    hidden_count: usize,
    bad_fit_count: usize,
    application_created_count: usize,
}

pub fn outcome_job_ids(events: &[UserEventRecord], feedback_job_ids: &[String]) -> Vec<String> {
    let mut job_ids = BTreeSet::new();

    for event in events {
        if !matches!(
            event.event_type,
            UserEventType::JobSaved
                | UserEventType::JobHidden
                | UserEventType::JobBadFit
                | UserEventType::ApplicationCreated
        ) {
            continue;
        }

        if let Some(job_id) = event.job_id.as_deref() {
            let job_id = job_id.trim();
            if !job_id.is_empty() {
                job_ids.insert(job_id.to_string());
            }
        }
    }

    for job_id in feedback_job_ids {
        let job_id = job_id.trim();
        if !job_id.is_empty() {
            job_ids.insert(job_id.to_string());
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

fn event_counts_by_job_id(events: &[UserEventRecord]) -> BTreeMap<String, EventCounts> {
    let mut counts_by_job_id = BTreeMap::<String, EventCounts>::new();

    for event in events {
        let Some(job_id) = event
            .job_id
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
        else {
            continue;
        };
        let counts = counts_by_job_id.entry(job_id.to_string()).or_default();

        match event.event_type {
            UserEventType::JobSaved => counts.saved_count += 1,
            UserEventType::JobHidden => counts.hidden_count += 1,
            UserEventType::JobBadFit => counts.bad_fit_count += 1,
            UserEventType::ApplicationCreated => counts.application_created_count += 1,
            _ => {}
        }
    }

    counts_by_job_id
}

fn assign_label(signals: &OutcomeSignals) -> Option<(OutcomeLabel, Vec<String>)> {
    if signals.application_created {
        return Some((
            OutcomeLabel::Positive,
            vec!["application_created".to_string()],
        ));
    }

    if signals.bad_fit || signals.hidden {
        let mut reasons = Vec::new();
        if signals.bad_fit {
            reasons.push("bad_fit".to_string());
        }
        if signals.hidden {
            reasons.push("hidden".to_string());
        }

        return Some((OutcomeLabel::Negative, reasons));
    }

    if signals.saved {
        return Some((OutcomeLabel::Medium, vec!["saved".to_string()]));
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
            salary_min_usd: None,
            salary_max_usd: None,
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
            event("evt-2", "job-positive", UserEventType::ApplicationCreated),
            event("evt-3", "job-negative", UserEventType::JobSaved),
            event("evt-4", "job-negative", UserEventType::JobBadFit),
            event("evt-5", "job-medium", UserEventType::JobSaved),
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
            ]
        );
    }

    #[test]
    fn outcome_job_ids_are_profile_events_plus_feedback_job_ids() {
        let events = vec![
            event("evt-1", "job-saved", UserEventType::JobSaved),
            event("evt-2", "job-opened", UserEventType::JobOpened),
            event("evt-3", "job-applied", UserEventType::ApplicationCreated),
        ];

        let job_ids = outcome_job_ids(&events, &["job-feedback".to_string()]);

        assert_eq!(job_ids, vec!["job-applied", "job-feedback", "job-saved"]);
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
