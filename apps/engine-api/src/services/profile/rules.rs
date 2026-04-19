use crate::domain::role::RoleId;

pub(crate) const MIN_ROLE_SCORE: u32 = 5;
pub(crate) const MAX_ROLE_CANDIDATES: usize = 3;

pub(crate) const KNOWN_SKILLS: &[&str] = &[
    "rust",
    "postgres",
    "postgresql",
    "go",
    "react native",
    "next.js",
    "typescript",
    "react",
    "javascript",
    "node.js",
    "python",
    "java",
    "sql",
    "graphql",
    "rest api",
    "microservices",
    "distributed systems",
    "swift",
    "kotlin",
    "ios",
    "android",
    "aws",
    "docker",
    "kubernetes",
    "terraform",
    "helm",
    "redis",
    "design system",
    "linux",
    "figma",
    "seo",
    "google ads",
    "salesforce",
    "jira",
    "excel",
    "power bi",
    "testing",
    "qa",
    "customer support",
    "recruiting",
    "sourcing",
];

pub(crate) const KNOWN_KEYWORDS: &[&str] = &[
    "mobile",
    "frontend",
    "backend",
    "fullstack",
    "web",
    "cloud",
    "platform",
    "api",
    "distributed systems",
    "microservices",
    "infrastructure",
    "automation",
    "testing",
    "design",
    "product",
    "analytics",
    "marketing",
    "sales",
    "support",
    "hiring",
    "remote",
];

pub(crate) struct RoleRule {
    pub(crate) role: RoleId,
    pub(crate) signals: &'static [(&'static str, u32)],
    pub(crate) combination_bonuses: &'static [CombinationBonusRule],
}

pub(crate) struct CombinationBonusRule {
    pub(crate) label: &'static str,
    pub(crate) required_groups: &'static [SignalGroup],
    pub(crate) bonus: u32,
}

pub(crate) struct SignalGroup {
    pub(crate) signals: &'static [&'static str],
    pub(crate) min_matches: usize,
}

