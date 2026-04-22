use std::collections::HashMap;

use tracing::warn;

use crate::domain::application::model::Application;
use crate::api::error::ApiError;
use crate::domain::feedback::model::{CompanyFeedbackStatus, JobFeedbackState};
use crate::domain::matching::RerankerMode;
use crate::services::behavior::{BehaviorService, ProfileBehaviorAggregates};
use crate::services::funnel::{FunnelService, ProfileFunnelAggregates};
use crate::services::learned_reranker::LearnedRerankerService;
use crate::services::outcome_dataset::{
    EventSignals, application_ids_by_job_id, normalize_signals,
};
use crate::services::salary::{SearchSalaryExpectation, score_search_salary};
use crate::services::search_ranking::RankedJob;
use crate::services::trained_reranker::TrainedRerankerFeatures;
use crate::state::AppState;

use super::sort_ranked_jobs;

/// Bonus applied to score when the job's company is whitelisted for the profile.
const WHITELIST_SCORE_BONUS: u8 = 10;

/// Penalty subtracted from score when the exact job is marked as bad fit.
const BAD_FIT_SCORE_PENALTY: u8 = 30;

pub(crate) struct SearchLearningAggregates {
    pub(crate) behavior: ProfileBehaviorAggregates,
    pub(crate) funnel: ProfileFunnelAggregates,
    pub(crate) events: Vec<crate::domain::user_event::model::UserEventRecord>,
    pub(crate) applications_by_job_id: HashMap<String, Application>,
}

/// Adjust fit scores based on explicit job feedback, then re-sort by adjusted score.
pub(crate) fn apply_feedback_scoring(
    mut ranked_jobs: Vec<RankedJob>,
    feedback_by_job_id: &HashMap<String, JobFeedbackState>,
) -> Vec<RankedJob> {
    for ranked in &mut ranked_jobs {
        let Some(feedback) = feedback_by_job_id.get(&ranked.job.job.id) else {
            continue;
        };

        if feedback.company_status == Some(CompanyFeedbackStatus::Whitelist) {
            ranked.fit.apply_matching_adjustment(
                i16::from(WHITELIST_SCORE_BONUS),
                Some("Company is whitelisted for this profile".to_string()),
            );
        }

        if feedback.bad_fit {
            ranked.fit.add_penalty(
                "bad_fit_feedback",
                -i16::from(BAD_FIT_SCORE_PENALTY),
                "Job was previously marked as bad fit".to_string(),
            );
        }
    }

    sort_ranked_jobs(&mut ranked_jobs);
    ranked_jobs
}

pub(crate) async fn load_learning_aggregates(
    state: &AppState,
    profile_id: Option<&str>,
) -> Option<SearchLearningAggregates> {
    let profile_id = profile_id?;
    let events = match state.user_events_service.list_by_profile(profile_id).await {
        Ok(events) => events,
        Err(error) => {
            warn!(
                error = %error,
                profile_id,
                "failed to load behavior aggregates; continuing without personalization"
            );
            return None;
        }
    };

    Some(SearchLearningAggregates {
        behavior: BehaviorService::new().build_aggregates(events.iter()),
        funnel: FunnelService::new().build_aggregates(events.iter()),
        applications_by_job_id: load_profile_applications_by_job_id(state, &events).await,
        events,
    })
}

pub(crate) async fn load_search_salary_expectation(
    state: &AppState,
    profile_id: Option<&str>,
) -> Result<Option<SearchSalaryExpectation>, ApiError> {
    let Some(profile_id) = profile_id else {
        return Ok(None);
    };
    let profile = state
        .profile_records
        .get_by_id(profile_id)
        .await
        .map_err(|error| ApiError::from_repository(error, "search_run_failed"))?;

    Ok(profile.map(|profile| SearchSalaryExpectation {
        min: profile.salary_min,
        max: profile.salary_max,
        currency: profile.salary_currency,
    }))
}

pub(crate) fn apply_salary_scoring(
    mut ranked_jobs: Vec<RankedJob>,
    expectation: Option<&SearchSalaryExpectation>,
) -> Vec<RankedJob> {
    for ranked in &mut ranked_jobs {
        let adjustment = score_search_salary(expectation, &ranked.job.job);
        ranked
            .fit
            .apply_salary_score(adjustment.score_delta, adjustment.reason);
    }

    sort_ranked_jobs(&mut ranked_jobs);
    ranked_jobs
}

pub(crate) fn apply_behavior_scoring(
    state: &AppState,
    mut ranked_jobs: Vec<RankedJob>,
    aggregates: &ProfileBehaviorAggregates,
) -> Vec<RankedJob> {
    let behavior_service = BehaviorService::new();

    for ranked in &mut ranked_jobs {
        let source = ranked
            .job
            .primary_variant
            .as_ref()
            .map(|variant| variant.source.as_str());
        let role_family = state.search_ranking.infer_role_family(&ranked.job);
        let adjustment = behavior_service.score_job(aggregates, source, role_family.as_deref());

        if adjustment.score_delta == 0 {
            continue;
        }

        ranked
            .fit
            .apply_matching_adjustment(adjustment.score_delta, None);
        ranked.fit.reasons.extend(adjustment.reasons);
    }

    sort_ranked_jobs(&mut ranked_jobs);
    ranked_jobs
}

