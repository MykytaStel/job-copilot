use serde_json::json;

use crate::domain::job::presentation::JobTextQuality;
use crate::domain::matching::{JobFit, JobScoreBreakdown, MissingSignalDetail};
use crate::domain::role::RoleId;
use crate::services::matching::{RankedJob, SearchRunResult};

use super::{
    JobFitResponse, RunSearchRequest, RunSearchResponse, SearchProfileRequest,
    SearchRerankerComparisonRequest, SearchRoleCandidateRequest,
};

#[test]
fn validates_run_search_request() {
    let input = RunSearchRequest {
        profile_id: None,
        search_profile: SearchProfileRequest {
            primary_role: "backend_engineer".to_string(),
            primary_role_confidence: Some(92),
            target_roles: vec!["devops_engineer".to_string()],
            role_candidates: vec![
                SearchRoleCandidateRequest {
                    role: "backend_engineer".to_string(),
                    confidence: 92,
                },
                SearchRoleCandidateRequest {
                    role: "devops_engineer".to_string(),
                    confidence: 61,
                },
            ],
            seniority: "senior".to_string(),
            target_regions: vec![],
            work_modes: vec![],
            allowed_sources: vec!["djinni".to_string()],
            profile_skills: vec!["rust".to_string(), "postgres".to_string()],
            profile_keywords: vec!["backend".to_string()],
            search_terms: vec!["rust".to_string()],
            exclude_terms: vec![],
        },
        limit: Some(25),
        reranker_comparison: Some(SearchRerankerComparisonRequest { top_n: Some(8) }),
    }
    .validate()
    .expect("request should validate");

    assert_eq!(input.limit, 25);
    assert_eq!(
        input
            .reranker_comparison
            .as_ref()
            .expect("comparison input should be present")
            .top_n,
        8
    );
    assert_eq!(input.search_profile.primary_role, RoleId::BackendEngineer);
    assert_eq!(input.search_profile.primary_role_confidence, Some(92));
    assert!(
        input
            .search_profile
            .target_roles
            .contains(&RoleId::DevopsEngineer)
    );
    assert_eq!(
        input.search_profile.role_candidates[1].role,
        RoleId::DevopsEngineer
    );
}

#[test]
fn rejects_unknown_primary_role() {
    let error = RunSearchRequest {
        profile_id: None,
        search_profile: SearchProfileRequest {
            primary_role: "wizard".to_string(),
            primary_role_confidence: None,
            target_roles: vec![],
            role_candidates: vec![],
            seniority: String::new(),
            target_regions: vec![],
            work_modes: vec![],
            allowed_sources: vec![],
            profile_skills: vec![],
            profile_keywords: vec![],
            search_terms: vec![],
            exclude_terms: vec![],
        },
        limit: None,
        reranker_comparison: None,
    }
    .validate()
    .expect_err("request should reject unknown role");

    let response = axum::response::IntoResponse::into_response(error);
    assert_eq!(response.status(), axum::http::StatusCode::BAD_REQUEST);
}

#[test]
fn serializes_fit_response_with_reasons() {
    let fit = JobFit {
        job_id: "job-1".to_string(),
        score: 88,
        score_breakdown: JobScoreBreakdown::deterministic(88),
        matched_roles: vec![RoleId::BackendEngineer],
        matched_skills: vec!["rust".to_string()],
        matched_keywords: vec!["platform".to_string()],
        source_match: true,
        work_mode_match: Some(true),
        region_match: Some(true),
        missing_signals: vec!["graphql".to_string()],
        missing_signal_details: vec![MissingSignalDetail {
            term: "graphql".to_string(),
            category: "profile_skill".to_string(),
        }],
        description_quality: JobTextQuality::Strong,
        reasons: vec!["Matched target roles: backend_engineer".to_string()],
    };

    let response =
        serde_json::to_value(JobFitResponse::from(fit)).expect("fit response should serialize");

    assert_eq!(response["score"], json!(88));
    assert_eq!(response["score_breakdown"]["total_score"], json!(88));
    assert_eq!(
        response["score_breakdown"]["reranker_mode"],
        json!("deterministic")
    );
    assert_eq!(response["missing_signals"], json!(["graphql"]));
    assert_eq!(
        response["missing_signal_details"],
        json!([{ "term": "graphql", "category": "profile_skill" }])
    );
    assert_eq!(response["description_quality"], json!("strong"));
    assert_eq!(
        response["reasons"],
        json!(["Matched target roles: backend_engineer"])
    );
}

#[test]
fn run_search_response_reports_meta() {
    let response = RunSearchResponse::from_result(SearchRunResult {
        ranked_jobs: Vec::<RankedJob>::new(),
        total_candidates: 10,
        filtered_out_by_source: 3,
        filtered_out_hidden: 2,
        filtered_out_company_blacklist: 1,
    });

    let payload = serde_json::to_value(response).expect("run search response should serialize");

    assert_eq!(payload["meta"]["total_candidates"], json!(10));
    assert_eq!(payload["meta"]["filtered_out_by_source"], json!(3));
    assert_eq!(payload["meta"]["filtered_out_hidden"], json!(2));
    assert_eq!(payload["meta"]["filtered_out_company_blacklist"], json!(1));
    assert_eq!(payload["meta"]["scored_jobs"], json!(4));
    assert_eq!(payload["meta"]["returned_jobs"], json!(0));
    assert_eq!(
        payload["meta"]["reranker_mode_requested"],
        json!("deterministic")
    );
    assert_eq!(
        payload["meta"]["reranker_mode_active"],
        json!("deterministic")
    );
    assert_eq!(payload["meta"].get("reranker_fallback_reason"), None);
    assert_eq!(payload["meta"].get("reranker_comparison"), None);
}
