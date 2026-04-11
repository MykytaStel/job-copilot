use serde::Serialize;

use crate::domain::matching::model::MatchResult;

#[derive(Debug, Serialize)]
pub struct MatchResultResponse {
    pub id: String,
    pub job_id: String,
    pub resume_id: String,
    pub score: i32,
    pub matched_skills: Vec<String>,
    pub missing_skills: Vec<String>,
    pub notes: String,
    pub created_at: String,
}

impl From<MatchResult> for MatchResultResponse {
    fn from(result: MatchResult) -> Self {
        Self {
            id: result.id,
            job_id: result.job_id,
            resume_id: result.resume_id,
            score: result.score,
            matched_skills: result.matched_skills,
            missing_skills: result.missing_skills,
            notes: result.notes,
            created_at: result.created_at,
        }
    }
}
