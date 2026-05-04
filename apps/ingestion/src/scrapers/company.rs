use std::sync::OnceLock;

use regex::Regex;

use crate::models::{CompanyMeta, NormalizedJob};

use super::text::{normalize_text, normalized_non_empty};

const COMPANY_INDUSTRY_DICTIONARY: &[(&str, &[&str])] = &[
    (
        "fintech",
        &["fintech", "financial technology", "payments", "banking"],
    ),
    (
        "edtech",
        &["edtech", "education technology", "e-learning", "elearning"],
    ),
    (
        "e-commerce",
        &[
            "e-commerce",
            "ecommerce",
            "e commerce",
            "marketplace",
            "retail tech",
        ],
    ),
    (
        "outsourcing",
        &[
            "outsourcing",
            "outstaffing",
            "software development company",
            "service company",
        ],
    ),
    (
        "healthtech",
        &["healthtech", "healthcare", "medical technology"],
    ),
    ("gamedev", &["gamedev", "game development", "gaming studio"]),
];

pub fn infer_company_meta(description: &str, company_url: Option<&str>) -> Option<CompanyMeta> {
    let size_hint = infer_company_size_hint(description);
    let industry_hint = infer_company_industry_hint(description);
    let url = company_url.and_then(|value| normalized_non_empty(Some(value)));

    if size_hint.is_none() && industry_hint.is_none() && url.is_none() {
        None
    } else {
        Some(CompanyMeta {
            size_hint,
            industry_hint,
            url,
        })
    }
}

pub fn infer_company_size_hint(description: &str) -> Option<String> {
    if let Some(captures) = company_employee_range_re().captures(description) {
        let start = captures.get(1)?.as_str().replace([' ', ','], "");
        let end = captures.get(2)?.as_str().replace([' ', ','], "");
        return Some(format!("{start}-{end} employees"));
    }

    if company_startup_re().is_match(description) {
        Some("startup".to_string())
    } else if company_enterprise_re().is_match(description) {
        Some("enterprise".to_string())
    } else {
        None
    }
}

pub fn infer_company_industry_hint(description: &str) -> Option<String> {
    for (label, re) in company_industry_regexes() {
        if re.is_match(description) {
            return Some(label.to_string());
        }
    }
    None
}

pub fn normalize_company_name(value: &str) -> Option<String> {
    let cleaned = normalize_text(value);
    if cleaned.is_empty() {
        return None;
    }

    let lowered = cleaned.to_lowercase();
    if matches!(
        lowered.as_str(),
        "unknown" | "n/a" | "na" | "company not specified" | "компанія не вказана"
    ) {
        return None;
    }

    Some(cleaned)
}

pub fn compute_job_quality_score(job: &NormalizedJob) -> i32 {
    let mut score = 0;

    if normalize_text(&job.description_text).chars().count() >= 200 {
        score += 30;
    }

    if has_salary_info(job) {
        score += 20;
    }

    if job.extracted_skills.len() >= 3 {
        score += 20;
    }

    if normalized_non_empty(job.seniority.as_deref()).is_some() {
        score += 10;
    }

    if normalized_non_empty(job.remote_type.as_deref()).is_some() {
        score += 10;
    }

    if normalize_company_name(&job.company_name).is_some() {
        score += 10;
    }

    score
}

fn has_salary_info(job: &NormalizedJob) -> bool {
    job.salary_min.is_some()
        || job.salary_max.is_some()
        || job.salary_usd_min.is_some()
        || job.salary_usd_max.is_some()
}

fn company_employee_range_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r"(?iu)\b(\d[\d\s,]{0,8})\s*[-–—]\s*(\d[\d\s,]{0,8})\s*(?:employees|people|specialists|engineers|фахівц(?:ів|і)|співробітник(?:ів|и)|працівник(?:ів|и))\b",
        )
        .expect("valid company employee range regex")
    })
}

fn company_startup_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?iu)(^|[^\p{L}\p{N}])(?:startup|start-up|стартап)($|[^\p{L}\p{N}])")
            .expect("valid startup regex")
    })
}

fn company_enterprise_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?iu)(^|[^\p{L}\p{N}])(?:enterprise|корпорац(?:ія|ії)|міжнародна компанія|large company|global company)($|[^\p{L}\p{N}])")
            .expect("valid enterprise regex")
    })
}

fn company_industry_regexes() -> &'static Vec<(&'static str, Regex)> {
    static RE: OnceLock<Vec<(&'static str, Regex)>> = OnceLock::new();
    RE.get_or_init(|| {
        COMPANY_INDUSTRY_DICTIONARY
            .iter()
            .map(|(label, aliases)| {
                let pattern = aliases
                    .iter()
                    .map(|a| regex::escape(a))
                    .collect::<Vec<_>>()
                    .join("|");
                let re = Regex::new(&format!(
                    r"(?iu)(^|[^\p{{L}}\p{{N}}])({pattern})($|[^\p{{L}}\p{{N}}])"
                ))
                .expect("valid company industry regex");
                (*label, re)
            })
            .collect()
    })
}
