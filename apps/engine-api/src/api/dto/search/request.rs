use serde::Deserialize;
use serde_json::json;

use crate::api::error::ApiError;
use crate::domain::role::RoleId;
use crate::domain::role::catalog::ROLE_CATALOG;
use crate::domain::search::profile::{SearchProfile, SearchRoleCandidate, TargetRegion, WorkMode};
use crate::domain::source::SOURCE_CATALOG;
use crate::domain::source::SourceId;

#[derive(Debug, Deserialize)]
pub struct RunSearchRequest {
    #[serde(default)]
    pub profile_id: Option<String>,
    pub search_profile: SearchProfileRequest,
    pub limit: Option<i64>,
    #[serde(default)]
    pub reranker_comparison: Option<SearchRerankerComparisonRequest>,
}

#[derive(Debug)]
pub struct RunSearchInput {
    pub profile_id: Option<String>,
    pub search_profile: SearchProfile,
    pub limit: i64,
    pub reranker_comparison: Option<SearchRerankerComparisonInput>,
}

#[derive(Debug, Deserialize)]
pub struct SearchRerankerComparisonRequest {
    #[serde(default)]
    pub top_n: Option<i64>,
}

#[derive(Debug)]
pub struct SearchRerankerComparisonInput {
    pub top_n: usize,
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
            reranker_comparison: self
                .reranker_comparison
                .map(|comparison| comparison.validate(limit))
                .transpose()?,
        })
    }
}

impl SearchRerankerComparisonRequest {
    pub fn validate(self, limit: i64) -> Result<SearchRerankerComparisonInput, ApiError> {
        let top_n = self.top_n.unwrap_or(5);

        if !(1..=10).contains(&top_n) {
            return Err(ApiError::bad_request(
                "invalid_reranker_comparison_top_n",
                "reranker_comparison.top_n must be between 1 and 10",
            ));
        }

        Ok(SearchRerankerComparisonInput {
            top_n: top_n.min(limit) as usize,
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
            scoring_weights: Default::default(),
        })
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
