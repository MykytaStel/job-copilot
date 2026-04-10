#[derive(Clone, Debug)]
pub struct Job {
    pub id: String,
    pub title: String,
    pub company: String,
    pub location: String,
    pub matched_keywords: Vec<String>,
}
