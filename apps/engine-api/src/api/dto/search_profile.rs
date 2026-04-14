use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::api::error::ApiError;
use crate::domain::role::RoleId;
use crate::domain::role::catalog::ROLE_CATALOG;
use crate::domain::search::profile::{SearchPreferences, SearchProfile, TargetRegion, WorkMode};
use crate::domain::source::SOURCE_CATALOG;
use crate::domain::source::SourceId;

use super::profile::AnalyzeProfileResponse;

#[derive(Default, Deserialize)]
pub struct SearchPreferencesRequest {
    #[serde(default)]
    pub target_regions: Vec<TargetRegion>,
    #[serde(default)]
    pub work_modes: Vec<WorkMode>,
    #[serde(default)]
    pub preferred_roles: Vec<String>,
    #[serde(default)]
    pub allowed_sources: Vec<String>,
    #[serde(default)]
    pub include_keywords: Vec<String>,
    #[serde(default)]
    pub exclude_keywords: Vec<String>,
}

#[derive(Default, Deserialize)]
pub struct BuildStoredSearchProfileRequest {
    #[serde(default)]
    pub preferences: SearchPreferencesRequest,
}

#[derive(Deserialize)]
pub struct BuildSearchProfileRequest {
    pub raw_text: String,
    #[serde(default)]
    pub preferences: SearchPreferencesRequest,
}

#[derive(Debug)]
pub struct BuildSearchProfileInput {
    pub raw_text: String,
    pub preferences: SearchPreferences,
}

#[derive(Serialize)]
pub struct SearchProfileResponse {
    pub primary_role: String,
    pub target_roles: Vec<String>,
    pub seniority: String,
    pub target_regions: Vec<TargetRegion>,
    pub work_modes: Vec<WorkMode>,
    pub allowed_sources: Vec<String>,
    pub search_terms: Vec<String>,
    pub exclude_terms: Vec<String>,
}

#[derive(Serialize)]
pub struct BuildSearchProfileResponse {
    pub analyzed_profile: AnalyzeProfileResponse,
    pub search_profile: SearchProfileResponse,
}

impl BuildSearchProfileRequest {
    pub fn validate(self) -> Result<BuildSearchProfileInput, ApiError> {
        Ok(BuildSearchProfileInput {
            raw_text: validate_required_string("raw_text", self.raw_text, 20_000)?,
            preferences: self.preferences.validate()?,
        })
    }
}

impl SearchPreferencesRequest {
    pub fn validate(self) -> Result<SearchPreferences, ApiError> {
        let mut preferred_roles = Vec::new();
        let mut invalid_preferred_roles = Vec::new();
        let mut allowed_sources = Vec::new();
        let mut invalid_allowed_sources = Vec::new();

        for role in self.preferred_roles {
            match RoleId::parse_canonical_key(&role) {
                Some(role_id) => push_unique(&mut preferred_roles, role_id),
                None => push_unique(&mut invalid_preferred_roles, role),
            }
        }

        if !invalid_preferred_roles.is_empty() {
            return Err(ApiError::bad_request_with_details(
                "invalid_preferred_roles",
                "Unknown preferred_roles values",
                json!({
                    "field": "preferred_roles",
                    "invalid_values": invalid_preferred_roles,
                    "allowed_values": ROLE_CATALOG
                        .iter()
                        .map(|role| role.canonical_key)
                        .collect::<Vec<_>>(),
                }),
            ));
        }

        for source in self.allowed_sources {
            match SourceId::parse_canonical_key(&source) {
                Some(source_id) => push_unique(&mut allowed_sources, source_id),
                None => push_unique(&mut invalid_allowed_sources, source),
            }
        }

        if !invalid_allowed_sources.is_empty() {
            return Err(ApiError::bad_request_with_details(
                "invalid_allowed_sources",
                "Unknown allowed_sources values",
                json!({
                    "field": "allowed_sources",
                    "invalid_values": invalid_allowed_sources,
                    "allowed_values": SOURCE_CATALOG
                        .iter()
                        .map(|source| source.canonical_key)
                        .collect::<Vec<_>>(),
                }),
            ));
        }

        Ok(SearchPreferences {
            target_regions: self.target_regions,
            work_modes: self.work_modes,
            preferred_roles,
            allowed_sources,
            include_keywords: self.include_keywords,
            exclude_keywords: self.exclude_keywords,
        })
    }
}

impl From<SearchProfile> for SearchProfileResponse {
    fn from(search_profile: SearchProfile) -> Self {
        let SearchProfile {
            primary_role,
            target_roles,
            seniority,
            target_regions,
            work_modes,
            allowed_sources,
            search_terms,
            exclude_terms,
        } = search_profile;

        Self {
            primary_role: primary_role.to_string(),
            target_roles: target_roles
                .into_iter()
                .map(|role| role.to_string())
                .collect(),
            seniority,
            target_regions,
            work_modes,
            allowed_sources: allowed_sources
                .into_iter()
                .map(|source| source.canonical_key().to_string())
                .collect(),
            search_terms,
            exclude_terms,
        }
    }
}

fn push_unique<T>(target: &mut Vec<T>, value: T)
where
    T: PartialEq,
{
    if !target.iter().any(|existing| existing == &value) {
        target.push(value);
    }
}

fn validate_required_string(
    field: &'static str,
    value: String,
    max_len: usize,
) -> Result<String, ApiError> {
    let value = value.trim().to_string();

    if value.is_empty() {
        return Err(ApiError::bad_request_with_details(
            "invalid_search_profile_input",
            format!("Field '{field}' must not be empty"),
            json!({ "field": field }),
        ));
    }

    if value.len() > max_len {
        return Err(ApiError::bad_request_with_details(
            "invalid_search_profile_input",
            format!("Field '{field}' must be at most {max_len} characters"),
            json!({
                "field": field,
                "max_length": max_len,
                "received_length": value.len(),
            }),
        ));
    }

    Ok(value)
}

