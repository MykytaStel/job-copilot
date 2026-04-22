use tracing::warn;

use crate::api::dto::analytics::{LlmContextEvidenceEntry, SearchQualitySummaryResponse};
use crate::api::error::ApiError;
use crate::domain::feedback::model::JobFeedbackRecord;
use crate::domain::search::profile::SearchPreferences;
use crate::services::search_ranking::summarize_match_quality;
use crate::state::AppState;

pub(super) async fn build_job_feedback_evidence_entries(
    state: &AppState,
    job_feedback: Vec<&JobFeedbackRecord>,
    entry_type: &str,
) -> Vec<LlmContextEvidenceEntry> {
    let mut entries = Vec::with_capacity(job_feedback.len());

    for feedback in job_feedback {
        entries.push(LlmContextEvidenceEntry {
            entry_type: entry_type.to_string(),
            label: resolve_job_feedback_label(state, &feedback.job_id).await,
        });
    }

    entries
}

pub(super) async fn build_search_quality_summary(
    state: &AppState,
    raw_text: &str,
) -> Result<SearchQualitySummaryResponse, ApiError> {
    let analyzed_profile = state.profile_analysis.analyze(raw_text);
    let search_profile = state
        .search_profile_builder
        .build(&analyzed_profile, &SearchPreferences::default());
    let jobs = state
        .jobs_service
        .list_filtered_views(200, Some("active"), None)
        .await
        .map_err(|error| ApiError::from_repository(error, "jobs_query_failed"))?;
    let ranked_jobs = state.search_ranking.run(&search_profile, jobs).ranked_jobs;
    let quality = summarize_match_quality(&ranked_jobs);

    Ok(SearchQualitySummaryResponse {
        low_evidence_jobs: quality.low_evidence_jobs,
        weak_description_jobs: quality.weak_description_jobs,
        role_mismatch_jobs: quality.role_mismatch_jobs,
        seniority_mismatch_jobs: quality.seniority_mismatch_jobs,
        source_mismatch_jobs: quality.source_mismatch_jobs,
        top_missing_signals: quality.top_missing_signals,
    })
}

async fn resolve_job_feedback_label(state: &AppState, job_id: &str) -> String {
    match state.jobs_service.get_view_by_id(job_id).await {
        Ok(Some(job_view)) => format_job_feedback_label(
            job_view.job.title.as_str(),
            job_view.job.company_name.as_str(),
        )
        .unwrap_or_else(|| job_id.to_string()),
        Ok(None) => job_id.to_string(),
        Err(error) => {
            warn!(
                error = %error,
                job_id,
                "failed to resolve job feedback label; falling back to job id"
            );
            job_id.to_string()
        }
    }
}

fn format_job_feedback_label(title: &str, company_name: &str) -> Option<String> {
    let title = title.trim();
    let company_name = company_name.trim();

    if !title.is_empty() && !company_name.is_empty() {
        return Some(format!("{title} at {company_name}"));
    }
    if !title.is_empty() {
        return Some(title.to_string());
    }
    if !company_name.is_empty() {
        return Some(company_name.to_string());
    }

    None
}