pub(crate) const ROLE_RULES: &[RoleRule] = &[
    RoleRule {
        role: RoleId::ReactNativeDeveloper,
        signals: &[
            ("react native", 10),
            ("mobile", 4),
            ("ios", 3),
            ("android", 3),
            ("typescript", 2),
            ("react", 2),
        ],
        combination_bonuses: &[CombinationBonusRule {
            label: "bonus: react native + ios/android/mobile",
            required_groups: &[SignalGroup {
                signals: &["react native", "ios", "android", "mobile"],
                min_matches: 2,
            }],
            bonus: 6,
        }],
    },
    RoleRule {
        role: RoleId::FrontendDeveloper,
        signals: &[
            ("frontend", 5),
            ("react", 3),
            ("typescript", 2),
            ("javascript", 2),
            ("next.js", 2),
            ("design system", 2),
            ("ui", 2),
            ("web", 2),
        ],
        combination_bonuses: &[],
    },
    RoleRule {
        role: RoleId::BackendDeveloper,
        signals: &[
            ("backend", 5),
            ("api", 3),
            ("node.js", 3),
            ("python", 3),
            ("java", 3),
            ("rust", 4),
            ("go", 3),
            ("postgres", 2),
            ("graphql", 2),
            ("distributed systems", 2),
            ("microservices", 2),
            ("sql", 2),
        ],
        combination_bonuses: &[],
    },
    RoleRule {
        role: RoleId::FullstackDeveloper,
        signals: &[
            ("fullstack", 6),
            ("full-stack", 6),
            ("frontend", 2),
            ("backend", 2),
            ("react", 1),
            ("node.js", 1),
            ("typescript", 1),
            ("graphql", 1),
            ("api", 1),
        ],
        combination_bonuses: &[CombinationBonusRule {
            label: "bonus: frontend + backend mix",
            required_groups: &[
                SignalGroup {
                    signals: &["frontend", "react", "typescript", "next.js"],
                    min_matches: 1,
                },
                SignalGroup {
                    signals: &["backend", "api", "node.js", "python", "java", "rust", "go"],
                    min_matches: 1,
                },
            ],
            bonus: 4,
        }],
    },
    RoleRule {
        role: RoleId::QaEngineer,
        signals: &[
            ("qa", 5),
            ("quality assurance", 5),
            ("testing", 4),
            ("test automation", 4),
            ("automation", 2),
        ],
        combination_bonuses: &[CombinationBonusRule {
            label: "bonus: qa/testing + automation",
            required_groups: &[
                SignalGroup {
                    signals: &["qa", "quality assurance", "testing"],
                    min_matches: 1,
                },
                SignalGroup {
                    signals: &["automation", "test automation"],
                    min_matches: 1,
                },
            ],
            bonus: 5,
        }],
    },
    RoleRule {
        role: RoleId::DevopsEngineer,
        signals: &[
            ("devops", 6),
            ("docker", 3),
            ("kubernetes", 3),
            ("aws", 3),
            ("terraform", 3),
            ("helm", 2),
            ("platform", 2),
            ("infrastructure", 2),
            ("linux", 2),
            ("ci/cd", 3),
        ],
        combination_bonuses: &[],
    },
    RoleRule {
        role: RoleId::DataAnalyst,
        signals: &[
            ("data analyst", 6),
            ("analytics", 4),
            ("sql", 3),
            ("excel", 2),
            ("power bi", 3),
            ("dashboard", 2),
        ],
        combination_bonuses: &[],
    },
    RoleRule {
        role: RoleId::UiUxDesigner,
        signals: &[
            ("ui", 4),
            ("ux", 4),
            ("figma", 4),
            ("design system", 3),
            ("prototype", 2),
            ("wireframe", 2),
        ],
        combination_bonuses: &[],
    },
    RoleRule {
        role: RoleId::ProductManager,
        signals: &[
            ("product manager", 7),
            ("product management", 6),
            ("roadmap", 3),
            ("stakeholder", 2),
            ("requirements", 2),
            ("discovery", 2),
        ],
        combination_bonuses: &[CombinationBonusRule {
            label: "bonus: roadmap + stakeholder/requirements",
            required_groups: &[
                SignalGroup {
                    signals: &["roadmap"],
                    min_matches: 1,
                },
                SignalGroup {
                    signals: &["stakeholder", "requirements"],
                    min_matches: 1,
                },
            ],
            bonus: 4,
        }],
    },
    RoleRule {
        role: RoleId::ProjectManager,
        signals: &[
            ("project manager", 7),
            ("project management", 6),
            ("delivery", 3),
            ("timeline", 2),
            ("coordination", 2),
            ("jira", 2),
        ],
        combination_bonuses: &[],
    },
    RoleRule {
        role: RoleId::MarketingSpecialist,
        signals: &[
            ("marketing", 5),
            ("seo", 4),
            ("google ads", 4),
            ("campaign", 3),
            ("content", 2),
            ("social media", 2),
        ],
        combination_bonuses: &[CombinationBonusRule {
            label: "bonus: seo/google ads/campaign/analytics mix",
            required_groups: &[SignalGroup {
                signals: &["seo", "google ads", "campaign", "analytics"],
                min_matches: 2,
            }],
            bonus: 5,
        }],
    },
    RoleRule {
        role: RoleId::SalesManager,
        signals: &[
            ("sales", 5),
            ("lead generation", 4),
            ("crm", 3),
            ("negotiation", 3),
            ("pipeline", 2),
            ("salesforce", 3),
        ],
        combination_bonuses: &[],
    },
    RoleRule {
        role: RoleId::CustomerSupportSpecialist,
        signals: &[
            ("customer support", 6),
            ("support", 4),
            ("client support", 4),
            ("ticket", 2),
            ("help desk", 3),
            ("communication", 1),
        ],
        combination_bonuses: &[],
    },
    RoleRule {
        role: RoleId::Recruiter,
        signals: &[
            ("recruiter", 6),
            ("recruiting", 5),
            ("sourcing", 4),
            ("candidate screening", 3),
            ("talent acquisition", 5),
            ("interviewing", 2),
        ],
        combination_bonuses: &[],
    },
];
