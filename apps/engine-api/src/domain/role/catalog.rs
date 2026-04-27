use super::role_id::RoleId;

pub struct RoleMetadata {
    pub id: RoleId,
    pub canonical_key: &'static str,
    pub display_name: &'static str,
    pub search_aliases: &'static [&'static str],
    pub family: Option<&'static str>,
    pub is_fallback: bool,
}

pub const ROLE_CATALOG: &[RoleMetadata] = &[
    RoleMetadata {
        id: RoleId::FrontendEngineer,
        canonical_key: "frontend_engineer",
        display_name: "Frontend Engineer",
        search_aliases: &[
            "frontend engineer",
            "frontend developer",
            "front-end engineer",
            "front-end developer",
            "react developer",
            "vue developer",
            "angular developer",
            "javascript developer",
            "typescript developer",
            "svelte developer",
            "nuxt developer",
            "ui engineer",
            "web engineer",
            "nextjs developer",
            "front end developer",
            "web developer",
        ],
        family: Some("engineering"),
        is_fallback: false,
    },
    RoleMetadata {
        id: RoleId::BackendEngineer,
        canonical_key: "backend_engineer",
        display_name: "Backend Engineer",
        search_aliases: &[
            "backend engineer",
            "backend developer",
            "back-end engineer",
            "back-end developer",
            "back end developer",
            "server-side developer",
            "server developer",
            "api developer",
            "platform engineer",
            "software engineer",
            "rust engineer",
            "go developer",
            "python developer",
            "java developer",
            "node.js developer",
            "nodejs developer",
            "node developer",
            "php developer",
            "ruby developer",
        ],
        family: Some("engineering"),
        is_fallback: false,
    },
    RoleMetadata {
        id: RoleId::FullstackEngineer,
        canonical_key: "fullstack_engineer",
        display_name: "Fullstack Engineer",
        search_aliases: &[
            "fullstack engineer",
            "fullstack developer",
            "full-stack engineer",
            "full-stack developer",
            "full stack engineer",
            "full stack developer",
            "full stack web developer",
        ],
        family: Some("engineering"),
        is_fallback: false,
    },
    RoleMetadata {
        id: RoleId::MobileEngineer,
        canonical_key: "mobile_engineer",
        display_name: "Mobile Engineer",
        search_aliases: &[
            "mobile engineer",
            "mobile developer",
            "react native developer",
            "ios developer",
            "android developer",
            "cross-platform developer",
            "expo developer",
        ],
        family: Some("engineering"),
        is_fallback: false,
    },
    RoleMetadata {
        id: RoleId::DevopsEngineer,
        canonical_key: "devops_engineer",
        display_name: "DevOps Engineer",
        search_aliases: &[
            "devops engineer",
            "devops developer",
            "platform engineer",
            "cloud engineer",
            "site reliability engineer",
            "sre",
            "reliability engineer",
            "infrastructure engineer",
            "systems engineer",
            "kubernetes engineer",
            "ci/cd engineer",
        ],
        family: Some("engineering"),
        is_fallback: false,
    },
    RoleMetadata {
        id: RoleId::DataEngineer,
        canonical_key: "data_engineer",
        display_name: "Data Engineer",
        search_aliases: &[
            "data engineer",
            "big data engineer",
            "etl developer",
            "data analyst",
            "analytics engineer",
            "data pipeline engineer",
            "bi developer",
            "business intelligence developer",
            "database developer",
        ],
        family: Some("data"),
        is_fallback: false,
    },
    RoleMetadata {
        id: RoleId::MlEngineer,
        canonical_key: "ml_engineer",
        display_name: "ML Engineer",
        search_aliases: &[
            "ml engineer",
            "machine learning engineer",
            "ai engineer",
            "ai developer",
            "artificial intelligence engineer",
            "data scientist",
            "deep learning engineer",
            "computer vision engineer",
            "nlp engineer",
            "research engineer",
        ],
        family: Some("data"),
        is_fallback: false,
    },
    RoleMetadata {
        id: RoleId::QaEngineer,
        canonical_key: "qa_engineer",
        display_name: "QA Engineer",
        search_aliases: &[
            "qa engineer",
            "test engineer",
            "automation qa engineer",
            "quality assurance engineer",
            "quality engineer",
            "tester",
            "software tester",
            "manual tester",
            "manual qa",
            "sdet",
            "automation tester",
        ],
        family: Some("engineering"),
        is_fallback: false,
    },
    RoleMetadata {
        id: RoleId::ProductDesigner,
        canonical_key: "product_designer",
        display_name: "Product Designer",
        search_aliases: &[
            "product designer",
            "ui ux designer",
            "ux designer",
            "ui designer",
            "interaction designer",
        ],
        family: Some("design"),
        is_fallback: false,
    },
    RoleMetadata {
        id: RoleId::ProductManager,
        canonical_key: "product_manager",
        display_name: "Product Manager",
        search_aliases: &[
            "product manager",
            "product owner",
            "digital product manager",
        ],
        family: Some("product"),
        is_fallback: false,
    },
    RoleMetadata {
        id: RoleId::ProjectManager,
        canonical_key: "project_manager",
        display_name: "Project Manager",
        search_aliases: &["project manager", "delivery manager", "program coordinator"],
        family: Some("operations"),
        is_fallback: false,
    },
    RoleMetadata {
        id: RoleId::TechLead,
        canonical_key: "tech_lead",
        display_name: "Tech Lead",
        search_aliases: &[
            "tech lead",
            "technical lead",
            "lead engineer",
            "lead developer",
            "lead software engineer",
        ],
        family: Some("engineering"),
        is_fallback: false,
    },
    RoleMetadata {
        id: RoleId::EngineeringManager,
        canonical_key: "engineering_manager",
        display_name: "Engineering Manager",
        search_aliases: &[
            "engineering manager",
            "head of engineering",
            "vp of engineering",
            "software development manager",
        ],
        family: Some("engineering"),
        is_fallback: false,
    },
    RoleMetadata {
        id: RoleId::Generalist,
        canonical_key: "generalist",
        display_name: "Generalist",
        search_aliases: &[],
        family: None,
        is_fallback: true,
    },
];

