use serde::Serialize;

use crate::domain::ranking::{FitScore, FitScoreComponents};

#[derive(Debug, Serialize)]
pub struct FitScoreResponse {
    pub job_id: String,
    pub total: u8,
    pub components: FitScoreComponentsResponse,
    pub matched_skills: Vec<String>,
    pub missing_skills: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct FitScoreComponentsResponse {
    pub skill_overlap: f32,
    pub seniority_alignment: f32,
    pub salary_overlap: f32,
    pub work_mode_match: f32,
    pub language_match: f32,
    pub recency_bonus: f32,
}

impl From<FitScore> for FitScoreResponse {
    fn from(score: FitScore) -> Self {
        Self {
            job_id: score.job_id,
            total: score.total,
            components: FitScoreComponentsResponse::from(score.components),
            matched_skills: score.matched_skills,
            missing_skills: score.missing_skills,
        }
    }
}

impl From<FitScoreComponents> for FitScoreComponentsResponse {
    fn from(c: FitScoreComponents) -> Self {
        Self {
            skill_overlap: c.skill_overlap,
            seniority_alignment: c.seniority_alignment,
            salary_overlap: c.salary_overlap,
            work_mode_match: c.work_mode_match,
            language_match: c.language_match,
            recency_bonus: c.recency_bonus,
        }
    }
}
