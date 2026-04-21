use std::collections::HashMap;

use axum::extract::{Query, State};
use serde::Deserialize;
use serde_json::json;
use tracing::info;

use crate::api::dto::search::{RunSearchRequest, RunSearchResponse, SearchResponse};
use crate::api::error::{ApiError, ApiJson};
use crate::api::routes::events::log_user_event_softly;
use crate::api::routes::jobs::load_feedback_state;
use crate::domain::feedback::model::CompanyFeedbackStatus;
use crate::domain::user_event::model::{CreateUserEvent, UserEventType};
use crate::services::matching::summarize_match_quality;
use crate::services::ranking::runtime::{RerankerRuntimeMode, resolve_reranker_runtime};
use crate::state::AppState;

use super::{
    apply_behavior_scoring, apply_feedback_scoring, apply_learned_reranking, apply_salary_scoring,
    apply_trained_reranking, build_reranker_comparison, load_learning_aggregates,
    load_search_salary_expectation, mark_reranker_fallback, score_by_job_id,
};

#[derive(Deserialize)]
pub struct SearchQuery {
    pub q: String,
    pub limit: Option<i64>,
}

pub async fn search(
    State(state): State<AppState>,
    Query(query): Query<SearchQuery>,
) -> Result<axum::Json<SearchResponse>, ApiError> {
    let q = query.q.trim();

    if q.is_empty() {
        return Err(ApiError::bad_request(
            "invalid_search_query",
            "Query parameter 'q' must not be empty",
        ));
    }

    let limit = query.limit.unwrap_or(10).clamp(1, 25);
    let jobs = state
        .jobs_service
        .search_active(q, limit)
        .await
        .map_err(|error| ApiError::from_repository(error, "search_query_failed"))?;
    let applications = state
        .applications_service
        .search_by_job_title(q, limit)
        .await
        .map_err(|error| ApiError::from_repository(error, "search_query_failed"))?;

    Ok(axum::Json(SearchResponse::new(jobs, applications)))
}