pub fn find_role(role_id: RoleId) -> &'static RoleMetadata {
    ROLE_CATALOG
        .iter()
        .find(|role| role.id == role_id)
        .expect("role catalog must contain every RoleId variant")
}

pub fn find_role_by_key(canonical_key: &str) -> Option<&'static RoleMetadata> {
    ROLE_CATALOG
        .iter()
        .find(|role| role.canonical_key == canonical_key)
}

#[cfg(test)]
mod tests {
    use crate::domain::role::RoleId;

    use super::{ROLE_CATALOG, find_role, find_role_by_key};

    #[test]
    fn find_role_returns_matching_metadata_for_every_variant() {
        let variants = [
            RoleId::FrontendEngineer,
            RoleId::BackendEngineer,
            RoleId::FullstackEngineer,
            RoleId::MobileEngineer,
            RoleId::DevopsEngineer,
            RoleId::DataEngineer,
            RoleId::MlEngineer,
            RoleId::QaEngineer,
            RoleId::ProductDesigner,
            RoleId::ProductManager,
            RoleId::ProjectManager,
            RoleId::TechLead,
            RoleId::EngineeringManager,
            RoleId::Generalist,
        ];
        for variant in variants {
            let metadata = find_role(variant);
            assert_eq!(
                metadata.id, variant,
                "catalog entry for {variant:?} has wrong id"
            );
        }
    }

    #[test]
    fn find_role_by_key_returns_metadata_for_every_canonical_key() {
        for entry in ROLE_CATALOG {
            let found = find_role_by_key(entry.canonical_key)
                .unwrap_or_else(|| panic!("canonical key '{}' not found", entry.canonical_key));
            assert_eq!(found.id, entry.id);
        }
    }

    #[test]
    fn find_role_by_key_returns_none_for_unknown_key() {
        assert!(find_role_by_key("not_a_real_role").is_none());
        assert!(find_role_by_key("").is_none());
    }

