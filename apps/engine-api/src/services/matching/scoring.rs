use super::{
    JobScorePenalty, LOW_SIGNAL_TERMS, PARTIAL_ROLE_MATCH_THRESHOLD, RoleAlignment, SearchProfile,
    SeniorityAlignment, TargetRegion, extract_significant_tokens, job_source, normalize_text,
};

pub(super) fn build_reasons(
    search_profile: &SearchProfile,
    job: &super::JobView,
    input: BuildReasonsInput<'_>,
) -> Vec<String> {
    let mut reasons = Vec::new();

    if input.role_alignment.matched_roles.is_empty() {
        if let Some((target_role, job_role, overlap)) = input.role_alignment.best_partial_match {
            if overlap >= PARTIAL_ROLE_MATCH_THRESHOLD {
                reasons.push(format!(
                    "Role family overlap matched {} with {}",
                    target_role, job_role
                ));
            } else {
                reasons.push(
                    "No target role signals matched the job title or description".to_string(),
                );
            }
        } else {
            reasons.push("No target role signals matched the job title or description".to_string());
        }
    } else {
        reasons.push(format!(
            "Matched target roles: {}",
            input
                .role_alignment
                .matched_roles
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }

    if let Some(confidence) = search_profile.primary_role_confidence {
        reasons.push(format!(
            "Primary role confidence carried into ranking: {}% for {}",
            confidence, search_profile.primary_role
        ));
    }

    if !input.matched_profile_skills.is_empty() {
        reasons.push(format!(
            "Matched profile skills: {}",
            input.matched_profile_skills.join(", ")
        ));
    }

    if !input.matched_profile_keywords.is_empty() {
        reasons.push(format!(
            "Matched profile keywords: {}",
            input.matched_profile_keywords.join(", ")
        ));
    }

    if !input.matched_search_terms.is_empty() {
        reasons.push(format!(
            "Matched search terms: {}",
            input.matched_search_terms.join(", ")
        ));
    }

    if search_profile.allowed_sources.is_empty() {
        reasons.push("All sources are allowed".to_string());
    } else if let Some(source) = job_source(job) {
        reasons.push(format!(
            "Source matched allowed sources: {}",
            source.canonical_key()
        ));
    } else {
        reasons.push("Job source was unavailable".to_string());
    }

    if !search_profile.work_modes.is_empty() {
        match input.work_mode_match {
            Some(true) => reasons.push("Work mode matched the search profile".to_string()),
            Some(false) => reasons.push("Work mode mismatch penalty applied".to_string()),
            None => reasons.push("Work mode could not be inferred from the job".to_string()),
        }
    }

    if !search_profile.target_regions.is_empty() {
        match input.region_match {
            Some(true) => reasons.push("Target region matched the job text".to_string()),
            Some(false) => reasons.push("Target region did not match the job text".to_string()),
            None => reasons.push("Region could not be inferred from the job".to_string()),
        }
    }

    match (
        input.seniority_alignment.normalized_profile.as_deref(),
        input.seniority_alignment.normalized_job.as_deref(),
    ) {
        (Some(profile_seniority), Some(job_seniority)) if profile_seniority == job_seniority => {
            reasons.push(format!("Matched seniority: {}", profile_seniority));
        }
        (Some(profile_seniority), Some(job_seniority)) => reasons.push(format!(
            "Seniority mismatch penalty applied: profile {} vs job {}",
            profile_seniority, job_seniority
        )),
        _ => {}
    }

    if !input.matched_exclude_terms.is_empty() {
        reasons.push(format!(
            "Exclude term penalty applied: {}",
            input.matched_exclude_terms.join(", ")
        ));
    }

    if input.role_alignment.mismatch_penalty > 0.0 && !input.role_alignment.job_roles.is_empty() {
        reasons.push(format!(
            "Role mismatch penalty applied: strongest job roles {}",
            input
                .role_alignment
                .job_roles
                .iter()
                .take(3)
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }

    if input.matched_profile_skills.is_empty()
        && !search_profile.profile_skills.is_empty()
        && input.role_alignment.matched_roles.is_empty()
        && input.matched_search_terms.is_empty()
    {
        reasons.push("No strong profile evidence matched the job text".to_string());
    }

    reasons
}

pub(super) struct BuildReasonsInput<'a> {
    pub(super) role_alignment: &'a RoleAlignment,
    pub(super) matched_profile_skills: &'a [String],
    pub(super) matched_profile_keywords: &'a [String],
    pub(super) matched_search_terms: &'a [String],
    pub(super) matched_exclude_terms: &'a [String],
    pub(super) work_mode_match: Option<bool>,
    pub(super) region_match: Option<bool>,
    pub(super) seniority_alignment: &'a SeniorityAlignment,
}

pub(super) fn confidence_factor(confidence: Option<u8>) -> f32 {
    match confidence {
        Some(value) => (0.45 + (value as f32 / 100.0) * 0.55).min(1.0),
        None => 0.60,
    }
}

pub(super) fn weighted_overlap_ratio(matched_strength: f32, total: f32) -> f32 {
    if total == 0.0 {
        0.0
    } else {
        (matched_strength / total).min(1.0)
    }
}

pub(super) fn penalty_entry(
    kind: &str,
    raw_penalty: f32,
    reason: Option<String>,
) -> Option<JobScorePenalty> {
    let reason = reason?;

    let score_delta = -((raw_penalty.round() as i16).max(0));
    if score_delta == 0 {
        return None;
    }

    Some(JobScorePenalty {
        kind: kind.to_string(),
        score_delta,
        reason,
    })
}

pub(super) fn push_unique_role(target: &mut Vec<super::RoleId>, value: super::RoleId) {
    if !target.contains(&value) {
        target.push(value);
    }
}

pub(super) fn push_unique_region(target: &mut Vec<TargetRegion>, value: TargetRegion) {
    if !target.contains(&value) {
        target.push(value);
    }
}

pub(super) fn push_unique_string(target: &mut Vec<String>, value: String) {
    if value.is_empty() || target.iter().any(|existing| existing == &value) {
        return;
    }

    target.push(value);
}

pub(super) fn push_ignored_term(target: &mut Vec<String>, value: &str) {
    let normalized = normalize_text(value);

    if normalized.is_empty() {
        return;
    }

    push_unique_string(target, normalized.clone());

    let canonical_tokens = extract_significant_tokens(&normalized);
    if canonical_tokens.is_empty() {
        return;
    }

    push_unique_string(target, canonical_tokens.join(" "));
}

pub(super) fn is_low_signal_term(token: &str) -> bool {
    !token.contains('_') && LOW_SIGNAL_TERMS.iter().any(|value| value == &token)
}
