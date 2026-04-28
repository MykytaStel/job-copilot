use axum::Extension;
use axum::extract::State;
use serde::Deserialize;
use tracing::warn;

use crate::api::error::{ApiError, ApiJson};
use crate::api::middleware::auth::AuthUser;
use crate::services::cv_tailoring::{
    CvTailoringError, CvTailoringMlRequest, CvTailoringMlResponse,
};
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct TailorCvRequest {
    pub job_id: String,
}

pub async fn tailor_cv(
    State(state): State<AppState>,
    auth: Option<Extension<AuthUser>>,
    ApiJson(payload): ApiJson<TailorCvRequest>,
) -> Result<axum::Json<CvTailoringMlResponse>, ApiError> {
    let Some(Extension(auth)) = auth else {
        return Err(ApiError::unauthorized(
            "auth_required",
            "Authentication is required for CV tailoring",
        ));
    };

    let profile_id = auth.profile_id;
    let job_id = payload.job_id.trim().to_string();

    if job_id.is_empty() {
        return Err(ApiError::bad_request(
            "invalid_job_id",
            "job_id is required",
        ));
    }

    let Some(job_view) = state
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

    let Some(profile) = state
        .profile_records
        .get_by_id(&profile_id)
        .await
        .map_err(|error| ApiError::from_repository(error, "profiles_query_failed"))?
    else {
        return Err(ApiError::not_found(
            "profile_not_found",
            format!("Profile '{profile_id}' was not found"),
        ));
    };

    let analysis = profile.analysis.as_ref();

    let ml_request = CvTailoringMlRequest {
        profile_id: profile_id.clone(),
        job_id: job_id.clone(),
        profile_summary: analysis
            .map(|value| value.summary.clone())
            .unwrap_or_default(),
        candidate_skills: analysis
            .map(|value| value.skills.clone())
            .unwrap_or_default(),
        job_title: job_view.job.title.clone(),
        job_description: job_view.job.description_text.clone(),
        job_required_skills: Vec::new(),
        job_nice_to_have_skills: Vec::new(),
        candidate_cv_text: Some(profile.raw_text.clone()),
    };

    let ml_response = state
        .cv_tailoring
        .tailor(&ml_request)
        .await
        .map_err(|error| {
            warn!(
                profile_id,
                job_id,
                error = %error,
                "cv tailoring ml call failed"
            );

            match error {
                CvTailoringError::Http(_) => ApiError::bad_gateway(
                    "ml_sidecar_unavailable",
                    "CV tailoring service is currently unavailable",
                ),
                CvTailoringError::Upstream { .. } => ApiError::bad_gateway(
                    "ml_sidecar_error",
                    "CV tailoring service returned an error",
                ),
                CvTailoringError::Decode(_) => ApiError::bad_gateway(
                    "ml_sidecar_decode_error",
                    "CV tailoring service returned an unexpected response",
                ),
            }
        })?;

    Ok(axum::Json(ml_response))
}

#[cfg(test)]
mod tests {
    use axum::Extension;
    use axum::extract::State;
    use axum::http::StatusCode;
    use axum::response::IntoResponse;

    use super::{TailorCvRequest, tailor_cv};
    use crate::api::error::ApiJson;
    use crate::api::middleware::auth::AuthUser;
    use crate::domain::job::model::{Job, JobLifecycleStage, JobView};
    use crate::domain::profile::model::{
        LanguageLevel, LanguageProficiency, Profile, ProfileAnalysis,
    };
    use crate::domain::role::RoleId;
    use crate::services::applications::{ApplicationsService, ApplicationsServiceStub};
    use crate::services::cv_tailoring::{
        CvTailoringError, CvTailoringGapItem, CvTailoringMlResponse, CvTailoringService,
        CvTailoringServiceStub, CvTailoringSuggestions,
    };
    use crate::services::jobs::{JobsService, JobsServiceStub};
    use crate::services::profiles::{ProfilesService, ProfilesServiceStub};
    use crate::services::resumes::{ResumesService, ResumesServiceStub};
    use crate::state::AppState;

    fn state_with_cv_stub(stub: CvTailoringServiceStub) -> AppState {
        AppState::for_services(
            ProfilesService::for_tests(
                ProfilesServiceStub::default().with_profile(profile("profile_test_001")),
            ),
            JobsService::for_tests(JobsServiceStub::default()),
            ApplicationsService::for_tests(ApplicationsServiceStub::default()),
            ResumesService::for_tests(ResumesServiceStub::default()),
        )
        .with_cv_tailoring_service(CvTailoringService::for_tests(stub))
    }

    fn state_with_job_and_cv_stub(job_id: &str, stub: CvTailoringServiceStub) -> AppState {
        let job_view = job_view(job_id);

        AppState::for_services(
            ProfilesService::for_tests(
                ProfilesServiceStub::default().with_profile(profile("profile_test_001")),
            ),
            JobsService::for_tests(JobsServiceStub::default().with_job_view(job_view)),
            ApplicationsService::for_tests(ApplicationsServiceStub::default()),
            ResumesService::for_tests(ResumesServiceStub::default()),
        )
        .with_cv_tailoring_service(CvTailoringService::for_tests(stub))
    }

    fn auth(profile_id: &str) -> Option<Extension<AuthUser>> {
        Some(Extension(AuthUser {
            profile_id: profile_id.to_string(),
        }))
    }

