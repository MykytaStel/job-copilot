use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::api::error::ApiError;
use crate::domain::role::RoleId;
use crate::domain::role::catalog::ROLE_CATALOG;
use crate::domain::search::profile::{SearchPreferences, SearchProfile, TargetRegion, WorkMode};

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
    pub include_keywords: Vec<String>,
    #[serde(default)]
    pub exclude_keywords: Vec<String>,
}

#[derive(Default, Deserialize)]
pub struct BuildSearchProfileRequest {
    #[serde(default)]
    pub preferences: SearchPreferencesRequest,
}

#[derive(Serialize)]
pub struct SearchProfileResponse {
    pub primary_role: String,
    pub target_roles: Vec<String>,
    pub seniority: String,
    pub target_regions: Vec<TargetRegion>,
    pub work_modes: Vec<WorkMode>,
    pub search_terms: Vec<String>,
    pub exclude_terms: Vec<String>,
}

#[derive(Serialize)]
pub struct BuildSearchProfileResponse {
    pub analyzed_profile: AnalyzeProfileResponse,
    pub search_profile: SearchProfileResponse,
}

impl SearchPreferencesRequest {
    pub fn validate(self) -> Result<SearchPreferences, ApiError> {
        let mut preferred_roles = Vec::new();
        let mut invalid_preferred_roles = Vec::new();

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

        Ok(SearchPreferences {
            target_regions: self.target_regions,
            work_modes: self.work_modes,
            preferred_roles,
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

#[cfg(test)]
mod tests {
    use axum::response::IntoResponse;
    use serde_json::json;

    use crate::domain::role::RoleId;
    use crate::domain::search::profile::SearchProfile;

    use super::{BuildSearchProfileRequest, SearchPreferencesRequest, TargetRegion, WorkMode};

    #[test]
    fn deserializes_valid_enum_preferences() {
        let payload = json!({
            "preferences": {
                "target_regions": ["ua", "eu_remote"],
                "work_modes": ["remote", "hybrid"],
                "preferred_roles": ["frontend_developer"],
                "include_keywords": ["product company"],
                "exclude_keywords": ["gambling"]
            }
        });

        let request: BuildSearchProfileRequest =
            serde_json::from_value(payload).expect("request should deserialize");

        assert_eq!(
            request.preferences.target_regions,
            vec![TargetRegion::Ua, TargetRegion::EuRemote]
        );
        assert_eq!(
            request.preferences.work_modes,
            vec![WorkMode::Remote, WorkMode::Hybrid]
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

        let result = serde_json::from_value::<BuildSearchProfileRequest>(payload);

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
    fn serializes_search_profile_roles_as_snake_case_strings() {
        let response = super::SearchProfileResponse::from(SearchProfile {
            primary_role: RoleId::ReactNativeDeveloper,
            target_roles: vec![RoleId::ReactNativeDeveloper, RoleId::FrontendDeveloper],
            seniority: "senior".to_string(),
            target_regions: vec![TargetRegion::Ua],
            work_modes: vec![WorkMode::Remote],
            search_terms: vec!["react native developer".to_string()],
            exclude_terms: vec!["gambling".to_string()],
        });

        assert_eq!(response.primary_role, "react_native_developer");
        assert_eq!(
            response.target_roles,
            vec!["react_native_developer", "frontend_developer"]
        );
    }
}
