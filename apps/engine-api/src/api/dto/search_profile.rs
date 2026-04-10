use serde::{Deserialize, Serialize};

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

#[derive(Deserialize)]
pub struct BuildSearchProfileRequest {
    pub raw_text: String,
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
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<BuildSearchProfileWarningResponse>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeprecatedPreferredRoleUsage {
    pub received: String,
    pub normalized_to: RoleId,
}

#[derive(Debug)]
pub struct ParsedSearchPreferences {
    pub preferences: SearchPreferences,
    pub deprecated_preferred_roles: Vec<DeprecatedPreferredRoleUsage>,
}

#[derive(Serialize)]
pub struct DeprecatedPreferredRoleReplacementResponse {
    pub received: String,
    pub normalized_to: String,
}

#[derive(Serialize)]
pub struct BuildSearchProfileWarningResponse {
    pub code: &'static str,
    pub field: &'static str,
    pub message: &'static str,
    pub replacements: Vec<DeprecatedPreferredRoleReplacementResponse>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct SearchPreferencesValidationError {
    invalid_preferred_roles: Vec<String>,
}

impl SearchPreferencesValidationError {
    pub fn invalid_preferred_roles(&self) -> &[String] {
        &self.invalid_preferred_roles
    }

    pub fn allowed_preferred_roles(&self) -> Vec<String> {
        ROLE_CATALOG
            .iter()
            .map(|role| role.canonical_key.to_string())
            .collect()
    }
}

impl SearchPreferencesRequest {
    pub fn parse(self) -> Result<ParsedSearchPreferences, SearchPreferencesValidationError> {
        let mut preferred_roles = Vec::new();
        let mut invalid_preferred_roles = Vec::new();
        let mut deprecated_preferred_roles = Vec::new();

        for role in self.preferred_roles {
            match RoleId::parse_api_key(&role) {
                Some(role_id) => {
                    if role_id.canonical_key() != role {
                        push_unique(
                            &mut deprecated_preferred_roles,
                            DeprecatedPreferredRoleUsage {
                                received: role.clone(),
                                normalized_to: role_id,
                            },
                        );
                    }

                    push_unique(&mut preferred_roles, role_id);
                }
                None => push_unique(&mut invalid_preferred_roles, role),
            }
        }

        if !invalid_preferred_roles.is_empty() {
            return Err(SearchPreferencesValidationError {
                invalid_preferred_roles,
            });
        }

        Ok(ParsedSearchPreferences {
            preferences: SearchPreferences {
                target_regions: self.target_regions,
                work_modes: self.work_modes,
                preferred_roles,
                include_keywords: self.include_keywords,
                exclude_keywords: self.exclude_keywords,
            },
            deprecated_preferred_roles,
        })
    }
}

impl TryFrom<SearchPreferencesRequest> for SearchPreferences {
    type Error = SearchPreferencesValidationError;

    fn try_from(request: SearchPreferencesRequest) -> Result<Self, Self::Error> {
        request.parse().map(|parsed| parsed.preferences)
    }
}

impl BuildSearchProfileWarningResponse {
    pub fn from_deprecated_preferred_roles(
        deprecated_preferred_roles: &[DeprecatedPreferredRoleUsage],
    ) -> Option<Self> {
        if deprecated_preferred_roles.is_empty() {
            return None;
        }

        Some(Self {
            code: "deprecated_preferred_roles",
            field: "preferred_roles",
            message: "Deprecated preferred_roles values were normalized to canonical role ids",
            replacements: deprecated_preferred_roles
                .iter()
                .map(|usage| DeprecatedPreferredRoleReplacementResponse {
                    received: usage.received.clone(),
                    normalized_to: usage.normalized_to.to_string(),
                })
                .collect(),
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
    use serde_json::json;

    use crate::domain::role::RoleId;
    use crate::domain::search::profile::{SearchPreferences, SearchProfile};

    use super::{
        BuildSearchProfileRequest, BuildSearchProfileWarningResponse, SearchPreferencesRequest,
        SearchPreferencesValidationError, TargetRegion, WorkMode,
    };

    #[test]
    fn deserializes_valid_enum_preferences() {
        let payload = json!({
            "raw_text": "Senior frontend engineer",
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
            "raw_text": "Senior frontend engineer",
            "preferences": {
                "target_regions": ["moon_remote"],
                "work_modes": ["anywhere"]
            }
        });

        let result = serde_json::from_value::<BuildSearchProfileRequest>(payload);

        assert!(result.is_err());
    }

    #[test]
    fn rejects_unknown_preferred_roles_during_conversion() {
        let payload = json!({
            "raw_text": "Senior frontend engineer",
            "preferences": {
                "preferred_roles": ["frontend_developer", "frontend_specialist", "frontend_specialist"]
            }
        });

        let request: BuildSearchProfileRequest =
            serde_json::from_value(payload).expect("request should deserialize");

        let error = SearchPreferences::try_from(request.preferences)
            .expect_err("conversion should fail for unknown roles");

        assert_eq!(
            error,
            SearchPreferencesValidationError {
                invalid_preferred_roles: vec!["frontend_specialist".to_string()],
            }
        );
        assert!(
            error
                .allowed_preferred_roles()
                .contains(&"frontend_developer".to_string())
        );
    }

    #[test]
    fn converts_known_preferred_roles_successfully() {
        let payload = json!({
            "raw_text": "Senior frontend engineer",
            "preferences": {
                "preferred_roles": ["frontend_developer", "react_native_developer"]
            }
        });

        let request: BuildSearchProfileRequest =
            serde_json::from_value(payload).expect("request should deserialize");

        let preferences = SearchPreferences::try_from(request.preferences)
            .expect("conversion should succeed for known roles");

        assert_eq!(
            preferences.preferred_roles,
            vec![RoleId::FrontendDeveloper, RoleId::ReactNativeDeveloper]
        );
    }

    #[test]
    fn normalizes_deprecated_preferred_role_ids() {
        let payload = json!({
            "raw_text": "Senior frontend engineer",
            "preferences": {
                "preferred_roles": ["front_end_developer", "full_stack_developer"]
            }
        });

        let request: BuildSearchProfileRequest =
            serde_json::from_value(payload).expect("request should deserialize");

        let preferences = SearchPreferences::try_from(request.preferences)
            .expect("conversion should accept deprecated role ids");

        assert_eq!(
            preferences.preferred_roles,
            vec![RoleId::FrontendDeveloper, RoleId::FullstackDeveloper]
        );
    }

    #[test]
    fn collects_deprecated_preferred_role_warnings() {
        let parsed = SearchPreferencesRequest {
            preferred_roles: vec![
                "front_end_developer".to_string(),
                "full_stack_developer".to_string(),
                "frontend_developer".to_string(),
            ],
            ..SearchPreferencesRequest::default()
        }
        .parse()
        .expect("conversion should succeed");

        let warning = BuildSearchProfileWarningResponse::from_deprecated_preferred_roles(
            &parsed.deprecated_preferred_roles,
        )
        .expect("warning should be present");

        let replacements = warning
            .replacements
            .iter()
            .map(|replacement| {
                (
                    replacement.received.as_str(),
                    replacement.normalized_to.as_str(),
                )
            })
            .collect::<Vec<_>>();

        assert_eq!(
            replacements,
            vec![
                ("front_end_developer", "frontend_developer"),
                ("full_stack_developer", "fullstack_developer"),
            ]
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

    #[test]
    fn skips_deprecated_preferred_role_warning_when_not_needed() {
        assert!(BuildSearchProfileWarningResponse::from_deprecated_preferred_roles(&[]).is_none());
    }
}
