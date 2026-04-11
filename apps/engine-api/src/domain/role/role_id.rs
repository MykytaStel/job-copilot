use std::fmt;
use std::str::FromStr;

use super::catalog::{find_role, find_role_by_key};

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

    pub fn family(self) -> Option<&'static str> {
        find_role(self).family
    }

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
    fn exposes_catalog_metadata() {
        assert_eq!(RoleId::ReactNativeDeveloper.family(), Some("engineering"));
        assert!(!RoleId::ReactNativeDeveloper.is_fallback());
        assert!(RoleId::Generalist.is_fallback());
    }
}
