pub const JOB_ALERT_SCORE_THRESHOLD: u8 = 60;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AlertProfile {
    pub primary_role: Option<String>,
    pub seniority: Option<String>,
    pub skills: Vec<String>,
    pub keywords: Vec<String>,
    pub preferred_roles: Vec<String>,
    pub include_keywords: Vec<String>,
    pub exclude_keywords: Vec<String>,
    pub work_modes: Vec<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AlertJob {
    pub title: String,
    pub company_name: String,
    pub location: Option<String>,
    pub remote_type: Option<String>,
    pub seniority: Option<String>,
    pub description_text: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AlertScore {
    pub score: u8,
    pub matched_signals: Vec<String>,
}

pub fn score_job_alert(profile: &AlertProfile, job: &AlertJob) -> AlertScore {
    let haystack = normalize(&format!(
        "{} {} {} {} {} {}",
        job.title,
        job.company_name,
        job.location.as_deref().unwrap_or_default(),
        job.remote_type.as_deref().unwrap_or_default(),
        job.seniority.as_deref().unwrap_or_default(),
        job.description_text
    ));
    let mut score = 0i16;
    let mut matched_signals = Vec::new();

    let mut role_terms = Vec::new();
    if let Some(primary_role) = profile.primary_role.as_deref() {
        push_role_terms(&mut role_terms, primary_role);
    }
    for role in &profile.preferred_roles {
        push_role_terms(&mut role_terms, role);
    }

    if let Some(matched_role) = first_matching_term(&haystack, &role_terms) {
        score += 35;
        push_unique(&mut matched_signals, format!("role:{matched_role}"));
    }

    let skill_matches = matching_terms(&haystack, &profile.skills);
    if !skill_matches.is_empty() {
        score += proportional_score(skill_matches.len(), profile.skills.len(), 30);
        for skill in skill_matches.into_iter().take(5) {
            push_unique(&mut matched_signals, format!("skill:{skill}"));
        }
    }

    let mut keywords = profile.keywords.clone();
    keywords.extend(profile.include_keywords.clone());
    let keyword_matches = matching_terms(&haystack, &keywords);
    if !keyword_matches.is_empty() {
        score += proportional_score(keyword_matches.len(), keywords.len(), 20);
        for keyword in keyword_matches.into_iter().take(5) {
            push_unique(&mut matched_signals, format!("keyword:{keyword}"));
        }
    }

    if work_mode_matches(&profile.work_modes, job.remote_type.as_deref()) {
        score += 10;
        if let Some(remote_type) = job.remote_type.as_deref() {
            push_unique(&mut matched_signals, format!("work_mode:{}", normalize(remote_type)));
        }
    }

    if seniority_matches(profile.seniority.as_deref(), job.seniority.as_deref()) {
        score += 5;
        if let Some(seniority) = job.seniority.as_deref() {
            push_unique(&mut matched_signals, format!("seniority:{}", normalize(seniority)));
        }
    }

    let exclude_matches = matching_terms(&haystack, &profile.exclude_keywords);
    if !exclude_matches.is_empty() {
        score -= 40;
        for term in exclude_matches.into_iter().take(3) {
            push_unique(&mut matched_signals, format!("excluded:{term}"));
        }
    }

    AlertScore {
        score: score.clamp(0, 100) as u8,
        matched_signals,
    }
}

pub fn alert_profile_has_signals(profile: &AlertProfile) -> bool {
    profile
        .primary_role
        .as_deref()
        .is_some_and(|value| !value.trim().is_empty())
        || !profile.skills.is_empty()
        || !profile.keywords.is_empty()
        || !profile.preferred_roles.is_empty()
        || !profile.include_keywords.is_empty()
}

fn push_role_terms(target: &mut Vec<String>, role: &str) {
    let normalized = normalize(role);
    if normalized.is_empty() {
        return;
    }

    push_unique(target, normalized.replace('_', " "));

    match normalized.as_str() {
        "backend_engineer" => {
            push_unique(target, "backend".to_string());
            push_unique(target, "back end".to_string());
            push_unique(target, "platform engineer".to_string());
        }
        "frontend_engineer" => {
            push_unique(target, "frontend".to_string());
            push_unique(target, "front end".to_string());
            push_unique(target, "web developer".to_string());
        }
        "fullstack_engineer" => {
            push_unique(target, "fullstack".to_string());
            push_unique(target, "full stack".to_string());
        }
        "mobile_engineer" => {
            push_unique(target, "mobile".to_string());
            push_unique(target, "ios".to_string());
            push_unique(target, "android".to_string());
            push_unique(target, "react native".to_string());
        }
        "devops_engineer" => {
            push_unique(target, "devops".to_string());
            push_unique(target, "sre".to_string());
            push_unique(target, "cloud engineer".to_string());
        }
        "qa_engineer" => {
            push_unique(target, "qa".to_string());
            push_unique(target, "quality assurance".to_string());
            push_unique(target, "test engineer".to_string());
        }
        "data_engineer" => {
            push_unique(target, "data engineer".to_string());
            push_unique(target, "analytics engineer".to_string());
        }
        "ml_engineer" => {
            push_unique(target, "machine learning".to_string());
            push_unique(target, "ml engineer".to_string());
            push_unique(target, "ai engineer".to_string());
        }
        "product_manager" => {
            push_unique(target, "product manager".to_string());
        }
        "project_manager" => {
            push_unique(target, "project manager".to_string());
        }
        "product_designer" => {
            push_unique(target, "product designer".to_string());
            push_unique(target, "ux designer".to_string());
            push_unique(target, "ui designer".to_string());
        }
        "tech_lead" => {
            push_unique(target, "tech lead".to_string());
            push_unique(target, "lead engineer".to_string());
        }
        "engineering_manager" => {
            push_unique(target, "engineering manager".to_string());
            push_unique(target, "head of engineering".to_string());
        }
        _ => {}
    }
}

fn first_matching_term(haystack: &str, terms: &[String]) -> Option<String> {
    terms
        .iter()
        .map(|term| normalize(term))
        .find(|term| term.len() >= 2 && contains_term(haystack, term))
}

fn matching_terms(haystack: &str, terms: &[String]) -> Vec<String> {
    let mut matches = Vec::new();

    for term in terms {
        let normalized = normalize(term);
        if normalized.len() >= 2 && contains_term(haystack, &normalized) {
            push_unique(&mut matches, normalized);
        }
    }

    matches
}

fn contains_term(haystack: &str, term: &str) -> bool {
    haystack.contains(term)
}

fn proportional_score(matches: usize, total: usize, max_score: i16) -> i16 {
    if total == 0 || matches == 0 {
        return 0;
    }

    ((matches as f32 / total as f32) * f32::from(max_score)).round() as i16
}

fn work_mode_matches(preferred_modes: &[String], remote_type: Option<&str>) -> bool {
    if preferred_modes.is_empty() {
        return false;
    }

    let Some(remote_type) = remote_type.map(normalize) else {
        return false;
    };

    preferred_modes
        .iter()
        .map(|mode| normalize(mode))
        .any(|mode| mode == remote_type || remote_type.contains(&mode))
}

fn seniority_matches(profile_seniority: Option<&str>, job_seniority: Option<&str>) -> bool {
    let Some(profile_seniority) = profile_seniority.map(normalize) else {
        return false;
    };
    let Some(job_seniority) = job_seniority.map(normalize) else {
        return false;
    };

    !profile_seniority.is_empty()
        && !job_seniority.is_empty()
        && (profile_seniority == job_seniority
            || profile_seniority.contains(&job_seniority)
            || job_seniority.contains(&profile_seniority))
}

fn normalize(value: &str) -> String {
    value
        .to_lowercase()
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || character == '_' {
                character
            } else {
                ' '
            }
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn push_unique(target: &mut Vec<String>, value: String) {
    if !value.trim().is_empty() && !target.iter().any(|existing| existing == &value) {
        target.push(value);
    }
}

#[cfg(test)]
mod tests {
    use super::{AlertJob, AlertProfile, JOB_ALERT_SCORE_THRESHOLD, score_job_alert};

    #[test]
    fn scores_strong_matching_alert_above_threshold() {
        let profile = AlertProfile {
            primary_role: Some("backend_engineer".to_string()),
            seniority: Some("senior".to_string()),
            skills: vec!["rust".to_string(), "postgres".to_string()],
            keywords: vec!["platform".to_string()],
            work_modes: vec!["remote".to_string()],
            ..AlertProfile::default()
        };
        let job = AlertJob {
            title: "Senior Backend Platform Engineer".to_string(),
            company_name: "SignalHire".to_string(),
            remote_type: Some("remote".to_string()),
            seniority: Some("senior".to_string()),
            description_text: "Build Rust services on Postgres.".to_string(),
            ..AlertJob::default()
        };

        let score = score_job_alert(&profile, &job);

        assert!(score.score > JOB_ALERT_SCORE_THRESHOLD);
        assert!(score.matched_signals.iter().any(|signal| signal == "skill:rust"));
    }

    #[test]
    fn exclude_terms_lower_alert_score() {
        let profile = AlertProfile {
            primary_role: Some("backend_engineer".to_string()),
            skills: vec!["rust".to_string()],
            exclude_keywords: vec!["gambling".to_string()],
            ..AlertProfile::default()
        };
        let job = AlertJob {
            title: "Backend Engineer".to_string(),
            company_name: "BetCo".to_string(),
            description_text: "Rust backend work for gambling products.".to_string(),
            ..AlertJob::default()
        };

        let score = score_job_alert(&profile, &job);

        assert!(score.score <= JOB_ALERT_SCORE_THRESHOLD);
        assert!(
            score
                .matched_signals
                .iter()
                .any(|signal| signal == "excluded:gambling")
        );
    }
}
