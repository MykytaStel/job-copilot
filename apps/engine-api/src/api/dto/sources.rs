use serde::Serialize;

use crate::domain::source::{SOURCE_CATALOG, SourceMetadata};

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct SourceCatalogItemResponse {
    pub id: String,
    pub display_name: String,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct SourceCatalogResponse {
    pub sources: Vec<SourceCatalogItemResponse>,
}

impl From<&SourceMetadata> for SourceCatalogItemResponse {
    fn from(source: &SourceMetadata) -> Self {
        Self {
            id: source.canonical_key.to_string(),
            display_name: source.id.display_name().to_string(),
        }
    }
}

impl SourceCatalogResponse {
    pub fn from_catalog() -> Self {
        Self {
            sources: SOURCE_CATALOG
                .iter()
                .map(SourceCatalogItemResponse::from)
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::SourceCatalogResponse;

    #[test]
    fn builds_source_catalog_response_from_domain_catalog() {
        let response = SourceCatalogResponse::from_catalog();

        assert!(
            response
                .sources
                .iter()
                .any(|source| source.id == "djinni" && source.display_name == "Djinni")
        );
        assert!(
            response
                .sources
                .iter()
                .any(|source| source.id == "dou_ua" && source.display_name == "DOU")
        );
        assert!(
            response
                .sources
                .iter()
                .any(|source| source.id == "work_ua" && source.display_name == "Work.ua")
        );
        assert!(
            response
                .sources
                .iter()
                .any(|source| source.id == "robota_ua" && source.display_name == "Robota.ua")
        );
    }
}
