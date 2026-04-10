use crate::domain::role::RoleId;

#[derive(Clone, Debug)]
pub struct RoleScore {
    pub role: RoleId,
    pub score: u32,
    pub confidence: u8,
    pub matched_signals: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct CandidateProfile {
    pub summary: String,
    pub primary_role: RoleId,
    pub seniority: String,
    pub skills: Vec<String>,
    pub keywords: Vec<String>,
    pub role_candidates: Vec<RoleScore>,
    pub suggested_search_terms: Vec<String>,
}
