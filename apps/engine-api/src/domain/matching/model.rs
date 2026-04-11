#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MatchResult {
    pub id: String,
    pub job_id: String,
    pub resume_id: String,
    pub score: i32,
    pub matched_skills: Vec<String>,
    pub missing_skills: Vec<String>,
    pub notes: String,
    pub created_at: String,
}
