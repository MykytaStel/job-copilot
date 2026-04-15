use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::api::dto::jobs::JobResponse;
use crate::api::error::ApiError;
use crate::domain::job::model::Job;
use crate::domain::matching::JobFit;
use crate::domain::role::RoleId;
use crate::domain::role::catalog::ROLE_CATALOG;
use crate::domain::search::profile::{SearchProfile, SearchRoleCandidate, TargetRegion, WorkMode};
use crate::domain::source::SOURCE_CATALOG;
use crate::domain::source::SourceId;
use crate::services::matching::SearchRunResult;

#[derive(Debug, Serialize)]
pub struct SearchResponse {
    pub jobs: Vec<JobResponse>,
    pub contacts: Vec<SearchContactResponse>,
    pub page: i64,
    pub per_page: i64,
    pub has_more: bool,
}

#[derive(Debug, Serialize)]
pub struct SearchContactResponse {
    pub id: String,
    pub name: String,
    pub role: Option<String>,
    pub email: Option<String>,
}

impl SearchResponse {
    pub fn from_jobs(jobs: Vec<Job>, page: i64, per_page: i64, has_more: bool) -> Self {
        Self {
            jobs: jobs.into_iter().map(JobResponse::from).collect(),
            contacts: Vec::new(),
            page,
            per_page,
            has_more,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct RunSearchRequest {
    #[serde(default)]
    pub profile_id: Option<String>,
    pub search_profile: SearchProfileRequest,
    pub limit: Option<i64>,
}

#[derive(Debug)]
pub struct RunSearchInput {
    pub profile_id: Option<String>,
    pub search_profile: SearchProfile,
    pub limit: i64,
}

#[derive(Debug, Deserialize)]
pub struct SearchRoleCandidateRequest {
    pub role: String,
    pub confidence: u8,
}

#[derive(Debug, Deserialize)]
pub struct SearchProfileRequest {
    pub primary_role: String,
    #[serde(default)]
    pub primary_role_confidence: Option<u8>,
    #[serde(default)]
    pub target_roles: Vec<String>,
    #[serde(default)]
    pub role_candidates: Vec<SearchRoleCandidateRequest>,
    #[serde(default)]
    pub seniority: String,
    #[serde(default)]
    pub target_regions: Vec<TargetRegion>,
    #[serde(default)]
    pub work_modes: Vec<WorkMode>,
    #[serde(default)]
    pub allowed_sources: Vec<String>,
    #[serde(default)]
    pub profile_skills: Vec<String>,
    #[serde(default)]
    pub profile_keywords: Vec<String>,
    #[serde(default)]
    pub search_terms: Vec<String>,
    #[serde(default)]
    pub exclude_terms: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct RunSearchResponse {
    pub results: Vec<RankedJobResponse>,
    pub meta: SearchRunMetaResponse,
}

#[derive(Debug, Serialize)]
pub struct RankedJobResponse {
    pub job: JobResponse,
    pub fit: JobFitResponse,
}

#[derive(Debug, Serialize)]
pub struct JobFitResponse {
    pub job_id: String,
    pub score: u8,
    pub matched_roles: Vec<String>,
    pub matched_skills: Vec<String>,
    pub matched_keywords: Vec<String>,
    pub source_match: bool,
    pub work_mode_match: Option<bool>,
    pub region_match: Option<bool>,
    pub reasons: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct SearchRunMetaResponse {
    pub total_candidates: usize,
    pub filtered_out_by_source: usize,
    pub filtered_out_hidden: usize,
    pub filtered_out_company_blacklist: usize,
    pub scored_jobs: usize,
    pub returned_jobs: usize,
    pub learned_reranker_enabled: bool,
    pub learned_reranker_adjusted_jobs: usize,
}

impl RunSearchRequest {
    pub fn validate(self) -> Result<RunSearchInput, ApiError> {
        let limit = self.limit.unwrap_or(20);

        if !(1..=100).contains(&limit) {
            return Err(ApiError::invalid_limit(limit));
        }

        Ok(RunSearchInput {
            profile_id: self
                .profile_id
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty()),
            search_profile: self.search_profile.validate()?,
            limit,
        })
    }
}

impl SearchProfileRequest {
    pub fn validate(self) -> Result<SearchProfile, ApiError> {
        let primary_role =
            RoleId::parse_canonical_key(self.primary_role.trim()).ok_or_else(|| {
                ApiError::bad_request_with_details(
                    "invalid_primary_role",
                    "Unknown primary_role value",
                    json!({
                        "field": "primary_role",
                        "invalid_value": self.primary_role,
                        "allowed_values": ROLE_CATALOG
                            .iter()
                            .map(|role| role.canonical_key)
                            .collect::<Vec<_>>(),
                    }),
                )
            })?;

        let target_roles = validate_roles("target_roles", self.target_roles)?;
        let role_candidates = validate_role_candidates(self.role_candidates)?;
        let allowed_sources = validate_sources(self.allowed_sources)?;

        Ok(SearchProfile {
            primary_role,
            primary_role_confidence: self.primary_role_confidence.or_else(|| {
                role_candidates
                    .iter()
                    .find(|candidate| candidate.role == primary_role)
                    .map(|candidate| candidate.confidence)
            }),
            target_roles: dedupe_roles(target_roles, primary_role),
            role_candidates,
            seniority: self.seniority.trim().to_string(),
            target_regions: self.target_regions,
            work_modes: self.work_modes,
            allowed_sources,
            profile_skills: normalize_string_list(self.profile_skills),
            profile_keywords: normalize_string_list(self.profile_keywords),
            search_terms: normalize_string_list(self.search_terms),
            exclude_terms: normalize_string_list(self.exclude_terms),
        })
    }
}

impl RunSearchResponse {
    pub fn from_result(result: SearchRunResult) -> Self {
        let scored_jobs = result
            .total_candidates
            .saturating_sub(result.filtered_out_by_source)
            .saturating_sub(result.filtered_out_hidden)
            .saturating_sub(result.filtered_out_company_blacklist);
        let results = result
            .ranked_jobs
            .into_iter()
            .map(RankedJobResponse::from)
            .collect::<Vec<_>>();
        let returned_jobs = results.len();

        Self {
            results,
            meta: SearchRunMetaResponse {
                total_candidates: result.total_candidates,
                filtered_out_by_source: result.filtered_out_by_source,
                filtered_out_hidden: result.filtered_out_hidden,
                filtered_out_company_blacklist: result.filtered_out_company_blacklist,
                scored_jobs,
                returned_jobs,
                learned_reranker_enabled: false,
                learned_reranker_adjusted_jobs: 0,
            },
        }
    }
}

impl From<crate::services::matching::RankedJob> for RankedJobResponse {
    fn from(value: crate::services::matching::RankedJob) -> Self {
        Self {
            job: JobResponse::from(value.job),
            fit: JobFitResponse::from(value.fit),
        }
    }
}

impl From<JobFit> for JobFitResponse {
    fn from(value: JobFit) -> Self {
        Self {
            job_id: value.job_id,
            score: value.score,
            matched_roles: value
                .matched_roles
                .into_iter()
                .map(|role| role.to_string())
                .collect(),
            matched_skills: value.matched_skills,
            matched_keywords: value.matched_keywords,
            source_match: value.source_match,
            work_mode_match: value.work_mode_match,
            region_match: value.region_match,
            reasons: value.reasons,
        }
    }
}

fn validate_roles(field: &'static str, roles: Vec<String>) -> Result<Vec<RoleId>, ApiError> {
    let mut validated = Vec::new();
    let mut invalid = Vec::new();

    for role in roles {
        match RoleId::parse_canonical_key(role.trim()) {
            Some(role_id) => push_unique_role(&mut validated, role_id),
            None => push_unique_string(&mut invalid, role),
        }
    }

    if invalid.is_empty() {
        Ok(validated)
    } else {
        Err(ApiError::bad_request_with_details(
            "invalid_target_roles",
            "Unknown target_roles values",
            json!({
                "field": field,
                "invalid_values": invalid,
                "allowed_values": ROLE_CATALOG
                    .iter()
                    .map(|role| role.canonical_key)
                    .collect::<Vec<_>>(),
            }),
        ))
    }
}

fn validate_role_candidates(
    candidates: Vec<SearchRoleCandidateRequest>,
) -> Result<Vec<SearchRoleCandidate>, ApiError> {
    let mut validated = Vec::new();
    let mut invalid = Vec::new();

    for candidate in candidates {
        match RoleId::parse_canonical_key(candidate.role.trim()) {
            Some(role) => {
                if !validated
                    .iter()
                    .any(|existing: &SearchRoleCandidate| existing.role == role)
                {
                    validated.push(SearchRoleCandidate {
                        role,
                        confidence: candidate.confidence,
                    });
                }
            }
            None => push_unique_string(&mut invalid, candidate.role),
        }
    }

    if invalid.is_empty() {
        Ok(validated)
    } else {
        Err(ApiError::bad_request_with_details(
            "invalid_role_candidates",
            "Unknown role_candidates values",
            json!({
                "field": "role_candidates",
                "invalid_values": invalid,
                "allowed_values": ROLE_CATALOG
                    .iter()
                    .map(|role| role.canonical_key)
                    .collect::<Vec<_>>(),
            }),
        ))
    }
}

fn validate_sources(sources: Vec<String>) -> Result<Vec<SourceId>, ApiError> {
    let mut validated = Vec::new();
    let mut invalid = Vec::new();

    for source in sources {
        match SourceId::parse_canonical_key(source.trim()) {
            Some(source_id) => push_unique_source(&mut validated, source_id),
            None => push_unique_string(&mut invalid, source),
        }
    }

    if invalid.is_empty() {
        Ok(validated)
    } else {
        Err(ApiError::bad_request_with_details(
            "invalid_allowed_sources",
            "Unknown allowed_sources values",
            json!({
                "field": "allowed_sources",
                "invalid_values": invalid,
                "allowed_values": SOURCE_CATALOG
                    .iter()
                    .map(|source| source.canonical_key)
                    .collect::<Vec<_>>(),
            }),
        ))
    }
}

fn dedupe_roles(mut target_roles: Vec<RoleId>, primary_role: RoleId) -> Vec<RoleId> {
    if !target_roles.contains(&primary_role) {
        target_roles.insert(0, primary_role);
    }

    target_roles
}

fn normalize_string_list(values: Vec<String>) -> Vec<String> {
    let mut normalized = Vec::new();

    for value in values {
        let value = value.trim().to_string();

        if value.is_empty() || normalized.iter().any(|existing| existing == &value) {
            continue;
        }

        normalized.push(value);
    }

    normalized
}

fn push_unique_role(target: &mut Vec<RoleId>, value: RoleId) {
    if !target.contains(&value) {
        target.push(value);
    }
}

fn push_unique_source(target: &mut Vec<SourceId>, value: SourceId) {
    if !target.contains(&value) {
        target.push(value);
    }
}

fn push_unique_string(target: &mut Vec<String>, value: String) {
    if !target.iter().any(|existing| existing == &value) {
        target.push(value);
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::domain::matching::JobFit;
    use crate::domain::role::RoleId;
    use crate::services::matching::{RankedJob, SearchRunResult};

    use super::{JobFitResponse, RunSearchRequest, RunSearchResponse, SearchProfileRequest};

    #[test]
    fn validates_run_search_request() {
        let input = RunSearchRequest {
            profile_id: None,
            search_profile: SearchProfileRequest {
                primary_role: "backend_developer".to_string(),
                primary_role_confidence: Some(92),
                target_roles: vec!["devops_engineer".to_string()],
                role_candidates: vec![
                    super::SearchRoleCandidateRequest {
                        role: "backend_developer".to_string(),
                        confidence: 92,
                    },
                    super::SearchRoleCandidateRequest {
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
        }
        .validate()
        .expect("request should validate");

        assert_eq!(input.limit, 25);
        assert_eq!(input.search_profile.primary_role, RoleId::BackendDeveloper);
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
            matched_roles: vec![RoleId::BackendDeveloper],
            matched_skills: vec!["rust".to_string()],
            matched_keywords: vec!["platform".to_string()],
            source_match: true,
            work_mode_match: Some(true),
            region_match: Some(true),
            reasons: vec!["Matched target roles: backend_developer".to_string()],
        };

        let response =
            serde_json::to_value(JobFitResponse::from(fit)).expect("fit response should serialize");

        assert_eq!(response["score"], json!(88));
        assert_eq!(
            response["reasons"],
            json!(["Matched target roles: backend_developer"])
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
    }
}
