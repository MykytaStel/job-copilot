use std::collections::HashMap;

use crate::api::dto::search::{
    SearchRerankerComparisonInput, SearchRerankerComparisonItemResponse,
    SearchRerankerComparisonModeResponse, SearchRerankerComparisonResponse,
};
use crate::domain::feedback::model::JobFeedbackState;
use crate::domain::matching::RerankerMode;
use crate::services::outcome_dataset::event_signals_by_job_id;
use crate::services::search_ranking::RankedJob;
use crate::services::search_ranking::runtime::{
    ResolvedRerankerRuntime, resolve_reranker_runtime_comparison,
};
use crate::state::AppState;

use super::{SearchLearningAggregates, apply_learned_reranking, apply_trained_reranking};

pub(super) fn build_reranker_comparison(
    state: &AppState,
    comparison: &SearchRerankerComparisonInput,
    live_runtime: &ResolvedRerankerRuntime,
    baseline_ranked_jobs: &[RankedJob],
    _live_ranked_jobs: &[RankedJob],
    learning_aggregates: Option<&SearchLearningAggregates>,
    feedback_by_job_id: &HashMap<String, JobFeedbackState>,
    deterministic_score_by_job_id: &HashMap<String, u8>,
    behavior_score_by_job_id: &HashMap<String, u8>,
) -> SearchRerankerComparisonResponse {
    let runtime_comparison = resolve_reranker_runtime_comparison(
        state.learned_reranker_enabled,
        learning_aggregates.is_some(),
        &state.trained_reranker_availability,
    );
    let learned_ranked_jobs = apply_reranker_runtime_path(
        state,
        baseline_ranked_jobs.to_vec(),
        &runtime_comparison.learned,
        learning_aggregates,
        feedback_by_job_id,
        deterministic_score_by_job_id,
        behavior_score_by_job_id,
    );
    let trained_ranked_jobs = apply_reranker_runtime_path(
        state,
        baseline_ranked_jobs.to_vec(),
        &runtime_comparison.trained,
        learning_aggregates,
        feedback_by_job_id,
        deterministic_score_by_job_id,
        behavior_score_by_job_id,
    );

    SearchRerankerComparisonResponse {
        baseline_mode: runtime_comparison.baseline.active_mode.as_str().to_string(),
        active_mode: live_runtime.active_mode.as_str().to_string(),
        top_n: comparison.top_n,
        baseline_top: top_reranker_comparison_items(baseline_ranked_jobs, comparison.top_n),
        learned: build_reranker_comparison_mode(
            &runtime_comparison.learned,
            baseline_ranked_jobs,
            &learned_ranked_jobs,
            comparison.top_n,
        ),
        trained: build_reranker_comparison_mode(
            &runtime_comparison.trained,
            baseline_ranked_jobs,
            &trained_ranked_jobs,
            comparison.top_n,
        ),
    }
}

fn build_reranker_comparison_mode(
    runtime: &ResolvedRerankerRuntime,
    baseline_ranked_jobs: &[RankedJob],
    compared_ranked_jobs: &[RankedJob],
    top_n: usize,
) -> SearchRerankerComparisonModeResponse {
    SearchRerankerComparisonModeResponse {
        active_mode: runtime.active_mode.as_str().to_string(),
        would_differ_from_baseline: ranked_jobs_differ(baseline_ranked_jobs, compared_ranked_jobs),
        fallback_reason: runtime.fallback_reason.clone(),
        top: top_reranker_comparison_items(compared_ranked_jobs, top_n),
    }
}

fn apply_reranker_runtime_path(
    state: &AppState,
    mut ranked_jobs: Vec<RankedJob>,
    runtime: &ResolvedRerankerRuntime,
    learning_aggregates: Option<&SearchLearningAggregates>,
    feedback_by_job_id: &HashMap<String, JobFeedbackState>,
    deterministic_score_by_job_id: &HashMap<String, u8>,
    behavior_score_by_job_id: &HashMap<String, u8>,
) -> Vec<RankedJob> {
    if runtime.apply_learned {
        if let Some(aggregates) = learning_aggregates {
            let (reranked_jobs, _) = apply_learned_reranking(
                state,
                ranked_jobs,
                &aggregates.behavior,
                &aggregates.funnel,
                feedback_by_job_id,
                deterministic_score_by_job_id,
            );
            ranked_jobs = reranked_jobs;
        }
    }

    if runtime.apply_trained {
        let event_signals_by_job_id = learning_aggregates
            .map(|aggregates| {
                event_signals_by_job_id(&aggregates.events)
                    .into_iter()
                    .collect::<HashMap<_, _>>()
            })
            .unwrap_or_default();
        let applications_by_job_id = learning_aggregates
            .map(|aggregates| aggregates.applications_by_job_id.clone())
            .unwrap_or_default();
        let (reranked_jobs, _) = apply_trained_reranking(
            state,
            ranked_jobs,
            deterministic_score_by_job_id,
            behavior_score_by_job_id,
            feedback_by_job_id,
            &event_signals_by_job_id,
            &applications_by_job_id,
        );
        ranked_jobs = reranked_jobs;
    }

    ranked_jobs
}

fn top_reranker_comparison_items(
    ranked_jobs: &[RankedJob],
    top_n: usize,
) -> Vec<SearchRerankerComparisonItemResponse> {
    ranked_jobs
        .iter()
        .take(top_n)
        .map(|ranked| SearchRerankerComparisonItemResponse {
            job_id: ranked.job.job.id.clone(),
            score: ranked.fit.score,
        })
        .collect()
}

fn ranked_jobs_differ(left: &[RankedJob], right: &[RankedJob]) -> bool {
    left.len() != right.len()
        || left.iter().zip(right.iter()).any(|(left, right)| {
            left.job.job.id != right.job.job.id || left.fit.score != right.fit.score
        })
}

pub(super) fn mark_reranker_fallback(ranked_jobs: &mut [RankedJob], reason: &str) {
    for ranked in ranked_jobs {
        if matches!(
            ranked.fit.score_breakdown.reranker_mode,
            RerankerMode::Deterministic | RerankerMode::Fallback
        ) {
            ranked.fit.mark_reranker_fallback(reason.to_string());
        }
    }
}
