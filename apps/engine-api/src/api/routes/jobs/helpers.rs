use std::collections::HashMap;

use tracing::info;

use crate::api::error::ApiError;
use crate::api::routes::feedback::ensure_profile_exists;
use crate::api::routes::search::{
    apply_behavior_scoring, apply_feedback_scoring, apply_learned_reranking,
    apply_trained_reranking, load_learning_aggregates, score_by_job_id,
};
use crate::domain::feedback::model::{CompanyFeedbackRecord, JobFeedbackRecord, JobFeedbackState};
use crate::domain::matching::RerankerMode;
use crate::domain::search::profile::SearchPreferences;
use crate::services::feedback::FeedbackService;
use crate::state::AppState;

use super::ProfileRankedJobsResult;

pub(super) async fn load_profile_ranked_jobs(
    state: &AppState,
    profile_id: &str,
    job_ids: &[String],
    entrypoint: &'static str,
) -> Result<ProfileRankedJobsResult, ApiError> {
    ensure_profile_exists(state, profile_id).await?;

    let profile = state
        .profiles_service
        .get_by_id(profile_id)
        .await
        .map_err(|error| ApiError::from_repository(error, "profiles_query_failed"))?
        .expect("profile existence already verified above");
    let analyzed_profile = state.profile_analysis_service.analyze(&profile.raw_text);
    let search_profile = state
        .search_profile_service
        .build(&analyzed_profile, &SearchPreferences::default());

    let mut jobs = Vec::new();
    for job_id in job_ids {
        if let Some(job) = state
            .jobs_service
            .get_view_by_id(job_id)
            .await
            .map_err(|error| ApiError::from_repository(error, "jobs_query_failed"))?
        {
            jobs.push(job);
        }
    }

    let feedback_states = load_feedback_state(state, Some(profile_id), &jobs).await?;
    let mut feedback_by_job_id = HashMap::new();
    let ranked_candidates = jobs
        .into_iter()
        .zip(feedback_states)
        .map(|(job, feedback)| {
            feedback_by_job_id.insert(job.job.id.clone(), feedback);
            job
        })
        .collect::<Vec<_>>();

    let result = state
        .search_matching_service
        .run(&search_profile, ranked_candidates);
    let deterministic_score_by_job_id = result
        .ranked_jobs
        .iter()
        .map(|ranked| (ranked.job.job.id.clone(), ranked.fit.score))
        .collect::<HashMap<_, _>>();

    let mut adjusted_jobs = apply_feedback_scoring(result.ranked_jobs, &feedback_by_job_id);
    let learning_aggregates = load_learning_aggregates(state, Some(profile_id)).await;

    if let Some(aggregates) = learning_aggregates.as_ref() {
        adjusted_jobs = apply_behavior_scoring(state, adjusted_jobs, &aggregates.behavior);
    }

    let behavior_score_by_job_id = score_by_job_id(&adjusted_jobs);
    let reranker_runtime = crate::services::ranking::runtime::resolve_reranker_runtime(
        state.reranker_runtime_mode,
        state.learned_reranker_enabled,
        learning_aggregates.is_some(),
        &state.trained_reranker_availability,
    );

    if reranker_runtime.apply_learned {
        if let Some(aggregates) = learning_aggregates.as_ref() {
            let (reranked_jobs, _adjusted_count) = apply_learned_reranking(
                state,
                adjusted_jobs,
                &aggregates.behavior,
                &aggregates.funnel,
                &feedback_by_job_id,
                &deterministic_score_by_job_id,
            );
            adjusted_jobs = reranked_jobs;
        }
    }

    if reranker_runtime.apply_trained {
        let (reranked_jobs, _adjusted_count) = apply_trained_reranking(
            state,
            adjusted_jobs,
            &deterministic_score_by_job_id,
            &behavior_score_by_job_id,
        );
        adjusted_jobs = reranked_jobs;
    }

    if matches!(
        reranker_runtime.active_mode,
        crate::services::ranking::runtime::RerankerRuntimeMode::Deterministic
    ) {
        if let Some(reason) = reranker_runtime.fallback_reason.as_deref() {
            mark_reranker_fallback(&mut adjusted_jobs, reason);
        }
    }

    info!(
        entrypoint,
        profile_id,
        requested_reranker_mode = reranker_runtime.requested_mode.as_str(),
        active_reranker_mode = reranker_runtime.active_mode.as_str(),
        reranker_fallback_reason = reranker_runtime
            .fallback_reason
            .as_deref()
            .unwrap_or("none"),
        "jobs reranker path resolved"
    );

    Ok(ProfileRankedJobsResult {
        ranked_jobs: adjusted_jobs,
        reranker_runtime,
    })
}

fn mark_reranker_fallback(ranked_jobs: &mut [crate::services::matching::RankedJob], reason: &str) {
    for ranked in ranked_jobs {
        if matches!(
            ranked.fit.score_breakdown.reranker_mode,
            RerankerMode::Deterministic | RerankerMode::Fallback
        ) {
            ranked.fit.mark_reranker_fallback(reason.to_string());
        }
    }
}

pub(crate) async fn load_feedback_state(
    state: &AppState,
    profile_id: Option<&str>,
    jobs: &[crate::domain::job::model::JobView],
) -> Result<Vec<JobFeedbackState>, ApiError> {
    let Some(profile_id) = profile_id.filter(|value| !value.trim().is_empty()) else {
        return Ok(vec![JobFeedbackState::default(); jobs.len()]);
    };

    ensure_profile_exists(state, profile_id).await?;

    let job_ids = jobs
        .iter()
        .map(|job| job.job.id.clone())
        .collect::<Vec<_>>();
    let normalized_company_names = jobs
        .iter()
        .map(|job| FeedbackService::normalize_company_name(&job.job.company_name))
        .collect::<Vec<_>>();

    let job_feedback = state
        .feedback_service
        .list_job_feedback_for_jobs(profile_id, &job_ids)
        .await
        .map_err(|error| ApiError::from_repository(error, "feedback_query_failed"))?;
    let company_feedback = state
        .feedback_service
        .list_company_feedback_for_names(profile_id, &normalized_company_names)
        .await
        .map_err(|error| ApiError::from_repository(error, "feedback_query_failed"))?;

    Ok(build_feedback_states(jobs, job_feedback, company_feedback))
}

pub(crate) fn build_feedback_states(
    jobs: &[crate::domain::job::model::JobView],
    job_feedback: Vec<JobFeedbackRecord>,
    company_feedback: Vec<CompanyFeedbackRecord>,
) -> Vec<JobFeedbackState> {
    let job_feedback_by_job_id = job_feedback
        .into_iter()
        .map(|record| (record.job_id.clone(), record))
        .collect::<HashMap<_, _>>();
    let company_feedback_by_name = company_feedback
        .into_iter()
        .map(|record| (record.normalized_company_name.clone(), record))
        .collect::<HashMap<_, _>>();

    jobs.iter()
        .map(|job| {
            let normalized_company_name =
                FeedbackService::normalize_company_name(&job.job.company_name);
            JobFeedbackState::from_sources(
                job_feedback_by_job_id.get(&job.job.id),
                company_feedback_by_name.get(&normalized_company_name),
            )
        })
        .collect()
}
