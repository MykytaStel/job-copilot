#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ApplicationSearchHit {
    pub id: String,
    pub job_id: String,
    pub resume_id: Option<String>,
    pub status: String,
    pub applied_at: Option<String>,
    pub due_date: Option<String>,
    pub updated_at: String,
    pub job_title: String,
    pub company_name: String,
}
