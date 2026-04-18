use super::SourceId;

pub struct SourceMetadata {
    pub id: SourceId,
    pub canonical_key: &'static str,
    pub display_name: &'static str,
}

pub const SOURCE_CATALOG: &[SourceMetadata] = &[
    SourceMetadata {
        id: SourceId::Djinni,
        canonical_key: "djinni",
        display_name: "Djinni",
    },
    SourceMetadata {
        id: SourceId::DouUa,
        canonical_key: "dou_ua",
        display_name: "DOU",
    },
    SourceMetadata {
        id: SourceId::WorkUa,
        canonical_key: "work_ua",
        display_name: "Work.ua",
    },
    SourceMetadata {
        id: SourceId::RobotaUa,
        canonical_key: "robota_ua",
        display_name: "Robota.ua",
    },
];

pub fn find_source(source_id: SourceId) -> &'static SourceMetadata {
    SOURCE_CATALOG
        .iter()
        .find(|source| source.id == source_id)
        .expect("source catalog must contain every SourceId variant")
}

pub fn find_source_by_key(canonical_key: &str) -> Option<&'static SourceMetadata> {
    SOURCE_CATALOG
        .iter()
        .find(|source| source.canonical_key == canonical_key)
}
