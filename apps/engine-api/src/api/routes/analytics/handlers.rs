use std::collections::BTreeMap;

use axum::Extension;
use axum::extract::{Path, State};
use sqlx::FromRow;
use sqlx::types::Json as SqlJson;

use crate::api::dto::analytics::{
    AnalyticsSummaryResponse, FeedbackSummarySection, FunnelSummaryResponse, IngestionSourceEntry,
    IngestionStatsResponse, JobsByLifecycleSection, JobsBySourceEntry, LlmContextAnalyzedProfile,
    LlmContextResponse, SalaryIntelligenceResponse,
};
use crate::api::error::ApiError;
use crate::api::middleware::auth::AuthUser;
use crate::api::routes::feedback::ensure_profile_exists;
use crate::domain::feedback::model::CompanyFeedbackStatus;
use crate::domain::source::SOURCE_CATALOG;
use crate::services::funnel::FunnelService;
use crate::state::AppState;

use super::helpers::{build_job_feedback_evidence_entries, build_search_quality_summary};

const DEFAULT_INGESTION_INTERVAL_MINUTES: i64 = 60;

pub async fn get_salary_intelligence(
    State(state): State<AppState>,
) -> Result<axum::Json<SalaryIntelligenceResponse>, ApiError> {
    let buckets = state
        .salary_service
        .salary_intelligence()
        .await
        .map_err(|error| ApiError::from_repository(error, "salary_query_failed"))?;

    Ok(axum::Json(SalaryIntelligenceResponse {
        buckets: buckets
            .into_iter()
            .map(crate::api::dto::analytics::SalaryBucketResponse::from)
            .collect(),
    }))
}

pub async fn get_analytics_summary(
    State(state): State<AppState>,
    auth: Option<Extension<AuthUser>>,
    Path(profile_id): Path<String>,
) -> Result<axum::Json<AnalyticsSummaryResponse>, ApiError> {
    ensure_profile_exists(&state, auth.as_deref(), &profile_id).await?;

    let profile = state
        .profile_records
        .get_by_id(&profile_id)
        .await
        .map_err(|error| ApiError::from_repository(error, "profiles_query_failed"))?
        .expect("profile existence already verified above");

    let job_feedback = state
        .feedback_service
        .list_job_feedback(&profile_id)
        .await
        .map_err(|error| ApiError::from_repository(error, "feedback_query_failed"))?;

    let company_feedback = state
        .feedback_service
        .list_company_feedback(&profile_id)
        .await
        .map_err(|error| ApiError::from_repository(error, "feedback_query_failed"))?;

    let feed_summary = state
        .jobs_service
        .feed_summary()
        .await
        .map_err(|error| ApiError::from_repository(error, "jobs_query_failed"))?;

    let source_counts = state
        .jobs_service
        .jobs_by_source()
        .await
        .map_err(|error| ApiError::from_repository(error, "jobs_query_failed"))?;

    let feedback = FeedbackSummarySection {
        saved_jobs_count: job_feedback.iter().filter(|j| j.saved).count(),
        hidden_jobs_count: job_feedback.iter().filter(|j| j.hidden).count(),
        bad_fit_jobs_count: job_feedback.iter().filter(|j| j.bad_fit).count(),
        whitelisted_companies_count: company_feedback
            .iter()
            .filter(|c| c.status == CompanyFeedbackStatus::Whitelist)
            .count(),
        blacklisted_companies_count: company_feedback
            .iter()
            .filter(|c| c.status == CompanyFeedbackStatus::Blacklist)
            .count(),
    };

    let (top_matched_roles, top_matched_skills, top_matched_keywords) =
        if let Some(analysis) = &profile.analysis {
            (
                vec![analysis.primary_role.to_string()],
                analysis.skills.iter().take(10).cloned().collect(),
                analysis.keywords.iter().take(10).cloned().collect(),
            )
        } else {
            (vec![], vec![], vec![])
        };

    let search_quality = build_search_quality_summary(&state, &profile.raw_text).await?;

    Ok(axum::Json(AnalyticsSummaryResponse {
        profile_id,
        feedback,
        jobs_by_source: source_counts
            .into_iter()
            .map(|s| JobsBySourceEntry {
                source: s.source,
                count: s.count,
            })
            .collect(),
        jobs_by_lifecycle: JobsByLifecycleSection {
            total: feed_summary.total_jobs,
            active: feed_summary.active_jobs,
            inactive: feed_summary.inactive_jobs,
            reactivated: feed_summary.reactivated_jobs,
        },
        top_matched_roles,
        top_matched_skills,
        top_matched_keywords,
        search_quality,
    }))
}

