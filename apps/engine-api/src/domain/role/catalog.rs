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
        id: RoleId::ReactNativeDeveloper,
        canonical_key: "react_native_developer",
        display_name: "React Native Developer",
        search_aliases: &[
            "react native developer",
            "mobile developer",
            "cross-platform developer",
            "expo developer",
        ],
        family: Some("engineering"),
        is_fallback: false,
    },
    RoleMetadata {
        id: RoleId::MobileDeveloper,
        canonical_key: "mobile_developer",
        display_name: "Mobile Developer",
        search_aliases: &[
            "mobile developer",
            "mobile engineer",
            "ios android developer",
        ],
        family: Some("engineering"),
        is_fallback: false,
    },
    RoleMetadata {
        id: RoleId::FrontendDeveloper,
        canonical_key: "frontend_developer",
        display_name: "Frontend Developer",
        search_aliases: &[
            "frontend developer",
            "react developer",
            "javascript developer",
            "front end developer",
            "web engineer",
            "ui engineer",
            "nextjs developer",
        ],
        family: Some("engineering"),
        is_fallback: false,
    },
    RoleMetadata {
        id: RoleId::BackendDeveloper,
        canonical_key: "backend_developer",
        display_name: "Backend Developer",
        search_aliases: &[
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
        id: RoleId::FullstackDeveloper,
        canonical_key: "fullstack_developer",
        display_name: "Fullstack Developer",
        search_aliases: &[
            "fullstack developer",
            "full-stack developer",
            "software engineer",
            "full stack engineer",
        ],
        family: Some("engineering"),
        is_fallback: false,
    },
    RoleMetadata {
        id: RoleId::QaEngineer,
        canonical_key: "qa_engineer",
        display_name: "QA Engineer",
        search_aliases: &["qa engineer", "test engineer", "automation qa engineer"],
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
        id: RoleId::DataAnalyst,
        canonical_key: "data_analyst",
        display_name: "Data Analyst",
        search_aliases: &[
            "data analyst",
            "business data analyst",
            "analytics specialist",
        ],
        family: Some("data"),
        is_fallback: false,
    },
    RoleMetadata {
        id: RoleId::UiUxDesigner,
        canonical_key: "ui_ux_designer",
        display_name: "UI/UX Designer",
        search_aliases: &["ui ux designer", "product designer", "ux designer"],
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
        id: RoleId::MarketingSpecialist,
        canonical_key: "marketing_specialist",
        display_name: "Marketing Specialist",
        search_aliases: &[
            "marketing specialist",
            "digital marketer",
            "performance marketer",
        ],
        family: Some("marketing"),
        is_fallback: false,
    },
    RoleMetadata {
        id: RoleId::SalesManager,
        canonical_key: "sales_manager",
        display_name: "Sales Manager",
        search_aliases: &[
            "sales manager",
            "business development manager",
            "account executive",
        ],
        family: Some("sales"),
        is_fallback: false,
    },
    RoleMetadata {
        id: RoleId::CustomerSupportSpecialist,
        canonical_key: "customer_support_specialist",
        display_name: "Customer Support Specialist",
        search_aliases: &[
            "customer support specialist",
            "support agent",
            "client support specialist",
        ],
        family: Some("support"),
        is_fallback: false,
    },
    RoleMetadata {
        id: RoleId::Recruiter,
        canonical_key: "recruiter",
        display_name: "Recruiter",
        search_aliases: &[
            "recruiter",
            "talent acquisition specialist",
            "technical recruiter",
        ],
        family: Some("people"),
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
