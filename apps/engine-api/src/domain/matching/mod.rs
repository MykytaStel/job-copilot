use crate::domain::role::RoleId;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JobFit {
    pub job_id: String,
    pub score: u8,
    pub matched_roles: Vec<RoleId>,
    pub matched_skills: Vec<String>,
    pub matched_keywords: Vec<String>,
    pub source_match: bool,
    pub work_mode_match: Option<bool>,
    pub region_match: Option<bool>,
    pub reasons: Vec<String>,
}