pub async fn get_funnel_summary(
    State(state): State<AppState>,
    auth: Option<Extension<AuthUser>>,
    Path(profile_id): Path<String>,
) -> Result<axum::Json<FunnelSummaryResponse>, ApiError> {
    ensure_profile_exists(&state, auth.as_deref(), &profile_id).await?;

    let events = state
        .user_events_service
        .list_by_profile(&profile_id)
        .await
        .map_err(|error| ApiError::from_repository(error, "user_event_query_failed"))?;
    let funnel_service = FunnelService::new();
    let aggregates = funnel_service.build_aggregates(events.iter());
    let summary = funnel_service.summarize(&aggregates);

    Ok(axum::Json(FunnelSummaryResponse::from_summary(
        profile_id, summary,
    )))
}

pub async fn get_llm_context(
    State(state): State<AppState>,
    auth: Option<Extension<AuthUser>>,
    Path(profile_id): Path<String>,
) -> Result<axum::Json<LlmContextResponse>, ApiError> {
    ensure_profile_exists(&state, auth.as_deref(), &profile_id).await?;

    let profile = state
        .profile_records
        .get_by_id(&profile_id)
        .await
        .map_err(|error| ApiError::from_repository(error, "profiles_query_failed"))?
        .expect("profile existence already verified above");

    let job_feedback = state
        .feedback_service
        .list_job_feedback(&profile_id)
        .await
        .map_err(|error| ApiError::from_repository(error, "feedback_query_failed"))?;

    let company_feedback = state
        .feedback_service
        .list_company_feedback(&profile_id)
        .await
        .map_err(|error| ApiError::from_repository(error, "feedback_query_failed"))?;

    let feed_summary = state
        .jobs_service
        .feed_summary()
        .await
        .map_err(|error| ApiError::from_repository(error, "jobs_query_failed"))?;

    let analyzed_profile = profile
        .analysis
        .as_ref()
        .map(|analysis| LlmContextAnalyzedProfile {
            summary: analysis.summary.clone(),
            primary_role: analysis.primary_role.to_string(),
            seniority: analysis.seniority.clone(),
            skills: analysis.skills.clone(),
            keywords: analysis.keywords.clone(),
        });

    let profile_skills = profile
        .analysis
        .as_ref()
        .map(|a| a.skills.clone())
        .unwrap_or_default();
    let profile_keywords = profile
        .analysis
        .as_ref()
        .map(|a| a.keywords.clone())
        .unwrap_or_default();

    let feedback_summary = FeedbackSummarySection {
        saved_jobs_count: job_feedback.iter().filter(|j| j.saved).count(),
        hidden_jobs_count: job_feedback.iter().filter(|j| j.hidden).count(),
        bad_fit_jobs_count: job_feedback.iter().filter(|j| j.bad_fit).count(),
        whitelisted_companies_count: company_feedback
            .iter()
            .filter(|c| c.status == CompanyFeedbackStatus::Whitelist)
            .count(),
        blacklisted_companies_count: company_feedback
            .iter()
            .filter(|c| c.status == CompanyFeedbackStatus::Blacklist)
            .count(),
    };

    let top_positive_evidence = build_job_feedback_evidence_entries(
        &state,
        job_feedback.iter().filter(|j| j.saved).take(10).collect(),
        "saved_job",
    )
    .await
    .into_iter()
    .chain(
        company_feedback
            .iter()
            .filter(|c| c.status == CompanyFeedbackStatus::Whitelist)
            .take(10)
            .map(|c| crate::api::dto::analytics::LlmContextEvidenceEntry {
                entry_type: "whitelisted_company".to_string(),
                label: c.company_name.clone(),
            }),
    )
    .collect();

    let top_negative_evidence = build_job_feedback_evidence_entries(
        &state,
        job_feedback.iter().filter(|j| j.bad_fit).take(10).collect(),
        "bad_fit_job",
    )
    .await
    .into_iter()
    .chain(
        company_feedback
            .iter()
            .filter(|c| c.status == CompanyFeedbackStatus::Blacklist)
            .take(10)
            .map(|c| crate::api::dto::analytics::LlmContextEvidenceEntry {
                entry_type: "blacklisted_company".to_string(),
                label: c.company_name.clone(),
            }),
    )
    .collect();

    Ok(axum::Json(LlmContextResponse {
        profile_id,
        analyzed_profile,
        profile_skills,
        profile_keywords,
        jobs_feed_summary: JobsByLifecycleSection {
            total: feed_summary.total_jobs,
            active: feed_summary.active_jobs,
            inactive: feed_summary.inactive_jobs,
            reactivated: feed_summary.reactivated_jobs,
        },
        feedback_summary,
        top_positive_evidence,
        top_negative_evidence,
    }))
}

