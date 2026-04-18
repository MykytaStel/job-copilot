use axum::extract::{Path, Query, State};
use serde::Deserialize;
use tracing::warn;

use crate::api::dto::jobs::{JobResponse, MlJobLifecycleResponse, RecentJobsResponse};
use crate::api::dto::ranking::FitScoreResponse;
use crate::api::error::ApiError;
use crate::api::routes::feedback::ensure_profile_exists;
use crate::domain::feedback::model::{CompanyFeedbackRecord, JobFeedbackRecord, JobFeedbackState};
use crate::domain::source::SourceId;
use crate::services::feedback::FeedbackService;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct RecentJobsQuery {
    pub limit: Option<i64>,
    /// Filter by lifecycle stage: "active" | "inactive" | "reactivated"
    pub lifecycle: Option<String>,
    /// Filter by source name: "djinni" | "work_ua" | "robota_ua"
    pub source: Option<SourceId>,
    pub profile_id: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
pub struct JobContextQuery {
    pub profile_id: Option<String>,
}

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

    Ok(axum::Json(JobResponse::from_view_with_feedback(
        job, feedback,
    )))
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
    let limit = query.limit.unwrap_or(50);

    if !(1..=200).contains(&limit) {
        return Err(ApiError::invalid_limit(limit));
    }

    let lifecycle = query.lifecycle.as_deref();
    let source = query.source.map(SourceId::canonical_key);
    let profile_id = query.profile_id.as_deref();

    let jobs = state
        .jobs_service
        .list_filtered_views(limit, lifecycle, source)
        .await
        .map_err(|error| ApiError::from_repository(error, "jobs_query_failed"))?;
    let feedback_states = load_feedback_state(&state, profile_id, &jobs).await?;
    let jobs = jobs
        .into_iter()
        .zip(feedback_states.into_iter())
        .filter(|(_, feedback)| !feedback.hidden)
        .map(|(job, feedback)| JobResponse::from_view_with_feedback(job, feedback))
        .collect();
    let summary = state
        .jobs_service
        .feed_summary()
        .await
        .map_err(|error| ApiError::from_repository(error, "jobs_query_failed"))?;

    Ok(axum::Json(RecentJobsResponse {
        jobs,
        summary: summary.into(),
    }))
}

/// Read the persisted fit score for a job (previously computed via GET /fit or POST /match).
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
        .collect::<std::collections::HashMap<_, _>>();
    let company_feedback_by_name = company_feedback
        .into_iter()
        .map(|record| (record.normalized_company_name.clone(), record))
        .collect::<std::collections::HashMap<_, _>>();

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

/// Return a fit score for a job against the active resume.
///
/// Cache-first: returns the persisted score when one exists so repeat calls are
/// instant.  On a cache miss the score is computed locally (no API call), then
/// persisted for subsequent requests.
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

    // Return persisted score when available — the common path after the first visit.
    if let Ok(Some(cached)) = state
        .fit_scores_repository
        .get_for_job_and_resume(&job_id, &resume.id)
        .await
    {
        return Ok(axum::Json(FitScoreResponse::from(cached)));
    }

    // Cache miss: fetch the job, compute, persist, then return.
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

    let candidate = state.profile_analysis_service.analyze(&resume.raw_text);
    // Best-effort: load stored profile for salary/work-mode prefs.
    // If absent, those components default to neutral (0.5).
    let profile = state.profiles_service.get_latest().await.ok().flatten();

    let score = state
        .ranking_service
        .compute(&candidate, &job, profile.as_ref());

    if let Err(error) = state.fit_scores_repository.upsert(&score, &resume.id).await {
        warn!(job_id = %job_id, resume_id = %resume.id, error = %error, "failed to persist fit score");
    }

    Ok(axum::Json(FitScoreResponse::from(score)))
}

