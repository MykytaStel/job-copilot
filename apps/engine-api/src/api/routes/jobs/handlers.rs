use std::time::Instant;

use axum::Extension;
use axum::extract::{Path, Query, State};
use tracing::{info, warn};

use crate::api::dto::jobs::{
    JobResponse, MlJobLifecycleResponse, RecentJobsResponse, score_signals_from_fit,
};
use crate::api::dto::ranking::FitScoreResponse;
use crate::api::dto::search::JobFitResponse;
use crate::api::error::{ApiError, ApiJson};
use crate::api::middleware::auth::{AuthUser, check_profile_ownership};
use crate::services::search_ranking::summarize_match_quality;
use crate::state::AppState;

use super::{
    BulkProfileJobMatchMeta, BulkProfileJobMatchRequest, BulkProfileJobMatchResponse,
    JobContextQuery, RecentJobsQuery, helpers::load_profile_ranked_jobs, load_feedback_state,
};

pub async fn get_job_by_id(
    State(state): State<AppState>,
    Path(job_id): Path<String>,
    Query(query): Query<JobContextQuery>,
) -> Result<axum::Json<JobResponse>, ApiError> {
    let Some(job) = state
        .jobs_service
        .get_view_by_id(&job_id)
        .await
        .map_err(|error| ApiError::from_repository(error, "jobs_query_failed"))?
    else {
        return Err(ApiError::not_found(
            "job_not_found",
            format!("Job '{job_id}' was not found"),
        ));
    };

    let feedback = load_feedback_state(
        &state,
        query.profile_id.as_deref(),
        std::slice::from_ref(&job),
    )
    .await?
    .into_iter()
    .next()
    .unwrap_or_default();

    let mut response = JobResponse::from_view_with_feedback(job, feedback);

    if let Some(profile_id) = query.profile_id.as_deref() {
        match load_profile_ranked_jobs(
            &state,
            profile_id,
            std::slice::from_ref(&job_id),
            "get_job_by_id_score_signals",
        )
        .await
        {
            Ok(ranked) => {
                if let Some(ranked_job) = ranked
                    .ranked_jobs
                    .iter()
                    .find(|ranked_job| ranked_job.job.job.id == job_id)
                {
                    response.presentation.score_signals = score_signals_from_fit(&ranked_job.fit);
                }
            }
            Err(error) => {
                warn!(
                    error = ?error,
                    profile_id,
                    job_id,
                    "failed to load score signals for job detail; continuing without them"
                );
            }
        }
    }

    Ok(axum::Json(response))
}

pub async fn get_ml_job_lifecycle(
    State(state): State<AppState>,
    Path(job_id): Path<String>,
) -> Result<axum::Json<MlJobLifecycleResponse>, ApiError> {
    let Some(job) = state
        .jobs_service
        .get_view_by_id(&job_id)
        .await
        .map_err(|error| ApiError::from_repository(error, "jobs_query_failed"))?
    else {
        return Err(ApiError::not_found(
            "job_not_found",
            format!("Job '{job_id}' was not found"),
        ));
    };

    Ok(axum::Json(MlJobLifecycleResponse::from(job)))
}

pub async fn get_recent_jobs(
    State(state): State<AppState>,
    Query(query): Query<RecentJobsQuery>,
) -> Result<axum::Json<RecentJobsResponse>, ApiError> {
    let started_at = Instant::now();
    let limit = query.limit.unwrap_or(50);

    if !(1..=200).contains(&limit) {
        return Err(ApiError::invalid_limit(limit));
    }

    let lifecycle = query.lifecycle.as_deref();
    let source = query
        .source
        .map(crate::domain::source::SourceId::canonical_key);
    let profile_id = query.profile_id.as_deref();

    let fetch_started_at = Instant::now();
    let jobs = state
        .jobs_service
        .list_filtered_views(limit, lifecycle, source)
        .await
        .map_err(|error| ApiError::from_repository(error, "jobs_query_failed"))?;
    let fetch_duration_ms = fetch_started_at.elapsed().as_millis();
    let fetched_jobs = jobs.len();

    let feedback_started_at = Instant::now();
    let feedback_states = load_feedback_state(&state, profile_id, &jobs).await?;
    let feedback_duration_ms = feedback_started_at.elapsed().as_millis();
    let jobs: Vec<JobResponse> = jobs
        .into_iter()
        .zip(feedback_states.into_iter())
        .filter(|(_, feedback)| !feedback.hidden)
        .map(|(job, feedback)| JobResponse::from_view_with_feedback(job, feedback))
        .collect();
    let returned_jobs = jobs.len();
    let summary = state
        .jobs_service
        .feed_summary()
        .await
        .map_err(|error| ApiError::from_repository(error, "jobs_query_failed"))?;

    info!(
        limit,
        lifecycle = lifecycle.unwrap_or("all"),
        source = source.unwrap_or("all"),
        profile_id = profile_id.unwrap_or(""),
        fetched_jobs,
        returned_jobs,
        fetch_duration_ms,
        feedback_duration_ms,
        total_duration_ms = started_at.elapsed().as_millis(),
        "recent jobs feed loaded"
    );

    Ok(axum::Json(RecentJobsResponse {
        jobs,
        summary: summary.into(),
    }))
}

pub async fn get_profile_job_match(
    State(state): State<AppState>,
    auth: Option<Extension<AuthUser>>,
    Path((profile_id, job_id)): Path<(String, String)>,
) -> Result<axum::Json<JobFitResponse>, ApiError> {
    check_profile_ownership(auth.as_deref(), &profile_id)?;
    let ranked_jobs = load_profile_ranked_jobs(
        &state,
        &profile_id,
        std::slice::from_ref(&job_id),
        "get_profile_job_match",
    )
    .await?
    .ranked_jobs;
    let Some(ranked) = ranked_jobs
        .into_iter()
        .find(|ranked| ranked.job.job.id == job_id)
    else {
        return Err(ApiError::not_found(
            "job_not_found",
            format!("Job '{job_id}' was not found"),
        ));
    };

    Ok(axum::Json(JobFitResponse::from(ranked.fit)))
}