    fn job_view(job_id: &str) -> JobView {
        JobView {
            job: Job {
                id: job_id.to_string(),
                title: "Senior Rust Engineer".to_string(),
                company_name: "Acme".to_string(),
                location: None,
                remote_type: Some("remote".to_string()),
                seniority: Some("senior".to_string()),
                description_text: "Build backend systems in Rust and Postgres.".to_string(),
                salary_min: None,
                salary_max: None,
                salary_currency: None,
                language: None,
                posted_at: None,
                last_seen_at: "2026-01-01".to_string(),
                is_active: true,
            },
            first_seen_at: "2026-01-01".to_string(),
            inactivated_at: None,
            reactivated_at: None,
            lifecycle_stage: JobLifecycleStage::Active,
            primary_variant: None,
        }
    }

    fn profile(profile_id: &str) -> Profile {
        Profile {
        id: profile_id.to_string(),
        name: "Test Candidate".to_string(),
        email: format!("{profile_id}@example.com"),
        location: Some("Kyiv, Ukraine".to_string()),
        raw_text: "Senior developer with Rust, TypeScript, React, Node.js and PostgreSQL experience."
            .to_string(),
        analysis: Some(ProfileAnalysis {
            summary: "Senior full-stack developer focused on backend systems and product engineering."
                .to_string(),
            primary_role: RoleId::FullstackEngineer,
            seniority: "senior".to_string(),
            skills: vec![
                "Rust".to_string(),
                "TypeScript".to_string(),
                "React".to_string(),
                "PostgreSQL".to_string(),
            ],
            keywords: vec![
                "backend".to_string(),
                "distributed systems".to_string(),
                "product engineering".to_string(),
            ],
        }),
        years_of_experience: Some(8),
        salary_min: None,
        salary_max: None,
        salary_currency: "USD".to_string(),
        languages: vec![
            LanguageProficiency {
                language: "English".to_string(),
                level: LanguageLevel::C1,
            },
            LanguageProficiency {
                language: "Ukrainian".to_string(),
                level: LanguageLevel::Native,
            },
        ],
        preferred_locations: vec![],
        work_mode_preference: "remote_only".to_string(),
        preferred_language: Some("en".to_string()),
        search_preferences: None,
        created_at: "2026-01-01T00:00:00Z".to_string(),
        updated_at: "2026-01-01T00:00:00Z".to_string(),
        skills_updated_at: Some("2026-01-01T00:00:00Z".to_string()),
    }
    }

    fn ok_response() -> CvTailoringMlResponse {
        CvTailoringMlResponse {
            suggestions: CvTailoringSuggestions {
                skills_to_highlight: vec!["Rust".to_string()],
                skills_to_mention: vec!["Postgres".to_string()],
                gaps_to_address: vec![CvTailoringGapItem {
                    skill: "AWS".to_string(),
                    suggestion: "Mention any cloud deployment or infrastructure experience."
                        .to_string(),
                }],
                summary_rewrite:
                    "Senior backend engineer with Rust and distributed systems experience."
                        .to_string(),
                key_phrases: vec!["distributed systems".to_string()],
            },
            provider: "template".to_string(),
            generated_at: "2026-01-01T00:00:00Z".to_string(),
        }
    }

    #[tokio::test]
    async fn returns_401_when_no_auth() {
        let state = state_with_cv_stub(CvTailoringServiceStub::default());

        let response = tailor_cv(
            State(state),
            None,
            ApiJson(TailorCvRequest {
                job_id: "job-1".to_string(),
            }),
        )
        .await
        .expect_err("should reject unauthenticated request")
        .into_response();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn returns_400_for_empty_job_id() {
        let state = state_with_cv_stub(CvTailoringServiceStub::default());

        let response = tailor_cv(
            State(state),
            auth("profile_test_001"),
            ApiJson(TailorCvRequest {
                job_id: "   ".to_string(),
            }),
        )
        .await
        .expect_err("should reject empty job_id")
        .into_response();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn returns_404_for_unknown_job() {
        let state = state_with_cv_stub(CvTailoringServiceStub::default());

        let response = tailor_cv(
            State(state),
            auth("profile_test_001"),
            ApiJson(TailorCvRequest {
                job_id: "nonexistent-job".to_string(),
            }),
        )
        .await
        .expect_err("should return 404 for unknown job")
        .into_response();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn returns_404_for_unknown_profile() {
        let state = state_with_job_and_cv_stub("job-1", CvTailoringServiceStub::default());

        let response = tailor_cv(
            State(state),
            auth("missing-profile"),
            ApiJson(TailorCvRequest {
                job_id: "job-1".to_string(),
            }),
        )
        .await
        .expect_err("should return 404 for unknown profile")
        .into_response();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn returns_502_when_ml_unavailable() {
        let state = state_with_job_and_cv_stub(
            "job-1",
            CvTailoringServiceStub::default().with_response(Err(CvTailoringError::Http(
                "connection refused".to_string(),
            ))),
        );

        let response = tailor_cv(
            State(state),
            auth("profile_test_001"),
            ApiJson(TailorCvRequest {
                job_id: "job-1".to_string(),
            }),
        )
        .await
        .expect_err("should return 502 when ML is unreachable")
        .into_response();

        assert_eq!(response.status(), StatusCode::BAD_GATEWAY);
    }

    #[tokio::test]
    async fn returns_suggestions_on_success() {
        let state = state_with_job_and_cv_stub(
            "job-2",
            CvTailoringServiceStub::default().with_response(Ok(ok_response())),
        );

        let result = tailor_cv(
            State(state),
            auth("profile_test_001"),
            ApiJson(TailorCvRequest {
                job_id: "job-2".to_string(),
            }),
        )
        .await
        .expect("should return suggestions on success");

        assert_eq!(result.suggestions.skills_to_highlight, vec!["Rust"]);
        assert_eq!(result.provider, "template");
    }
}
