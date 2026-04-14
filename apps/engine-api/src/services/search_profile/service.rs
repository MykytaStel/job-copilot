use crate::domain::candidate::profile::CandidateProfile;
use crate::domain::role::RoleId;
use crate::domain::search::profile::{SearchPreferences, SearchProfile};

#[derive(Clone, Default)]
pub struct SearchProfileService;

impl SearchProfileService {
    pub fn new() -> Self {
        Self
    }

    pub fn build(
        &self,
        analyzed_profile: &CandidateProfile,
        preferences: &SearchPreferences,
    ) -> SearchProfile {
        let mut target_roles: Vec<RoleId> = Vec::new();

        for role_candidate in &analyzed_profile.role_candidates {
            push_unique(&mut target_roles, role_candidate.role);
        }

        for preferred_role in &preferences.preferred_roles {
            push_unique(&mut target_roles, *preferred_role);
        }

        let mut search_terms = Vec::new();

        for term in &analyzed_profile.suggested_search_terms {
            push_unique(&mut search_terms, term.clone());
        }

        for preferred_role in &preferences.preferred_roles {
            push_unique(&mut search_terms, preferred_role.search_label());
        }

        for keyword in &preferences.include_keywords {
            push_unique(&mut search_terms, keyword.clone());
        }

        let mut exclude_terms = Vec::new();

        for keyword in &preferences.exclude_keywords {
            push_unique(&mut exclude_terms, keyword.clone());
        }

        SearchProfile {
            primary_role: analyzed_profile.primary_role,
            target_roles,
            seniority: analyzed_profile.seniority.clone(),
            target_regions: preferences.target_regions.clone(),
            work_modes: preferences.work_modes.clone(),
            search_terms,
            exclude_terms,
        }
    }
}

fn push_unique<T>(target: &mut Vec<T>, value: T)
where
    T: PartialEq,
{
    if !target.iter().any(|existing| existing == &value) {
        target.push(value);
    }
}
