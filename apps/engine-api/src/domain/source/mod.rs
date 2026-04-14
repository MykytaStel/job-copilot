pub mod catalog;

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use self::catalog::{find_source, find_source_by_key};

pub use catalog::{SOURCE_CATALOG, SourceMetadata};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceId {
    Djinni,
    WorkUa,
    RobotaUa,
}

impl SourceId {
    pub fn canonical_key(self) -> &'static str {
        find_source(self).canonical_key
    }

    pub fn display_name(self) -> &'static str {
        find_source(self).display_name
    }

    pub fn parse_canonical_key(value: &str) -> Option<Self> {
        find_source_by_key(value).map(|source| source.id)
    }
}

impl fmt::Display for SourceId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.canonical_key())
    }
}

impl FromStr for SourceId {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::parse_canonical_key(value).ok_or(())
    }
}

#[cfg(test)]
mod tests {
    use super::SourceId;

    #[test]
    fn serde_roundtrip_uses_canonical_snake_case() {
        let serialized =
            serde_json::to_string(&SourceId::WorkUa).expect("source should serialize successfully");
        let deserialized: SourceId =
            serde_json::from_str(&serialized).expect("source should deserialize successfully");

        assert_eq!(serialized, "\"work_ua\"");
        assert_eq!(deserialized, SourceId::WorkUa);
    }

    #[test]
    fn canonical_key_and_parser_match_catalog() {
        assert_eq!(SourceId::Djinni.canonical_key(), "djinni");
        assert_eq!(SourceId::WorkUa.canonical_key(), "work_ua");
        assert_eq!(SourceId::RobotaUa.canonical_key(), "robota_ua");

        assert_eq!(
            SourceId::parse_canonical_key("djinni"),
            Some(SourceId::Djinni)
        );
        assert_eq!(
            SourceId::parse_canonical_key("work_ua"),
            Some(SourceId::WorkUa)
        );
        assert_eq!(
            SourceId::parse_canonical_key("robota_ua"),
            Some(SourceId::RobotaUa)
        );
        assert_eq!(SourceId::parse_canonical_key("linkedin"), None);
    }
}