pub async fn run_search(
    State(state): State<AppState>,
    ApiJson(payload): ApiJson<RunSearchRequest>,
) -> Result<axum::Json<RunSearchResponse>, ApiError> {
    let input = payload.validate()?;
    let fetch_limit = (input.limit * 5).clamp(50, 200);

    let candidate_jobs = state
        .jobs_service
        .list_filtered_views(fetch_limit, Some("active"), None)
        .await
        .map_err(|error| ApiError::from_repository(error, "search_run_failed"))?;
    let total_candidates = candidate_jobs.len();
    let learning_aggregates = load_learning_aggregates(&state, input.profile_id.as_deref()).await;
    let feedback_states =
        load_feedback_state(&state, input.profile_id.as_deref(), &candidate_jobs).await?;
    let mut filtered_out_hidden = 0usize;
    let mut filtered_out_company_blacklist = 0usize;
    let jobs_with_feedback = candidate_jobs
        .into_iter()
        .zip(feedback_states.into_iter())
        .filter_map(|(job, feedback)| {
            if feedback.hidden {
                filtered_out_hidden += 1;
                return None;
            }

            if feedback.company_status == Some(CompanyFeedbackStatus::Blacklist) {
                filtered_out_company_blacklist += 1;
                return None;
            }

            Some((job, feedback))
        })
        .collect::<Vec<_>>();
    let mut feedback_by_job_id = HashMap::new();
    let ranked_candidates = jobs_with_feedback
        .iter()
        .map(|(job, feedback)| {
            feedback_by_job_id.insert(job.job.id.clone(), feedback.clone());
            job.clone()
        })
        .collect::<Vec<_>>();

    let result = state
        .search_matching_service
        .run(&input.search_profile, ranked_candidates);
    let salary_expectation =
        load_search_salary_expectation(&state, input.profile_id.as_deref()).await?;
    let deterministic_score_by_job_id = result
        .ranked_jobs
        .iter()
        .map(|ranked| (ranked.job.job.id.clone(), ranked.fit.score))
        .collect::<HashMap<_, _>>();

    let mut adjusted_jobs = apply_salary_scoring(result.ranked_jobs, salary_expectation.as_ref());
    adjusted_jobs = apply_feedback_scoring(adjusted_jobs, &feedback_by_job_id);
    if let Some(aggregates) = learning_aggregates.as_ref() {
        adjusted_jobs = apply_behavior_scoring(&state, adjusted_jobs, &aggregates.behavior);
    }
    let behavior_score_by_job_id = score_by_job_id(&adjusted_jobs);
    let deterministic_ranked_jobs = adjusted_jobs.clone();
    let reranker_runtime = resolve_reranker_runtime(
        state.reranker_runtime_mode,
        state.learned_reranker_enabled,
        learning_aggregates.is_some(),
        &state.trained_reranker_availability,
    );
    let mut learned_reranker_adjusted_jobs = 0usize;
    if reranker_runtime.apply_learned {
        if let Some(aggregates) = learning_aggregates.as_ref() {
            let (reranked_jobs, adjusted_count) = apply_learned_reranking(
                &state,
                adjusted_jobs,
                &aggregates.behavior,
                &aggregates.funnel,
                &feedback_by_job_id,
                &deterministic_score_by_job_id,
            );
            adjusted_jobs = reranked_jobs;
            learned_reranker_adjusted_jobs = adjusted_count;
        }
    }
    let mut trained_reranker_adjusted_jobs = 0usize;
    if reranker_runtime.apply_trained {
        let (reranked_jobs, adjusted_count) = apply_trained_reranking(
            &state,
            adjusted_jobs,
            &deterministic_score_by_job_id,
            &behavior_score_by_job_id,
        );
        adjusted_jobs = reranked_jobs;
        trained_reranker_adjusted_jobs = adjusted_count;
    }
    if matches!(
        reranker_runtime.active_mode,
        RerankerRuntimeMode::Deterministic
    ) {
        if let Some(reason) = reranker_runtime.fallback_reason.as_deref() {
            mark_reranker_fallback(&mut adjusted_jobs, reason);
        }
    }
    let reranker_comparison = input.reranker_comparison.as_ref().map(|comparison| {
        build_reranker_comparison(
            &state,
            comparison,
            &reranker_runtime,
            &deterministic_ranked_jobs,
            &adjusted_jobs,
            learning_aggregates.as_ref(),
            &feedback_by_job_id,
            &deterministic_score_by_job_id,
            &behavior_score_by_job_id,
        )
    });
    adjusted_jobs.truncate(input.limit as usize);
    let quality = summarize_match_quality(&adjusted_jobs);
    let ranked_jobs: Vec<crate::api::dto::search::RankedJobResponse> = adjusted_jobs
        .into_iter()
        .map(|ranked| {
            let feedback = feedback_by_job_id
                .get(&ranked.job.job.id)
                .cloned()
                .unwrap_or_default();

            crate::api::dto::search::RankedJobResponse {
                job: crate::api::dto::jobs::JobResponse::from_view_with_feedback(
                    ranked.job, feedback,
                ),
                fit: crate::api::dto::search::JobFitResponse::from(ranked.fit),
            }
        })
        .collect();

    let meta = crate::api::dto::search::SearchRunMetaResponse {
        total_candidates,
        filtered_out_by_source: result.filtered_out_by_source,
        filtered_out_hidden,
        filtered_out_company_blacklist,
        scored_jobs: total_candidates
            .saturating_sub(result.filtered_out_by_source)
            .saturating_sub(filtered_out_hidden)
            .saturating_sub(filtered_out_company_blacklist),
        returned_jobs: ranked_jobs.len(),
        low_evidence_jobs: quality.low_evidence_jobs,
        weak_description_jobs: quality.weak_description_jobs,
        role_mismatch_jobs: quality.role_mismatch_jobs,
        seniority_mismatch_jobs: quality.seniority_mismatch_jobs,
        source_mismatch_jobs: quality.source_mismatch_jobs,
        top_missing_signals: quality.top_missing_signals,
        reranker_mode_requested: reranker_runtime.requested_mode.as_str().to_string(),
        reranker_mode_active: reranker_runtime.active_mode.as_str().to_string(),
        reranker_fallback_reason: reranker_runtime.fallback_reason.clone(),
        learned_reranker_enabled: state.learned_reranker_enabled,
        learned_reranker_adjusted_jobs,
        trained_reranker_enabled: state.trained_reranker_availability.is_ready(),
        trained_reranker_adjusted_jobs,
        reranker_comparison,
    };

    info!(
        profile_id = input.profile_id.as_deref().unwrap_or(""),
        requested_reranker_mode = meta.reranker_mode_requested.as_str(),
        active_reranker_mode = meta.reranker_mode_active.as_str(),
        reranker_fallback_reason = meta.reranker_fallback_reason.as_deref().unwrap_or("none"),
        learned_reranker_adjusted_jobs = meta.learned_reranker_adjusted_jobs,
        trained_reranker_adjusted_jobs = meta.trained_reranker_adjusted_jobs,
        "search reranker path resolved"
    );

    if let Some(profile_id) = input.profile_id.clone() {
        let allowed_sources = input
            .search_profile
            .allowed_sources
            .iter()
            .map(|source| source.canonical_key().to_string())
            .collect::<Vec<_>>();
        let primary_source = match allowed_sources.as_slice() {
            [source] => Some(source.clone()),
            _ => None,
        };

        log_user_event_softly(
            &state,
            CreateUserEvent {
                profile_id,
                event_type: UserEventType::SearchRun,
                job_id: None,
                company_name: None,
                source: primary_source,
                role_family: input
                    .search_profile
                    .primary_role
                    .family()
                    .map(str::to_string),
                payload_json: Some(json!({
                    "limit": input.limit,
                    "primary_role": input.search_profile.primary_role.canonical_key(),
                    "primary_role_confidence": input.search_profile.primary_role_confidence,
                    "target_roles": input
                        .search_profile
                        .target_roles
                        .iter()
                        .map(|role| role.canonical_key())
                        .collect::<Vec<_>>(),
                    "allowed_sources": allowed_sources,
                    "target_regions": input.search_profile.target_regions,
                    "work_modes": input.search_profile.work_modes,
                    "search_terms": input.search_profile.search_terms,
                    "exclude_terms": input.search_profile.exclude_terms,
                    "meta": {
                        "total_candidates": meta.total_candidates,
                        "filtered_out_by_source": meta.filtered_out_by_source,
                        "filtered_out_hidden": meta.filtered_out_hidden,
                        "filtered_out_company_blacklist": meta.filtered_out_company_blacklist,
                        "scored_jobs": meta.scored_jobs,
                        "returned_jobs": meta.returned_jobs,
                        "reranker_mode_requested": meta.reranker_mode_requested.as_str(),
                        "reranker_mode_active": meta.reranker_mode_active.as_str(),
                        "reranker_fallback_reason": meta.reranker_fallback_reason.as_deref(),
                        "learned_reranker_enabled": meta.learned_reranker_enabled,
                        "learned_reranker_adjusted_jobs": meta.learned_reranker_adjusted_jobs,
                        "trained_reranker_enabled": meta.trained_reranker_enabled,
                        "trained_reranker_adjusted_jobs": meta.trained_reranker_adjusted_jobs,
                    }
                })),
            },
        )
        .await;
    }

    Ok(axum::Json(RunSearchResponse {
        meta,
        results: ranked_jobs,
    }))
}
