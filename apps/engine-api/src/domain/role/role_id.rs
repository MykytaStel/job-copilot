use std::fmt;
use std::str::FromStr;

use super::catalog::{find_role, find_role_by_api_key, find_role_by_key};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RoleId {
    ReactNativeDeveloper,
    MobileDeveloper,
    FrontendDeveloper,
    BackendDeveloper,
    FullstackDeveloper,
    QaEngineer,
    DevopsEngineer,
    DataAnalyst,
    UiUxDesigner,
    ProductManager,
    ProjectManager,
    MarketingSpecialist,
    SalesManager,
    CustomerSupportSpecialist,
    Recruiter,
    Generalist,
}

impl RoleId {
    pub fn canonical_key(self) -> &'static str {
        find_role(self).canonical_key
    }

    pub fn display_name(self) -> &'static str {
        find_role(self).display_name
    }

    pub fn search_aliases(self) -> &'static [&'static str] {
        find_role(self).search_aliases
    }

    pub fn deprecated_api_keys(self) -> &'static [&'static str] {
        find_role(self).deprecated_api_keys
    }

    #[allow(dead_code)]
    pub fn family(self) -> Option<&'static str> {
        find_role(self).family
    }

    #[allow(dead_code)]
    pub fn is_fallback(self) -> bool {
        find_role(self).is_fallback
    }

    pub fn search_label(self) -> String {
        self.canonical_key().replace('_', " ")
    }

    pub fn fallback() -> Self {
        Self::Generalist
    }

    pub fn parse_canonical_key(value: &str) -> Option<Self> {
        find_role_by_key(value).map(|role| role.id)
    }

    pub fn parse_api_key(value: &str) -> Option<Self> {
        find_role_by_api_key(value).map(|role| role.id)
    }
}

impl fmt::Display for RoleId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.canonical_key())
    }
}

impl FromStr for RoleId {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::parse_canonical_key(value).ok_or(())
    }
}

#[cfg(test)]
mod tests {
    use super::RoleId;

    #[test]
    fn converts_to_canonical_snake_case() {
        assert_eq!(
            RoleId::ReactNativeDeveloper.canonical_key(),
            "react_native_developer"
        );
        assert_eq!(RoleId::Generalist.canonical_key(), "generalist");
    }

    #[test]
    fn converts_to_display_name() {
        assert_eq!(
            RoleId::ReactNativeDeveloper.display_name(),
            "React Native Developer"
        );
        assert_eq!(RoleId::Generalist.display_name(), "Generalist");
    }

    #[test]
    fn parses_canonical_snake_case() {
        assert_eq!(
            RoleId::parse_canonical_key("react_native_developer"),
            Some(RoleId::ReactNativeDeveloper)
        );
        assert_eq!(
            RoleId::parse_canonical_key("generalist"),
            Some(RoleId::Generalist)
        );
        assert_eq!(RoleId::parse_canonical_key("unknown_role"), None);
    }

    #[test]
    fn parses_deprecated_api_keys() {
        assert_eq!(
            RoleId::parse_api_key("front_end_developer"),
            Some(RoleId::FrontendDeveloper)
        );
        assert_eq!(
            RoleId::parse_api_key("full_stack_developer"),
            Some(RoleId::FullstackDeveloper)
        );
        assert_eq!(RoleId::parse_api_key("unknown_role"), None);
    }

    #[test]
    fn exposes_catalog_metadata() {
        assert_eq!(RoleId::ReactNativeDeveloper.family(), Some("engineering"));
        assert!(!RoleId::ReactNativeDeveloper.is_fallback());
        assert!(RoleId::Generalist.is_fallback());
        assert_eq!(
            RoleId::FrontendDeveloper.deprecated_api_keys(),
            &["front_end_developer"]
        );
    }
}
