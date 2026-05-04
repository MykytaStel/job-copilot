use std::sync::OnceLock;

use regex::Regex;

use super::text::normalize_text;

pub const SKILL_DICTIONARY: &[(&str, &[&str])] = &[
    ("React", &["react", "react.js", "reactjs"]),
    ("Vue", &["vue", "vue.js", "vuejs"]),
    ("TypeScript", &["typescript", "type script"]),
    ("Node.js", &["node.js", "nodejs", "node js"]),
    ("Python", &["python"]),
    ("Rust", &["rust"]),
    ("Go", &["go", "golang"]),
    ("Java", &["java"]),
    ("Kotlin", &["kotlin"]),
    ("PostgreSQL", &["postgresql", "postgres"]),
    ("MongoDB", &["mongodb", "mongo db"]),
    ("Redis", &["redis"]),
    ("Docker", &["docker"]),
    ("Kubernetes", &["kubernetes", "k8s"]),
    ("AWS", &["aws", "amazon web services"]),
    ("GCP", &["gcp", "google cloud platform"]),
    ("Azure", &["azure"]),
    ("Git", &["git"]),
    ("CI/CD", &["ci/cd", "cicd", "ci cd"]),
    ("GraphQL", &["graphql", "graph ql"]),
    ("REST", &["rest", "restful"]),
    ("FastAPI", &["fastapi", "fast api"]),
    ("Django", &["django"]),
    ("Spring Boot", &["spring boot"]),
    ("Terraform", &["terraform"]),
    ("Linux", &["linux"]),
];

pub fn extract_skills(description: &str) -> Vec<String> {
    let searchable = skill_searchable_text(description);
    skill_regexes()
        .iter()
        .filter_map(|(skill, re)| {
            if re.is_match(&searchable) {
                Some(skill.to_string())
            } else {
                None
            }
        })
        .collect()
}

fn skill_searchable_text(description: &str) -> String {
    let mut parts = vec![description.to_string()];
    parts.extend(explicit_skill_segments(description));
    normalize_text(&parts.join(" "))
}

fn explicit_skill_segments(description: &str) -> Vec<String> {
    description
        .lines()
        .map(str::trim)
        .filter_map(|line| {
            let (_, value) = line.split_once(':')?;
            if explicit_skill_prefix_re().is_match(line) {
                Some(value.trim().to_string())
            } else {
                None
            }
        })
        .collect()
}

fn explicit_skill_prefix_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?iu)^\s*(?:required|skills|вимоги|технології)\s*:")
            .expect("valid explicit skill prefix regex")
    })
}

fn skill_regexes() -> &'static Vec<(&'static str, Regex)> {
    static RE: OnceLock<Vec<(&'static str, Regex)>> = OnceLock::new();
    RE.get_or_init(|| {
        SKILL_DICTIONARY
            .iter()
            .map(|(skill, aliases)| {
                let pattern = aliases
                    .iter()
                    .map(|a| regex::escape(a))
                    .collect::<Vec<_>>()
                    .join("|");
                let re = Regex::new(&format!(
                    r"(?iu)(^|[^\p{{L}}\p{{N}}])({pattern})($|[^\p{{L}}\p{{N}}])"
                ))
                .expect("valid skill regex");
                (*skill, re)
            })
            .collect()
    })
}
