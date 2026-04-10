use serde::Serialize;

use crate::domain::role::catalog::{ROLE_CATALOG, RoleMetadata};

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct RoleCatalogItemResponse {
    pub id: String,
    pub display_name: String,
    pub deprecated_api_ids: Vec<String>,
    pub family: Option<String>,
    pub is_fallback: bool,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct RoleCatalogResponse {
    pub roles: Vec<RoleCatalogItemResponse>,
}

impl From<&RoleMetadata> for RoleCatalogItemResponse {
    fn from(role: &RoleMetadata) -> Self {
        Self {
            id: role.canonical_key.to_string(),
            display_name: role.display_name.to_string(),
            deprecated_api_ids: role
                .id
                .deprecated_api_keys()
                .iter()
                .map(|id| (*id).to_string())
                .collect(),
            family: role.family.map(str::to_string),
            is_fallback: role.is_fallback,
        }
    }
}

impl RoleCatalogResponse {
    pub fn from_catalog() -> Self {
        Self {
            roles: ROLE_CATALOG
                .iter()
                .map(RoleCatalogItemResponse::from)
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::RoleCatalogResponse;

    #[test]
    fn builds_role_catalog_response_from_domain_catalog() {
        let response = RoleCatalogResponse::from_catalog();

        assert!(
            response
                .roles
                .iter()
                .any(|role| role.id == "frontend_developer"
                    && role.display_name == "Frontend Developer"
                    && role.deprecated_api_ids == vec!["front_end_developer".to_string()]
                    && role.family.as_deref() == Some("engineering")
                    && !role.is_fallback)
        );
        assert!(
            response
                .roles
                .iter()
                .any(|role| role.id == "generalist" && role.is_fallback)
        );
    }
}