/// Force-recompute and persist a fit score for a job (same as GET /fit but via POST).
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

    let candidate = state.profile_analysis_service.analyze(&resume.raw_text);
    let profile = state.profiles_service.get_latest().await.ok().flatten();
    let score = state
        .ranking_service
        .compute(&candidate, &job, profile.as_ref());

    if let Err(error) = state.fit_scores_repository.upsert(&score, &resume.id).await {
        warn!(job_id = %job_id, resume_id = %resume.id, error = %error, "failed to persist fit score");
    }

    Ok(axum::Json(FitScoreResponse::from(score)))
}

#[cfg(test)]
mod tests {
    use axum::Json;
    use axum::extract::{Path, Query, State};
    use axum::http::Uri;
    use axum::response::IntoResponse;
    use axum::{body, http::StatusCode};
    use serde_json::{Value, json};

    use crate::domain::job::model::{
        Job, JobFeedSummary, JobLifecycleStage, JobSourceVariant, JobView,
    };
    use crate::domain::source::SourceId;
    use crate::services::applications::{ApplicationsService, ApplicationsServiceStub};
    use crate::services::jobs::{JobsService, JobsServiceStub};
    use crate::services::profiles::{ProfilesService, ProfilesServiceStub};
    use crate::services::resumes::{ResumesService, ResumesServiceStub};
    use crate::state::AppState;

    use super::{
        JobContextQuery, RecentJobsQuery, get_job_by_id, get_ml_job_lifecycle, get_recent_jobs,
    };

    fn sample_job_view(id: &str) -> JobView {
        JobView {
            job: Job {
                id: id.to_string(),
                title: "Platform Engineer".to_string(),
                company_name: "SignalHire".to_string(),
                location: Some("Remote, Europe".to_string()),
                remote_type: Some("remote".to_string()),
                seniority: Some("senior".to_string()),
                description_text: "Rust and Postgres".to_string(),
                salary_min: Some(5000),
                salary_max: Some(6500),
                salary_currency: Some("USD".to_string()),
                posted_at: Some("2026-04-14T08:00:00Z".to_string()),
                last_seen_at: "2026-04-16T09:00:00Z".to_string(),
                is_active: true,
            },
            first_seen_at: "2026-04-14T08:00:00Z".to_string(),
            inactivated_at: None,
            reactivated_at: Some("2026-04-16T09:00:00Z".to_string()),
            lifecycle_stage: JobLifecycleStage::Reactivated,
            primary_variant: Some(JobSourceVariant {
                source: "mock_source".to_string(),
                source_job_id: "platform-001".to_string(),
                source_url: "https://mock-source.example/jobs/platform-001".to_string(),
                raw_payload: None,
                fetched_at: "2026-04-16T09:00:00Z".to_string(),
                last_seen_at: "2026-04-16T09:00:00Z".to_string(),
                is_active: true,
                inactivated_at: None,
            }),
        }
    }

