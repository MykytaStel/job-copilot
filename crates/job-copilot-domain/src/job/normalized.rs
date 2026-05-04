use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct NormalizedJob {
    pub id: String,
    #[serde(default)]
    pub duplicate_of: Option<String>,
    pub title: String,
    pub company_name: String,
    #[serde(default)]
    pub company_meta: Option<CompanyMeta>,
    pub location: Option<String>,
    pub remote_type: Option<String>,
    pub seniority: Option<String>,
    pub description_text: String,
    #[serde(default)]
    pub extracted_skills: Vec<String>,
    pub salary_min: Option<i32>,
    pub salary_max: Option<i32>,
    pub salary_currency: Option<String>,
    #[serde(default)]
    pub salary_usd_min: Option<i32>,
    #[serde(default)]
    pub salary_usd_max: Option<i32>,
    #[serde(default)]
    pub quality_score: Option<i32>,
    pub posted_at: Option<String>,
    pub last_seen_at: String,
    #[serde(default = "default_true")]
    pub is_active: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct CompanyMeta {
    pub size_hint: Option<String>,
    pub industry_hint: Option<String>,
    pub url: Option<String>,
}

pub fn default_true() -> bool {
    true
}
