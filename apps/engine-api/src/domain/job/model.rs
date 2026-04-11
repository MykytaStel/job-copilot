#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Job {
    pub id: String,
    pub title: String,
    pub company_name: String,
    pub remote_type: Option<String>,
    pub seniority: Option<String>,
    pub description_text: String,
    pub salary_min: Option<i32>,
    pub salary_max: Option<i32>,
    pub salary_currency: Option<String>,
    pub posted_at: Option<String>,
    pub last_seen_at: String,
    pub is_active: bool,
}
