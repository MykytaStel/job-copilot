use axum::extract::{Path, State};

use crate::api::dto::analytics::{
    AnalyticsSummaryResponse, FeedbackSummarySection, JobsByLifecycleSection, JobsBySourceEntry,
    LlmContextAnalyzedProfile, LlmContextEvidenceEntry, LlmContextResponse,
    SalaryIntelligenceResponse,
};
use crate::api::error::ApiError;
use crate::api::routes::feedback::ensure_profile_exists;
use crate::domain::feedback::model::CompanyFeedbackStatus;
use crate::state::AppState;

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
    Path(profile_id): Path<String>,
) -> Result<axum::Json<AnalyticsSummaryResponse>, ApiError> {
    ensure_profile_exists(&state, &profile_id).await?;

    let profile = state
        .profiles_service
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
    }))
}

pub async fn get_llm_context(
    State(state): State<AppState>,
    Path(profile_id): Path<String>,
) -> Result<axum::Json<LlmContextResponse>, ApiError> {
    ensure_profile_exists(&state, &profile_id).await?;

    let profile = state
        .profiles_service
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

    let top_positive_evidence: Vec<LlmContextEvidenceEntry> = job_feedback
        .iter()
        .filter(|j| j.saved)
        .take(10)
        .map(|j| LlmContextEvidenceEntry {
            entry_type: "saved_job".to_string(),
            label: j.job_id.clone(),
        })
        .chain(
            company_feedback
                .iter()
                .filter(|c| c.status == CompanyFeedbackStatus::Whitelist)
                .take(10)
                .map(|c| LlmContextEvidenceEntry {
                    entry_type: "whitelisted_company".to_string(),
                    label: c.company_name.clone(),
                }),
        )
        .collect();

    let top_negative_evidence: Vec<LlmContextEvidenceEntry> = job_feedback
        .iter()
        .filter(|j| j.bad_fit)
        .take(10)
        .map(|j| LlmContextEvidenceEntry {
            entry_type: "bad_fit_job".to_string(),
            label: j.job_id.clone(),
        })
        .chain(
            company_feedback
                .iter()
                .filter(|c| c.status == CompanyFeedbackStatus::Blacklist)
                .take(10)
                .map(|c| LlmContextEvidenceEntry {
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

#[cfg(test)]
mod tests {
    use axum::Json;
    use axum::extract::{Path, State};

    use crate::domain::analytics::model::JobSourceCount;
    use crate::domain::feedback::model::{
        CompanyFeedbackRecord, CompanyFeedbackStatus, JobFeedbackRecord,
    };
    use crate::domain::job::model::JobFeedSummary;
    use crate::domain::profile::model::{Profile, ProfileAnalysis};
    use crate::domain::role::RoleId;
    use crate::services::applications::{ApplicationsService, ApplicationsServiceStub};
    use crate::services::feedback::{FeedbackService, FeedbackServiceStub};
    use crate::services::jobs::{JobsService, JobsServiceStub};
    use crate::services::profiles::{ProfilesService, ProfilesServiceStub};
    use crate::services::resumes::{ResumesService, ResumesServiceStub};
    use crate::state::AppState;

    use super::{get_analytics_summary, get_llm_context};

    fn sample_profile_with_analysis() -> Profile {
        Profile {
            id: "profile-1".to_string(),
            name: "Jane".to_string(),
            email: "jane@example.com".to_string(),
            location: None,
            raw_text: "Senior Rust backend engineer".to_string(),
            analysis: Some(ProfileAnalysis {
                summary: "Senior backend engineer with Rust expertise".to_string(),
                primary_role: RoleId::BackendDeveloper,
                seniority: "senior".to_string(),
                skills: vec!["rust".to_string(), "postgres".to_string()],
                keywords: vec!["backend".to_string(), "distributed".to_string()],
            }),
            salary_min_usd: None,
            salary_max_usd: None,
            preferred_work_mode: None,
            created_at: "2026-04-14T00:00:00Z".to_string(),
            updated_at: "2026-04-14T00:00:00Z".to_string(),
            skills_updated_at: None,
        }
    }

    fn test_state() -> AppState {
        AppState::for_services(
            ProfilesService::for_tests(
                ProfilesServiceStub::default().with_profile(sample_profile_with_analysis()),
            ),
            JobsService::for_tests(
                JobsServiceStub::default()
                    .with_feed_summary(JobFeedSummary {
                        total_jobs: 10,
                        active_jobs: 6,
                        inactive_jobs: 3,
                        reactivated_jobs: 1,
                    })
                    .with_jobs_by_source(vec![
                        JobSourceCount {
                            source: "djinni".to_string(),
                            count: 7,
                        },
                        JobSourceCount {
                            source: "work_ua".to_string(),
                            count: 3,
                        },
                    ]),
            ),
            ApplicationsService::for_tests(ApplicationsServiceStub::default()),
            ResumesService::for_tests(ResumesServiceStub::default()),
        )
    }

    fn with_feedback(state: AppState) -> AppState {
        state.with_feedback_service(FeedbackService::for_tests(
            FeedbackServiceStub::default()
                .with_job_feedback(JobFeedbackRecord {
                    profile_id: "profile-1".to_string(),
                    job_id: "job-1".to_string(),
                    saved: true,
                    hidden: false,
                    bad_fit: false,
                    created_at: "2026-04-14T00:00:00Z".to_string(),
                    updated_at: "2026-04-14T00:00:00Z".to_string(),
                })
                .with_job_feedback(JobFeedbackRecord {
                    profile_id: "profile-1".to_string(),
                    job_id: "job-2".to_string(),
                    saved: false,
                    hidden: true,
                    bad_fit: true,
                    created_at: "2026-04-14T00:00:00Z".to_string(),
                    updated_at: "2026-04-14T00:00:00Z".to_string(),
                })
                .with_company_feedback(CompanyFeedbackRecord {
                    profile_id: "profile-1".to_string(),
                    company_name: "GoodCorp".to_string(),
                    normalized_company_name: "goodcorp".to_string(),
                    status: CompanyFeedbackStatus::Whitelist,
                    created_at: "2026-04-14T00:00:00Z".to_string(),
                    updated_at: "2026-04-14T00:00:00Z".to_string(),
                })
                .with_company_feedback(CompanyFeedbackRecord {
                    profile_id: "profile-1".to_string(),
                    company_name: "BadCorp".to_string(),
                    normalized_company_name: "badcorp".to_string(),
                    status: CompanyFeedbackStatus::Blacklist,
                    created_at: "2026-04-14T00:00:00Z".to_string(),
                    updated_at: "2026-04-14T00:00:00Z".to_string(),
                }),
        ))
    }

    // ─── analytics summary tests ──────────────────────────────────────────────

    #[tokio::test]
    async fn analytics_summary_feedback_counts_are_correct() {
        let state = with_feedback(test_state());

        let Json(summary) = get_analytics_summary(State(state), Path("profile-1".to_string()))
            .await
            .expect("analytics summary should succeed");

        assert_eq!(summary.feedback.saved_jobs_count, 1);
        assert_eq!(summary.feedback.hidden_jobs_count, 1);
        assert_eq!(summary.feedback.bad_fit_jobs_count, 1);
        assert_eq!(summary.feedback.whitelisted_companies_count, 1);
        assert_eq!(summary.feedback.blacklisted_companies_count, 1);
    }

    #[tokio::test]
    async fn analytics_summary_source_aggregation_is_correct() {
        let state = test_state();

        let Json(summary) = get_analytics_summary(State(state), Path("profile-1".to_string()))
            .await
            .expect("analytics summary should succeed");

        assert_eq!(summary.jobs_by_source.len(), 2);
        assert_eq!(summary.jobs_by_source[0].source, "djinni");
        assert_eq!(summary.jobs_by_source[0].count, 7);
        assert_eq!(summary.jobs_by_source[1].source, "work_ua");
        assert_eq!(summary.jobs_by_source[1].count, 3);
    }

    #[tokio::test]
    async fn analytics_summary_lifecycle_distribution_is_correct() {
        let state = test_state();

        let Json(summary) = get_analytics_summary(State(state), Path("profile-1".to_string()))
            .await
            .expect("analytics summary should succeed");

        assert_eq!(summary.jobs_by_lifecycle.total, 10);
        assert_eq!(summary.jobs_by_lifecycle.active, 6);
        assert_eq!(summary.jobs_by_lifecycle.inactive, 3);
        assert_eq!(summary.jobs_by_lifecycle.reactivated, 1);
    }

    #[tokio::test]
    async fn analytics_summary_top_matched_come_from_profile_analysis() {
        let state = test_state();

        let Json(summary) = get_analytics_summary(State(state), Path("profile-1".to_string()))
            .await
            .expect("analytics summary should succeed");

        assert_eq!(summary.top_matched_roles, vec!["backend_developer"]);
        assert_eq!(summary.top_matched_skills, vec!["rust", "postgres"]);
        assert_eq!(summary.top_matched_keywords, vec!["backend", "distributed"]);
    }

    // ─── LLM context tests ────────────────────────────────────────────────────

    #[tokio::test]
    async fn llm_context_payload_shape_is_complete() {
        let state = with_feedback(test_state());

        let Json(ctx) = get_llm_context(State(state), Path("profile-1".to_string()))
            .await
            .expect("llm context should succeed");

        assert_eq!(ctx.profile_id, "profile-1");
        assert!(ctx.analyzed_profile.is_some());
        assert_eq!(ctx.profile_skills, vec!["rust", "postgres"]);
        assert_eq!(ctx.profile_keywords, vec!["backend", "distributed"]);
        assert_eq!(ctx.jobs_feed_summary.total, 10);
        assert_eq!(ctx.feedback_summary.saved_jobs_count, 1);
    }

    #[tokio::test]
    async fn llm_context_positive_evidence_includes_saved_jobs_and_whitelisted_companies() {
        let state = with_feedback(test_state());

        let Json(ctx) = get_llm_context(State(state), Path("profile-1".to_string()))
            .await
            .expect("llm context should succeed");

        let saved = ctx
            .top_positive_evidence
            .iter()
            .find(|e| e.entry_type == "saved_job");
        let whitelisted = ctx
            .top_positive_evidence
            .iter()
            .find(|e| e.entry_type == "whitelisted_company");

        assert!(saved.is_some(), "should include saved job evidence");
        assert_eq!(saved.unwrap().label, "job-1");
        assert!(
            whitelisted.is_some(),
            "should include whitelisted company evidence"
        );
        assert_eq!(whitelisted.unwrap().label, "GoodCorp");
    }

    #[tokio::test]
    async fn llm_context_negative_evidence_includes_bad_fit_jobs_and_blacklisted_companies() {
        let state = with_feedback(test_state());

        let Json(ctx) = get_llm_context(State(state), Path("profile-1".to_string()))
            .await
            .expect("llm context should succeed");

        let bad_fit = ctx
            .top_negative_evidence
            .iter()
            .find(|e| e.entry_type == "bad_fit_job");
        let blacklisted = ctx
            .top_negative_evidence
            .iter()
            .find(|e| e.entry_type == "blacklisted_company");

        assert!(bad_fit.is_some(), "should include bad fit job evidence");
        assert_eq!(bad_fit.unwrap().label, "job-2");
        assert!(
            blacklisted.is_some(),
            "should include blacklisted company evidence"
        );
        assert_eq!(blacklisted.unwrap().label, "BadCorp");
    }

    #[tokio::test]
    async fn llm_context_analyzed_profile_fields_match_profile_analysis() {
        let state = test_state();

        let Json(ctx) = get_llm_context(State(state), Path("profile-1".to_string()))
            .await
            .expect("llm context should succeed");

        let analysis = ctx
            .analyzed_profile
            .expect("analyzed_profile should be present");
        assert_eq!(analysis.primary_role, "backend_developer");
        assert_eq!(analysis.seniority, "senior");
        assert!(analysis.skills.contains(&"rust".to_string()));
    }
}
