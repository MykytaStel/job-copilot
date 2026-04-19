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
            "react developer",
            "vue developer",
            "angular developer",
            "javascript developer",
            "ui engineer",
            "web engineer",
            "nextjs developer",
            "front end developer",
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
            "server-side developer",
            "api developer",
            "platform engineer",
            "software engineer",
            "rust engineer",
            "go developer",
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
            "full-stack developer",
            "full stack engineer",
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
            "platform engineer",
            "cloud engineer",
            "site reliability engineer",
            "sre",
            "infrastructure engineer",
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
            "data scientist",
            "deep learning engineer",
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