#[cfg(test)]
mod tests {
    use axum::response::IntoResponse;
    use serde_json::json;

    use crate::domain::role::RoleId;
    use crate::domain::search::profile::SearchProfile;
    use crate::domain::source::SourceId;

    use super::{
        BuildSearchProfileRequest, BuildStoredSearchProfileRequest, SearchPreferencesRequest,
        TargetRegion, WorkMode,
    };

    #[test]
    fn deserializes_valid_enum_preferences() {
        let payload = json!({
            "preferences": {
                "target_regions": ["ua", "eu_remote"],
                "work_modes": ["remote", "hybrid"],
                "preferred_roles": ["frontend_developer"],
                "allowed_sources": ["djinni", "work_ua"],
                "include_keywords": ["product company"],
                "exclude_keywords": ["gambling"]
            }
        });

        let request: BuildStoredSearchProfileRequest =
            serde_json::from_value(payload).expect("request should deserialize");

        assert_eq!(
            request.preferences.target_regions,
            vec![TargetRegion::Ua, TargetRegion::EuRemote]
        );
        assert_eq!(
            request.preferences.work_modes,
            vec![WorkMode::Remote, WorkMode::Hybrid]
        );
        assert_eq!(
            request.preferences.allowed_sources,
            vec!["djinni".to_string(), "work_ua".to_string()]
        );
    }

    #[test]
    fn rejects_unknown_enum_values() {
        let payload = json!({
            "preferences": {
                "target_regions": ["moon_remote"],
                "work_modes": ["anywhere"]
            }
        });

        let result = serde_json::from_value::<BuildStoredSearchProfileRequest>(payload);

        assert!(result.is_err());
    }

    #[test]
    fn rejects_unknown_preferred_roles() {
        let error = SearchPreferencesRequest {
            preferred_roles: vec![
                "frontend_developer".to_string(),
                "frontend_specialist".to_string(),
            ],
            ..SearchPreferencesRequest::default()
        }
        .validate()
        .expect_err("conversion should fail for unknown roles");

        assert_eq!(error.into_response().status(), 400);
    }

    #[test]
    fn converts_known_preferred_roles_successfully() {
        let preferences = SearchPreferencesRequest {
            preferred_roles: vec![
                "frontend_developer".to_string(),
                "react_native_developer".to_string(),
            ],
            ..SearchPreferencesRequest::default()
        }
        .validate()
        .expect("conversion should succeed for known roles");

        assert_eq!(
            preferences.preferred_roles,
            vec![RoleId::FrontendDeveloper, RoleId::ReactNativeDeveloper]
        );
    }

    #[test]
    fn accepts_known_allowed_sources() {
        let preferences = SearchPreferencesRequest {
            allowed_sources: vec!["djinni".to_string(), "work_ua".to_string()],
            ..SearchPreferencesRequest::default()
        }
        .validate()
        .expect("conversion should succeed for known sources");

        assert_eq!(
            preferences.allowed_sources,
            vec![SourceId::Djinni, SourceId::WorkUa]
        );
    }

    #[test]
    fn rejects_unknown_allowed_sources() {
        let error = SearchPreferencesRequest {
            allowed_sources: vec!["djinni".to_string(), "linkedin".to_string()],
            ..SearchPreferencesRequest::default()
        }
        .validate()
        .expect_err("conversion should fail for unknown sources");

        assert_eq!(error.into_response().status(), 400);
    }

    #[test]
    fn validates_raw_text_request_successfully() {
        let request = BuildSearchProfileRequest {
            raw_text: " Senior frontend engineer ".to_string(),
            preferences: SearchPreferencesRequest {
                preferred_roles: vec!["frontend_developer".to_string()],
                allowed_sources: vec!["djinni".to_string()],
                ..SearchPreferencesRequest::default()
            },
        }
        .validate()
        .expect("request should validate successfully");

        assert_eq!(request.raw_text, "Senior frontend engineer");
        assert_eq!(
            request.preferences.preferred_roles,
            vec![RoleId::FrontendDeveloper]
        );
        assert_eq!(request.preferences.allowed_sources, vec![SourceId::Djinni]);
    }

    #[test]
    fn rejects_blank_raw_text() {
        let result = BuildSearchProfileRequest {
            raw_text: "   ".to_string(),
            preferences: SearchPreferencesRequest::default(),
        }
        .validate();

        let error = match result {
            Ok(_) => panic!("blank raw_text should fail validation"),
            Err(error) => error,
        };

        assert_eq!(error.into_response().status(), 400);
    }

    #[test]
    fn serializes_search_profile_roles_as_snake_case_strings() {
        let response = super::SearchProfileResponse::from(SearchProfile {
            primary_role: RoleId::ReactNativeDeveloper,
            target_roles: vec![RoleId::ReactNativeDeveloper, RoleId::FrontendDeveloper],
            seniority: "senior".to_string(),
            target_regions: vec![TargetRegion::Ua],
            work_modes: vec![WorkMode::Remote],
            allowed_sources: vec![SourceId::Djinni, SourceId::RobotaUa],
            search_terms: vec!["react native developer".to_string()],
            exclude_terms: vec!["gambling".to_string()],
        });

        assert_eq!(response.primary_role, "react_native_developer");
        assert_eq!(
            response.target_roles,
            vec!["react_native_developer", "frontend_developer"]
        );
        assert_eq!(response.allowed_sources, vec!["djinni", "robota_ua"]);
    }
}
