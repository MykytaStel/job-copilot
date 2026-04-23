use std::collections::BTreeMap;

use crate::domain::application::model::Application;
use crate::domain::feedback::model::JobFeedbackState;
use crate::domain::job::model::JobView;
use crate::services::behavior::{BehaviorService, ProfileBehaviorAggregates};
use crate::services::funnel::ProfileFunnelAggregates;
use crate::services::learned_reranker::LearnedRerankerService;
use crate::services::search_ranking::SearchRankingService;

mod ids;
mod labeling;
mod ranking;
mod signals;
#[cfg(test)]
mod tests;
mod types;

pub(crate) use ids::application_ids_by_job_id;
pub use ids::outcome_job_ids;
use labeling::{assign_label, resolve_label_observed_at};
use ranking::ranking_features;
use signals::search_profile_from_analysis;
pub(crate) use signals::{EventSignals, event_signals_by_job_id, normalize_signals};
#[allow(unused_imports)]
pub use types::OutcomeLabel;
pub use types::{
    OutcomeDataset, OutcomeDatasetError, OutcomeExample, OutcomeRankingFeatures, OutcomeSignals,
};

pub const OUTCOME_LABEL_POLICY_VERSION: &str = "outcome_label_v3";

#[derive(Clone, Default)]
pub struct OutcomeDatasetService;

impl OutcomeDatasetService {
    pub fn new() -> Self {
        Self
    }

    pub fn build(
        &self,
        profile: &crate::domain::profile::model::Profile,
        events: &[crate::domain::user_event::model::UserEventRecord],
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
                feedback_updated_at_by_job_id
                    .get(&job_id)
                    .map(String::as_str),
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