    #[test]
    fn all_canonical_keys_are_unique() {
        let mut keys: Vec<&str> = ROLE_CATALOG
            .iter()
            .map(|entry| entry.canonical_key)
            .collect();
        let original_len = keys.len();
        keys.dedup();
        keys.sort_unstable();
        keys.dedup();
        assert_eq!(
            keys.len(),
            original_len,
            "duplicate canonical keys in ROLE_CATALOG"
        );
    }

    #[test]
    fn generalist_is_the_only_fallback_and_has_no_family() {
        let fallbacks: Vec<_> = ROLE_CATALOG
            .iter()
            .filter(|entry| entry.is_fallback)
            .collect();
        assert_eq!(fallbacks.len(), 1, "expected exactly one fallback role");
        assert_eq!(fallbacks[0].id, RoleId::Generalist);
        assert!(
            fallbacks[0].family.is_none(),
            "Generalist must have no family"
        );
    }

    #[test]
    fn non_fallback_roles_all_have_a_family() {
        for entry in ROLE_CATALOG.iter().filter(|entry| !entry.is_fallback) {
            assert!(
                entry.family.is_some(),
                "non-fallback role '{}' has no family",
                entry.canonical_key
            );
        }
    }

    fn aliases_for(role_id: RoleId) -> &'static [&'static str] {
        find_role(role_id).search_aliases
    }

    #[test]
    fn new_frontend_aliases_present() {
        let aliases = aliases_for(RoleId::FrontendEngineer);
        for expected in [
            "front-end engineer",
            "front-end developer",
            "typescript developer",
            "svelte developer",
            "nuxt developer",
            "web developer",
        ] {
            assert!(
                aliases.contains(&expected),
                "FrontendEngineer missing alias: {expected}"
            );
        }
    }

    #[test]
    fn new_backend_aliases_present() {
        let aliases = aliases_for(RoleId::BackendEngineer);
        for expected in [
            "back-end engineer",
            "back-end developer",
            "back end developer",
            "node.js developer",
            "nodejs developer",
            "node developer",
            "python developer",
            "java developer",
            "php developer",
            "ruby developer",
        ] {
            assert!(
                aliases.contains(&expected),
                "BackendEngineer missing alias: {expected}"
            );
        }
    }

    #[test]
    fn new_fullstack_aliases_present() {
        let aliases = aliases_for(RoleId::FullstackEngineer);
        for expected in [
            "full-stack engineer",
            "full stack developer",
            "full stack web developer",
        ] {
            assert!(
                aliases.contains(&expected),
                "FullstackEngineer missing alias: {expected}"
            );
        }
    }

    #[test]
    fn new_devops_aliases_present() {
        let aliases = aliases_for(RoleId::DevopsEngineer);
        for expected in [
            "devops developer",
            "reliability engineer",
            "systems engineer",
            "kubernetes engineer",
            "ci/cd engineer",
        ] {
            assert!(
                aliases.contains(&expected),
                "DevopsEngineer missing alias: {expected}"
            );
        }
    }

    #[test]
    fn new_ml_aliases_present() {
        let aliases = aliases_for(RoleId::MlEngineer);
        for expected in [
            "ai developer",
            "artificial intelligence engineer",
            "computer vision engineer",
            "nlp engineer",
            "research engineer",
        ] {
            assert!(
                aliases.contains(&expected),
                "MlEngineer missing alias: {expected}"
            );
        }
    }

    #[test]
    fn new_qa_aliases_present() {
        let aliases = aliases_for(RoleId::QaEngineer);
        for expected in [
            "quality engineer",
            "tester",
            "software tester",
            "manual tester",
            "manual qa",
            "sdet",
            "automation tester",
        ] {
            assert!(
                aliases.contains(&expected),
                "QaEngineer missing alias: {expected}"
            );
        }
    }

    #[test]
    fn new_data_aliases_present() {
        let aliases = aliases_for(RoleId::DataEngineer);
        for expected in [
            "data pipeline engineer",
            "bi developer",
            "business intelligence developer",
            "database developer",
        ] {
            assert!(
                aliases.contains(&expected),
                "DataEngineer missing alias: {expected}"
            );
        }
    }
}