    #[tokio::test]
    async fn returns_service_unavailable_when_database_is_missing() {
        let result = get_job_by_id(
            State(AppState::without_database()),
            Path("job-123".to_string()),
            Query(JobContextQuery::default()),
        )
        .await;

        let response = match result {
            Ok(_) => panic!("handler should fail without a configured database"),
            Err(error) => error.into_response(),
        };

        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    async fn returns_not_found_for_unknown_job() {
        let state = AppState::for_services(
            ProfilesService::for_tests(ProfilesServiceStub::default()),
            JobsService::for_tests(JobsServiceStub::default()),
            ApplicationsService::for_tests(ApplicationsServiceStub::default()),
            ResumesService::for_tests(ResumesServiceStub::default()),
        );
        let result = get_job_by_id(
            State(state),
            Path("missing-job".to_string()),
            Query(JobContextQuery::default()),
        )
        .await;

        let response = match result {
            Ok(_) => panic!("handler should return not found for unknown job"),
            Err(error) => error.into_response(),
        };

        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        let body = body::to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("response body should be readable");
        let payload: Value =
            serde_json::from_slice(&body).expect("response body should be valid json");

        assert_eq!(payload["code"], json!("job_not_found"));
    }

    #[tokio::test]
    async fn rejects_invalid_recent_jobs_limit() {
        let result = get_recent_jobs(
            State(AppState::without_database()),
            Query(RecentJobsQuery {
                limit: Some(0),
                lifecycle: None,
                source: None,
                profile_id: None,
            }),
        )
        .await;

        let response = match result {
            Ok(Json(_)) => panic!("handler should reject invalid limit"),
            Err(error) => error.into_response(),
        };

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = body::to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("response body should be readable");
        let payload: Value =
            serde_json::from_slice(&body).expect("response body should be valid json");

        assert_eq!(payload["code"], json!("invalid_limit"));
    }

    #[tokio::test]
    async fn returns_job_feed_summary_and_lifecycle_fields() {
        let state = AppState::for_services(
            ProfilesService::for_tests(ProfilesServiceStub::default()),
            JobsService::for_tests(
                JobsServiceStub::default()
                    .with_job_view(sample_job_view("job-123"))
                    .with_feed_summary(JobFeedSummary {
                        total_jobs: 3,
                        active_jobs: 2,
                        inactive_jobs: 1,
                        reactivated_jobs: 1,
                    }),
            ),
            ApplicationsService::for_tests(ApplicationsServiceStub::default()),
            ResumesService::for_tests(ResumesServiceStub::default()),
        );

        let response = get_recent_jobs(
            State(state),
            Query(RecentJobsQuery {
                limit: Some(20),
                lifecycle: None,
                source: None,
                profile_id: None,
            }),
        )
        .await
        .expect("recent jobs should succeed")
        .into_response();

        let body = body::to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("response body should be readable");
        let payload: Value =
            serde_json::from_slice(&body).expect("response body should be valid JSON");

        assert_eq!(payload["summary"]["reactivated_jobs"], json!(1));
        assert_eq!(payload["jobs"][0]["lifecycle_stage"], json!("reactivated"));
        assert_eq!(payload["jobs"][0]["location"], json!("Remote, Europe"));
        assert_eq!(
            payload["jobs"][0]["primary_variant"]["source"],
            json!("mock_source")
        );
        assert_eq!(
            payload["jobs"][0]["presentation"]["salary_label"],
            json!("5,000-6,500 USD")
        );
    }

    #[tokio::test]
    async fn returns_ml_lifecycle_payload() {
        let state = AppState::for_services(
            ProfilesService::for_tests(ProfilesServiceStub::default()),
            JobsService::for_tests(
                JobsServiceStub::default().with_job_view(sample_job_view("job-123")),
            ),
            ApplicationsService::for_tests(ApplicationsServiceStub::default()),
            ResumesService::for_tests(ResumesServiceStub::default()),
        );

        let response = get_ml_job_lifecycle(State(state), Path("job-123".to_string()))
            .await
            .expect("ML lifecycle route should succeed")
            .into_response();

        let body = body::to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("response body should be readable");
        let payload: Value =
            serde_json::from_slice(&body).expect("response body should be valid JSON");

        assert_eq!(payload["id"], json!("job-123"));
        assert_eq!(payload["lifecycle_stage"], json!("reactivated"));
        assert_eq!(payload["location"], json!("Remote, Europe"));
        assert_eq!(
            payload["primary_variant"]["source_url"],
            json!("https://mock-source.example/jobs/platform-001")
        );
        assert_eq!(payload["presentation"]["location_label"], json!("Europe"));
    }

    #[test]
    fn recent_jobs_query_accepts_known_source() {
        let uri: Uri = "/api/v1/jobs/recent?source=djinni"
            .parse()
            .expect("uri should parse");
        let Query(query) =
            Query::<RecentJobsQuery>::try_from_uri(&uri).expect("query should deserialize");

        assert_eq!(query.source, Some(SourceId::Djinni));
    }

    #[test]
    fn recent_jobs_query_rejects_unknown_source() {
        let uri: Uri = "/api/v1/jobs/recent?source=linkedin"
            .parse()
            .expect("uri should parse");

        let result = Query::<RecentJobsQuery>::try_from_uri(&uri);

        assert!(result.is_err());
    }
}