pub async fn bulk_profile_job_match(
    State(state): State<AppState>,
    auth: Option<Extension<AuthUser>>,
    Path(profile_id): Path<String>,
    ApiJson(payload): ApiJson<BulkProfileJobMatchRequest>,
) -> Result<axum::Json<BulkProfileJobMatchResponse>, ApiError> {
    check_profile_ownership(auth.as_deref(), &profile_id)?;
    let input = payload.validate()?;
    let ranked = load_profile_ranked_jobs(
        &state,
        &profile_id,
        &input.job_ids,
        "bulk_profile_job_match",
    )
    .await?;
    let quality = summarize_match_quality(&ranked.ranked_jobs);
    let results = ranked
        .ranked_jobs
        .into_iter()
        .map(|ranked| JobFitResponse::from(ranked.fit))
        .collect::<Vec<_>>();

    Ok(axum::Json(BulkProfileJobMatchResponse {
        profile_id,
        meta: BulkProfileJobMatchMeta {
            returned_jobs: results.len(),
            low_evidence_jobs: quality.low_evidence_jobs,
            weak_description_jobs: quality.weak_description_jobs,
            role_mismatch_jobs: quality.role_mismatch_jobs,
            seniority_mismatch_jobs: quality.seniority_mismatch_jobs,
            source_mismatch_jobs: quality.source_mismatch_jobs,
            top_missing_signals: quality.top_missing_signals,
            reranker_mode_requested: ranked.reranker_runtime.requested_mode.as_str().to_string(),
            reranker_mode_active: ranked.reranker_runtime.active_mode.as_str().to_string(),
            reranker_fallback_reason: ranked.reranker_runtime.fallback_reason,
        },
        results,
    }))
}

pub async fn get_job_match(
    State(state): State<AppState>,
    Path(job_id): Path<String>,
) -> Result<axum::Json<FitScoreResponse>, ApiError> {
    let Some(resume) = state
        .resumes_service
        .get_active()
        .await
        .map_err(|error| ApiError::from_repository(error, "match_query_failed"))?
    else {
        return Err(ApiError::not_found(
            "active_resume_not_found",
            "No active resume was found",
        ));
    };

    let Some(score) = state
        .fit_scores_repository
        .get_for_job_and_resume(&job_id, &resume.id)
        .await
        .map_err(|error| ApiError::from_repository(error, "match_query_failed"))?
    else {
        return Err(ApiError::not_found(
            "match_result_not_found",
            format!("No fit score exists for job '{job_id}' — call GET /fit first"),
        ));
    };

    Ok(axum::Json(FitScoreResponse::from(score)))
}

pub async fn get_job_fit(
    State(state): State<AppState>,
    Path(job_id): Path<String>,
) -> Result<axum::Json<FitScoreResponse>, ApiError> {
    let Some(resume) = state
        .resumes_service
        .get_active()
        .await
        .map_err(|error| ApiError::from_repository(error, "fit_query_failed"))?
    else {
        return Err(ApiError::not_found(
            "active_resume_not_found",
            "No active resume was found",
        ));
    };

    if let Ok(Some(cached)) = state
        .fit_scores_repository
        .get_for_job_and_resume(&job_id, &resume.id)
        .await
    {
        return Ok(axum::Json(FitScoreResponse::from(cached)));
    }

    let Some(job) = state
        .jobs_service
        .get_by_id(&job_id)
        .await
        .map_err(|error| ApiError::from_repository(error, "fit_query_failed"))?
    else {
        return Err(ApiError::not_found(
            "job_not_found",
            format!("Job '{job_id}' was not found"),
        ));
    };

    let candidate = state.profile_analysis.analyze(&resume.raw_text);
    let profile = state.profile_records.get_latest().await.ok().flatten();

    let score = state
        .fit_scoring
        .compute(&candidate, &job, profile.as_ref());

    if let Err(error) = state.fit_scores_repository.upsert(&score, &resume.id).await {
        warn!(
            job_id = %job_id,
            resume_id = %resume.id,
            error = ?error,
            "failed to persist fit score"
        );
    }

    Ok(axum::Json(FitScoreResponse::from(score)))
}

pub async fn score_job_match(
    State(state): State<AppState>,
    Path(job_id): Path<String>,
) -> Result<axum::Json<FitScoreResponse>, ApiError> {
    let Some(job) = state
        .jobs_service
        .get_by_id(&job_id)
        .await
        .map_err(|error| ApiError::from_repository(error, "match_query_failed"))?
    else {
        return Err(ApiError::not_found(
            "job_not_found",
            format!("Job '{job_id}' was not found"),
        ));
    };

    let Some(resume) = state
        .resumes_service
        .get_active()
        .await
        .map_err(|error| ApiError::from_repository(error, "match_query_failed"))?
    else {
        return Err(ApiError::not_found(
            "active_resume_not_found",
            "No active resume was found",
        ));
    };

    let candidate = state.profile_analysis.analyze(&resume.raw_text);
    let profile = state.profile_records.get_latest().await.ok().flatten();
    let score = state
        .fit_scoring
        .compute(&candidate, &job, profile.as_ref());

    if let Err(error) = state.fit_scores_repository.upsert(&score, &resume.id).await {
        warn!(
            job_id = %job_id,
            resume_id = %resume.id,
            error = ?error,
            "failed to persist fit score"
        );
    }

    Ok(axum::Json(FitScoreResponse::from(score)))
}
