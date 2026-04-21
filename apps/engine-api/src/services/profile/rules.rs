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
    "vue",
    "angular",
    "javascript",
    "node.js",
    "express",
    "python",
    "java",
    "spring boot",
    "django",
    "fastapi",
    "sql",
    "graphql",
    "rest api",
    "microservices",
    "distributed systems",
    "kafka",
    "rabbitmq",
    "elasticsearch",
    "mongodb",
    "redis",
    "swift",
    "kotlin",
    "ios",
    "android",
    "aws",
    "gcp",
    "azure",
    "docker",
    "kubernetes",
    "terraform",
    "helm",
    "ci/cd",
    "design system",
    "linux",
    "figma",
    "jira",
    "excel",
    "power bi",
    "testing",
    "qa",
    "machine learning",
    "pytorch",
    "tensorflow",
    "scikit-learn",
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
    "data",
    "machine learning",
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
        role: RoleId::MobileEngineer,
        signals: &[
            ("react native", 10),
            ("mobile", 4),
            ("ios", 3),
            ("android", 3),
            ("swift", 3),
            ("kotlin", 3),
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
        role: RoleId::FrontendEngineer,
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
        role: RoleId::BackendEngineer,
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
        role: RoleId::FullstackEngineer,
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
        role: RoleId::DataEngineer,
        signals: &[
            ("data engineer", 8),
            ("etl", 5),
            ("data pipeline", 5),
            ("analytics", 4),
            ("sql", 3),
            ("spark", 4),
            ("airflow", 4),
            ("data warehouse", 4),
            ("excel", 2),
            ("power bi", 3),
        ],
        combination_bonuses: &[CombinationBonusRule {
            label: "bonus: etl/pipeline + sql/spark",
            required_groups: &[
                SignalGroup {
                    signals: &["etl", "data pipeline", "airflow"],
                    min_matches: 1,
                },
                SignalGroup {
                    signals: &["sql", "spark", "data warehouse"],
                    min_matches: 1,
                },
            ],
            bonus: 5,
        }],
    },
    RoleRule {
        role: RoleId::MlEngineer,
        signals: &[
            ("machine learning", 8),
            ("ml", 5),
            ("deep learning", 6),
            ("pytorch", 5),
            ("tensorflow", 5),
            ("scikit-learn", 4),
            ("model training", 4),
            ("python", 2),
            ("data scientist", 5),
        ],
        combination_bonuses: &[CombinationBonusRule {
            label: "bonus: ml framework + python",
            required_groups: &[
                SignalGroup {
                    signals: &["pytorch", "tensorflow", "scikit-learn"],
                    min_matches: 1,
                },
                SignalGroup {
                    signals: &["python"],
                    min_matches: 1,
                },
            ],
            bonus: 5,
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
        role: RoleId::ProductDesigner,
        signals: &[
            ("product designer", 8),
            ("ui", 4),
            ("ux", 4),
            ("figma", 4),
            ("design system", 3),
            ("prototype", 2),
            ("wireframe", 2),
            ("user research", 3),
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
        role: RoleId::TechLead,
        signals: &[
            ("tech lead", 8),
            ("technical lead", 8),
            ("lead engineer", 7),
            ("lead developer", 7),
            ("architecture", 4),
            ("mentoring", 3),
            ("code review", 3),
            ("technical decisions", 3),
        ],
        combination_bonuses: &[CombinationBonusRule {
            label: "bonus: lead title + architecture/mentoring",
            required_groups: &[
                SignalGroup {
                    signals: &[
                        "tech lead",
                        "technical lead",
                        "lead engineer",
                        "lead developer",
                    ],
                    min_matches: 1,
                },
                SignalGroup {
                    signals: &["architecture", "mentoring", "technical decisions"],
                    min_matches: 1,
                },
            ],
            bonus: 5,
        }],
    },
    RoleRule {
        role: RoleId::EngineeringManager,
        signals: &[
            ("engineering manager", 9),
            ("head of engineering", 8),
            ("vp of engineering", 8),
            ("people management", 5),
            ("team lead", 4),
            ("hiring", 3),
            ("performance review", 3),
            ("engineering org", 3),
        ],
        combination_bonuses: &[],
    },
];