pub(crate) fn apply_learned_reranking(
    state: &AppState,
    mut ranked_jobs: Vec<RankedJob>,
    behavior: &ProfileBehaviorAggregates,
    funnel: &ProfileFunnelAggregates,
    feedback_by_job_id: &HashMap<String, JobFeedbackState>,
    deterministic_score_by_job_id: &HashMap<String, u8>,
) -> (Vec<RankedJob>, usize) {
    let learned_reranker = LearnedRerankerService::new();
    let mut adjusted_count = 0usize;

    for ranked in &mut ranked_jobs {
        let source = ranked
            .job
            .primary_variant
            .as_ref()
            .map(|variant| variant.source.as_str());
        let role_family = state.search_ranking.infer_role_family(&ranked.job);
        let feedback = feedback_by_job_id
            .get(&ranked.job.job.id)
            .cloned()
            .unwrap_or_default();
        let deterministic_score = deterministic_score_by_job_id
            .get(&ranked.job.job.id)
            .copied()
            .unwrap_or(ranked.fit.score);
        let learned_score = learned_reranker.score_job(
            deterministic_score,
            source,
            role_family.as_deref(),
            behavior,
            funnel,
            &feedback,
        );

        if learned_score.score_delta == 0 {
            ranked
                .fit
                .apply_reranker_score(0, RerankerMode::Learned, Vec::new());
            continue;
        }

        ranked.fit.apply_reranker_score(
            learned_score.score_delta,
            RerankerMode::Learned,
            learned_score.reasons,
        );
        adjusted_count += 1;
    }

    sort_ranked_jobs(&mut ranked_jobs);
    (ranked_jobs, adjusted_count)
}

pub(crate) fn apply_trained_reranking(
    state: &AppState,
    mut ranked_jobs: Vec<RankedJob>,
    deterministic_score_by_job_id: &HashMap<String, u8>,
    behavior_score_by_job_id: &HashMap<String, u8>,
    feedback_by_job_id: &HashMap<String, JobFeedbackState>,
    event_signals_by_job_id: &HashMap<String, EventSignals>,
    applications_by_job_id: &HashMap<String, Application>,
) -> (Vec<RankedJob>, usize) {
    let Some(model) = state.trained_reranker_model.as_ref() else {
        return (ranked_jobs, 0);
    };
    let mut adjusted_count = 0usize;

    for ranked in &mut ranked_jobs {
        let job_id = ranked.job.job.id.as_str();
        let deterministic_score = deterministic_score_by_job_id
            .get(job_id)
            .copied()
            .unwrap_or(ranked.fit.score);
        let behavior_score = behavior_score_by_job_id
            .get(job_id)
            .copied()
            .unwrap_or(deterministic_score);
        let learned_reranker_score = ranked.fit.score;
        let source_present = ranked.job.primary_variant.is_some();
        let role_family_present = state
            .search_ranking
            .infer_role_family(&ranked.job)
            .is_some();
        let feedback = feedback_by_job_id.get(job_id).cloned().unwrap_or_default();
        let default_event_signals = EventSignals::default();
        let event_signals = event_signals_by_job_id
            .get(job_id)
            .unwrap_or(&default_event_signals);
        let signals = normalize_signals(&feedback, event_signals, applications_by_job_id.get(job_id));
        let rating = signals.interest_rating.unwrap_or(0);
        let quick_apply = signals.time_to_apply_days.is_some_and(|days| days <= 3);
        let delayed_apply = signals.time_to_apply_days.is_some_and(|days| days > 14);
        let features = TrainedRerankerFeatures {
            deterministic_score,
            behavior_score_delta: i16::from(behavior_score) - i16::from(deterministic_score),
            behavior_score,
            learned_reranker_score_delta: i16::from(learned_reranker_score)
                - i16::from(behavior_score),
            learned_reranker_score,
            matched_role_count: ranked.fit.matched_roles.len(),
            matched_skill_count: ranked.fit.matched_skills.len(),
            matched_keyword_count: ranked.fit.matched_keywords.len(),
            source_present,
            role_family_present,
            outcome_received_offer: signals.received_offer,
            outcome_reached_interview: signals.reached_interview,
            outcome_rejected: signals.was_rejected,
            has_salary_rejection: signals.has_salary_rejection,
            has_remote_rejection: signals.has_remote_rejection,
            has_tech_rejection: signals.has_tech_rejection,
            interest_rating_positive: f64::from(rating.max(0).clamp(0, 2)) / 2.0,
            interest_rating_negative: f64::from((-rating).max(0).clamp(0, 2)) / 2.0,
            work_mode_deal_breaker: signals.work_mode_deal_breaker,
            scrolled_to_bottom: signals.scrolled_to_bottom,
            returned_count: signals.returned_count,
            quick_apply,
            delayed_apply,
            legitimacy_suspicious: signals.legitimacy_suspicious,
        };
        let score = model.score(&features);

        if score.score_delta == 0 {
            ranked
                .fit
                .apply_reranker_score(0, RerankerMode::Trained, Vec::new());
            continue;
        }

        ranked
            .fit
            .apply_reranker_score(score.score_delta, RerankerMode::Trained, score.reasons);
        adjusted_count += 1;
    }

    sort_ranked_jobs(&mut ranked_jobs);
    (ranked_jobs, adjusted_count)
}

async fn load_profile_applications_by_job_id(
    state: &AppState,
    events: &[crate::domain::user_event::model::UserEventRecord],
) -> HashMap<String, Application> {
    let mut applications_by_job_id = HashMap::new();

    for (job_id, application_id) in application_ids_by_job_id(events) {
        match state.applications_service.get_by_id(&application_id).await {
            Ok(Some(application)) => {
                applications_by_job_id.insert(job_id, application);
            }
            Ok(None) => {}
            Err(error) => {
                warn!(
                    error = %error,
                    application_id,
                    "failed to load application outcome for trained reranker; continuing without it"
                );
            }
        }
    }

    applications_by_job_id
}

pub(crate) fn score_by_job_id(ranked_jobs: &[RankedJob]) -> HashMap<String, u8> {
    ranked_jobs
        .iter()
        .map(|ranked| (ranked.job.job.id.clone(), ranked.fit.score))
        .collect()
}
