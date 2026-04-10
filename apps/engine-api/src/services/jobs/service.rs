use crate::domain::job::model::Job;

#[derive(Clone, Default)]
pub struct JobsService;

impl JobsService {
    pub fn new() -> Self {
        Self
    }

    pub fn get_mock_job(&self) -> Job {
        Job {
            id: "job-1".to_string(),
            title: "Frontend Developer".to_string(),
            company: "Example Inc".to_string(),
            location: "Kyiv".to_string(),
            matched_keywords: vec![
                "react".to_string(),
                "typescript".to_string(),
                "frontend".to_string(),
            ],
        }
    }
}
