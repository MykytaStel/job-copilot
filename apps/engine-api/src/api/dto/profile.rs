use serde::{Deserialize, Serialize};

use crate::domain::candidate::profile::{CandidateProfile, RoleScore};

#[derive(Deserialize)]
pub struct AnalyzeProfileRequest {
    pub raw_text: String,
}

#[derive(Serialize)]
pub struct RoleCandidateResponse {
    pub role: String,
    pub score: u32,
    pub confidence: u8,
    pub matched_signals: Vec<String>,
}

#[derive(Serialize)]
pub struct AnalyzeProfileResponse {
    pub summary: String,
    pub primary_role: String,
    pub seniority: String,
    pub skills: Vec<String>,
    pub keywords: Vec<String>,
    pub role_candidates: Vec<RoleCandidateResponse>,
    pub suggested_search_terms: Vec<String>,
}

impl From<RoleScore> for RoleCandidateResponse {
    fn from(role_score: RoleScore) -> Self {
        Self {
            role: role_score.role.to_string(),
            score: role_score.score,
            confidence: role_score.confidence,
            matched_signals: role_score.matched_signals,
        }
    }
}

impl From<CandidateProfile> for AnalyzeProfileResponse {
    fn from(profile: CandidateProfile) -> Self {
        let CandidateProfile {
            summary,
            primary_role,
            seniority,
            skills,
            keywords,
            role_candidates,
            suggested_search_terms,
        } = profile;

        Self {
            summary,
            primary_role: primary_role.to_string(),
            seniority,
            skills,
            keywords,
            role_candidates: role_candidates
                .into_iter()
                .map(RoleCandidateResponse::from)
                .collect(),
            suggested_search_terms,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::candidate::profile::{CandidateProfile, RoleScore};
    use crate::domain::role::RoleId;

    use super::AnalyzeProfileResponse;

    #[test]
    fn serializes_role_ids_as_snake_case_strings() {
        let response = AnalyzeProfileResponse::from(CandidateProfile {
            summary: "Summary".to_string(),
            primary_role: RoleId::ReactNativeDeveloper,
            seniority: "senior".to_string(),
            skills: vec!["react native".to_string()],
            keywords: vec!["mobile".to_string()],
            role_candidates: vec![RoleScore {
                role: RoleId::ReactNativeDeveloper,
                score: 30,
                confidence: 100,
                matched_signals: vec!["react native".to_string()],
            }],
            suggested_search_terms: vec!["react native developer".to_string()],
        });

        assert_eq!(response.primary_role, "react_native_developer");
        assert_eq!(response.role_candidates[0].role, "react_native_developer");
    }
}
