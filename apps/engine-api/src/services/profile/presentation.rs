use crate::domain::candidate::profile::RoleScore;

use super::rules::MIN_ROLE_SCORE;

pub(crate) fn build_search_terms(
    role_candidates: &[RoleScore],
    skills: &[String],
    seniority: &str,
) -> Vec<String> {
    let mut terms = Vec::new();

    for candidate in role_candidates
        .iter()
        .filter(|candidate| candidate.score >= MIN_ROLE_SCORE)
        .take(2)
    {
        let readable_role = candidate.role.search_label();

        push_unique(&mut terms, readable_role.clone());

        if seniority != "unknown" {
            push_unique(&mut terms, format!("{} {}", seniority, readable_role));
        }

        for alias in candidate.role.search_aliases().iter().take(2) {
            push_unique(&mut terms, (*alias).to_string());
        }
    }

    for skill in skills.iter().take(5) {
        push_unique(&mut terms, skill.clone());
    }

    terms
}

pub(crate) fn build_summary(
    primary_role: crate::domain::role::RoleId,
    seniority: &str,
    role_candidates: &[RoleScore],
    skills: &[String],
) -> String {
    let readable_role = primary_role.display_name();
    let top_candidate = role_candidates.first();

    let seniority_label = if seniority == "unknown" {
        "Candidate".to_string()
    } else {
        format!("{} candidate", capitalize_first(seniority))
    };

    let leading_signals = top_candidate
        .map(|candidate| {
            if candidate.matched_signals.is_empty() {
                "general experience".to_string()
            } else {
                candidate
                    .matched_signals
                    .iter()
                    .take(4)
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(", ")
            }
        })
        .unwrap_or_else(|| "general experience".to_string());

    if top_candidate
        .map(|candidate| candidate.score == 0)
        .unwrap_or(true)
    {
        let skills_part = if skills.is_empty() {
            String::new()
        } else {
            format!(
                " Detected skills: {}.",
                skills
                    .iter()
                    .take(5)
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        };

        return format!(
            "{} has broad experience, but the current profile does not contain enough role-specific evidence for a confident match.{}",
            seniority_label, skills_part
        );
    }

    let confidence = top_candidate
        .map(|candidate| candidate.confidence)
        .unwrap_or(0);
    let confidence_phrase = confidence_phrase(confidence);
    let confidence_note = confidence_note(confidence);

    let alternative_roles = if role_candidates.len() > 1 {
        let items = role_candidates
            .iter()
            .skip(1)
            .take(2)
            .map(|candidate| candidate.role.display_name())
            .collect::<Vec<_>>();

        format!(" Alternative role directions: {}.", items.join(", "))
    } else {
        String::new()
    };

    format!(
        "{} {} {} based on signals: {}.{}{}",
        seniority_label,
        confidence_phrase,
        readable_role,
        leading_signals,
        confidence_note,
        alternative_roles
    )
}

fn confidence_phrase(confidence: u8) -> &'static str {
    if confidence >= 75 {
        "is strongly aligned with"
    } else if confidence >= 45 {
        "appears aligned with"
    } else {
        "shows tentative alignment with"
    }
}

fn confidence_note(confidence: u8) -> String {
    if confidence >= 75 {
        " The signal quality is strong.".to_string()
    } else if confidence >= 45 {
        " The signal mix is reasonable but not definitive.".to_string()
    } else {
        " The evidence is limited, so this should be treated as a tentative direction.".to_string()
    }
}

fn push_unique(target: &mut Vec<String>, value: String) {
    if !target.iter().any(|existing| existing == &value) {
        target.push(value);
    }
}

fn capitalize_first(value: &str) -> String {
    let mut chars = value.chars();

    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}
