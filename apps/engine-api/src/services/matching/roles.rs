use std::collections::BTreeMap;

use super::{
    PARTIAL_ROLE_MATCH_THRESHOLD, PreparedText, ROLE_CATALOG, ROLE_MISMATCH_PENALTY_WEIGHT,
    RoleAlignment, RoleId, SearchProfile, SearchRoleCandidate, normalize_text, push_unique_role,
    push_unique_string,
};

pub(super) fn infer_role_family_for_job(
    job: &super::Job,
    source: Option<&str>,
) -> Option<&'static str> {
    let prepared_text = PreparedText::new(&super::build_searchable_text_parts(job, source));
    let job_roles = collect_job_roles(&prepared_text);
    let mut families = BTreeMap::new();

    for role in job_roles {
        let Some(family) = role.family() else {
            continue;
        };

        *families.entry(family).or_insert(0usize) += 1;
    }

    families
        .into_iter()
        .max_by(|left, right| left.1.cmp(&right.1).then_with(|| right.0.cmp(left.0)))
        .map(|(family, _)| family)
}

pub(super) fn collect_target_roles(search_profile: &SearchProfile) -> Vec<RoleId> {
    let mut roles = Vec::new();
    push_unique_role(&mut roles, search_profile.primary_role);

    for role in &search_profile.target_roles {
        push_unique_role(&mut roles, *role);
    }

    roles
}

pub(super) fn role_matches(prepared_text: &PreparedText, role: RoleId) -> bool {
    if prepared_text.matches_signal(&role.search_label()) {
        return true;
    }

    if prepared_text.matches_signal(role.display_name()) {
        return true;
    }

    role.search_aliases()
        .iter()
        .any(|alias| prepared_text.matches_signal(alias))
}

pub(super) fn collect_role_terms(target_roles: &[RoleId]) -> Vec<String> {
    let mut terms = Vec::new();

    for role in target_roles {
        push_unique_string(&mut terms, normalize_text(&role.search_label()));
        push_unique_string(&mut terms, normalize_text(role.display_name()));

        for alias in role.search_aliases() {
            push_unique_string(&mut terms, normalize_text(alias));
        }
    }

    terms
}

pub(super) fn analyze_role_alignment(
    search_profile: &SearchProfile,
    prepared_text: &PreparedText,
    target_roles: &[RoleId],
) -> RoleAlignment {
    let job_roles = collect_job_roles(prepared_text);
    let matched_roles = target_roles
        .iter()
        .copied()
        .filter(|role| role_matches(prepared_text, *role))
        .collect::<Vec<_>>();
    let primary_overlap = best_role_overlap(search_profile.primary_role, &job_roles);
    let best_partial_match = best_role_pair(target_roles, &job_roles);
    let best_target_overlap = best_partial_match
        .map(|(_, _, overlap)| overlap)
        .unwrap_or(0.0);
    let candidate_overlap =
        weighted_role_candidate_overlap(&search_profile.role_candidates, &job_roles);
    let mismatch_penalty =
        compute_role_mismatch_penalty(target_roles, &job_roles, best_target_overlap);

    RoleAlignment {
        matched_roles,
        job_roles,
        primary_overlap,
        best_target_overlap,
        best_partial_match,
        candidate_overlap,
        mismatch_penalty,
    }
}

pub(super) fn collect_job_roles(prepared_text: &PreparedText) -> Vec<RoleId> {
    ROLE_CATALOG
        .iter()
        .filter(|metadata| !metadata.is_fallback && role_matches(prepared_text, metadata.id))
        .map(|metadata| metadata.id)
        .collect()
}

fn weighted_role_candidate_overlap(
    role_candidates: &[SearchRoleCandidate],
    job_roles: &[RoleId],
) -> f32 {
    let total_weight = role_candidates
        .iter()
        .map(|candidate| candidate.confidence as f32)
        .sum::<f32>();

    if total_weight <= 0.0 || job_roles.is_empty() {
        return 0.0;
    }

    let weighted_overlap = role_candidates
        .iter()
        .map(|candidate| best_role_overlap(candidate.role, job_roles) * candidate.confidence as f32)
        .sum::<f32>();

    (weighted_overlap / total_weight).min(1.0)
}

fn best_role_overlap(target_role: RoleId, job_roles: &[RoleId]) -> f32 {
    job_roles
        .iter()
        .map(|job_role| role_family_overlap(target_role, *job_role))
        .fold(0.0, f32::max)
}

fn best_role_pair(target_roles: &[RoleId], job_roles: &[RoleId]) -> Option<(RoleId, RoleId, f32)> {
    let mut best_match = None;

    for target_role in target_roles {
        for job_role in job_roles {
            let overlap = role_family_overlap(*target_role, *job_role);

            if best_match
                .as_ref()
                .map(|(_, _, best_overlap)| overlap > *best_overlap)
                .unwrap_or(true)
            {
                best_match = Some((*target_role, *job_role, overlap));
            }
        }
    }

    best_match
}

fn role_family_overlap(left: RoleId, right: RoleId) -> f32 {
    if left == right {
        return 1.0;
    }

    match (left, right) {
        (RoleId::FrontendEngineer, RoleId::FullstackEngineer)
        | (RoleId::FullstackEngineer, RoleId::FrontendEngineer) => 0.70,
        (RoleId::BackendEngineer, RoleId::FullstackEngineer)
        | (RoleId::FullstackEngineer, RoleId::BackendEngineer) => 0.70,
        (RoleId::MobileEngineer, RoleId::FrontendEngineer)
        | (RoleId::FrontendEngineer, RoleId::MobileEngineer) => 0.40,
        (RoleId::MobileEngineer, RoleId::FullstackEngineer)
        | (RoleId::FullstackEngineer, RoleId::MobileEngineer) => 0.35,
        (RoleId::BackendEngineer, RoleId::DevopsEngineer)
        | (RoleId::DevopsEngineer, RoleId::BackendEngineer) => 0.45,
        (RoleId::FullstackEngineer, RoleId::DevopsEngineer)
        | (RoleId::DevopsEngineer, RoleId::FullstackEngineer) => 0.40,
        (RoleId::DataEngineer, RoleId::MlEngineer) | (RoleId::MlEngineer, RoleId::DataEngineer) => {
            0.50
        }
        (RoleId::TechLead, RoleId::EngineeringManager)
        | (RoleId::EngineeringManager, RoleId::TechLead) => 0.55,
        _ if left.is_fallback() || right.is_fallback() => 0.0,
        _ if left.family().is_some() && left.family() == right.family() => 0.15,
        _ => 0.0,
    }
}

fn compute_role_mismatch_penalty(
    target_roles: &[RoleId],
    job_roles: &[RoleId],
    best_target_overlap: f32,
) -> f32 {
    if job_roles.is_empty()
        || target_roles.iter().all(|role| role.is_fallback())
        || best_target_overlap >= PARTIAL_ROLE_MATCH_THRESHOLD
    {
        return 0.0;
    }

    ROLE_MISMATCH_PENALTY_WEIGHT * (1.0 - best_target_overlap)
}
