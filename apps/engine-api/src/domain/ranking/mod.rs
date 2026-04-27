#[derive(Clone, Debug)]
pub struct FitScore {
    pub job_id: String,
    pub total: u8,
    pub components: FitScoreComponents,
    pub matched_skills: Vec<String>,
    pub missing_skills: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct FitScoreComponents {
    pub skill_overlap: f32,
    pub seniority_alignment: f32,
    pub salary_overlap: f32,
    pub work_mode_match: f32,
    pub language_match: f32,
    pub recency_bonus: f32,
}