pub async fn get_ingestion_stats(
    State(state): State<AppState>,
) -> Result<axum::Json<IngestionStatsResponse>, ApiError> {
    let feed_summary = state
        .jobs_service
        .feed_summary()
        .await
        .map_err(|error| ApiError::from_repository(error, "jobs_query_failed"))?;

    let source_counts = state
        .jobs_service
        .jobs_by_source()
        .await
        .map_err(|error| ApiError::from_repository(error, "jobs_query_failed"))?;
    let latest_runs = latest_ingestion_runs(&state).await?;
    let runs_by_source = latest_runs
        .into_iter()
        .map(|run| (run.source.clone(), run))
        .collect::<BTreeMap<_, _>>();
    let counts_by_source = source_counts
        .into_iter()
        .map(|source| (source.source.clone(), source))
        .collect::<BTreeMap<_, _>>();

    let mut source_keys = SOURCE_CATALOG
        .iter()
        .map(|source| source.canonical_key.to_string())
        .collect::<Vec<_>>();
    for source in counts_by_source.keys().chain(runs_by_source.keys()) {
        if !source_keys.contains(source) {
            source_keys.push(source.clone());
        }
    }

    Ok(axum::Json(IngestionStatsResponse {
        last_ingested_at: feed_summary.last_ingested_at,
        total_jobs: feed_summary.total_jobs as u32,
        active_jobs: feed_summary.active_jobs as u32,
        inactive_jobs: feed_summary.inactive_jobs as u32,
        sources: source_keys
            .into_iter()
            .map(|source| {
                let count = counts_by_source.get(&source);
                let run = runs_by_source.get(&source);

                IngestionSourceEntry {
                    display_name: display_name_for_source(&source),
                    source,
                    count: count.map(|s| s.count).unwrap_or(0) as u32,
                    last_seen: count.map(|s| s.last_seen.clone()),
                    last_run_at: run.map(|run| run.run_at.clone()),
                    next_scheduled_run_at: run.map(|run| run.next_scheduled_run_at.clone()),
                    status: run
                        .map(|run| run.status.clone())
                        .unwrap_or_else(|| "unknown".to_string()),
                    jobs_fetched: run.map(|run| run.jobs_fetched as u32).unwrap_or(0),
                    jobs_attempted: run.map(|run| run.jobs_attempted as u32).unwrap_or(0),
                    jobs_upserted: run.map(|run| run.jobs_upserted as u32).unwrap_or(0),
                    jobs_failed: run.map(|run| run.jobs_failed as u32).unwrap_or(0),
                    errors: run.map(|run| run.errors as u32).unwrap_or(0),
                    errors_json: run.map(|run| run.errors_json.0.clone()).unwrap_or_default(),
                }
            })
            .collect(),
    }))
}

#[derive(Debug, FromRow)]
struct IngestionRunRow {
    source: String,
    run_at: String,
    next_scheduled_run_at: String,
    jobs_fetched: i32,
    jobs_attempted: i32,
    jobs_upserted: i32,
    jobs_failed: i32,
    errors: i32,
    errors_json: SqlJson<Vec<String>>,
    status: String,
}

async fn latest_ingestion_runs(state: &AppState) -> Result<Vec<IngestionRunRow>, ApiError> {
    let Some(pool) = state.database.pool() else {
        return Ok(Vec::new());
    };

    sqlx::query_as::<_, IngestionRunRow>(
        r#"
        WITH ranked AS (
            SELECT
                source,
                run_at,
                jobs_fetched,
                jobs_attempted,
                jobs_upserted,
                jobs_failed,
                errors,
                errors_json,
                status,
                ROW_NUMBER() OVER (
                    PARTITION BY source
                    ORDER BY run_at DESC, id DESC
                ) AS run_rank
            FROM ingestion_runs
        )
        SELECT
            source,
            run_at::text AS run_at,
            (run_at + make_interval(mins => CAST($1 AS integer)))::text AS next_scheduled_run_at,
            jobs_fetched,
            jobs_attempted,
            jobs_upserted,
            jobs_failed,
            errors,
            errors_json,
            status
        FROM ranked
        WHERE run_rank = 1
        "#,
    )
    .bind(DEFAULT_INGESTION_INTERVAL_MINUTES)
    .fetch_all(pool)
    .await
    .map_err(|error| {
        tracing::error!(error = %error, "failed to query latest ingestion runs");
        ApiError::internal("ingestion_runs_query_failed", error.to_string())
    })
}

fn display_name_for_source(source: &str) -> String {
    SOURCE_CATALOG
        .iter()
        .find(|metadata| metadata.canonical_key == source)
        .map(|metadata| metadata.display_name.to_string())
        .unwrap_or_else(|| source.to_string())
}
