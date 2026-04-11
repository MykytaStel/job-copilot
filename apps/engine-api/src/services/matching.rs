use crate::db::repositories::{MatchResultsRepository, RepositoryError};
use crate::domain::job::model::Job;
use crate::domain::matching::model::MatchResult;
use crate::domain::resume::model::ResumeVersion;
use crate::services::profile::service::ProfileAnalysisService;

#[derive(Clone)]
pub struct MatchService {
    repository: MatchResultsRepository,
    profile_analysis_service: ProfileAnalysisService,
}

impl MatchService {
    pub fn new(
        repository: MatchResultsRepository,
        profile_analysis_service: ProfileAnalysisService,
    ) -> Self {
        Self {
            repository,
            profile_analysis_service,
        }
    }

    pub async fn get_for_job_and_resume(
        &self,
        job_id: &str,
        resume_id: &str,
    ) -> Result<Option<MatchResult>, RepositoryError> {
        self.repository
            .get_for_job_and_resume(job_id, resume_id)
            .await
    }

    pub async fn score_and_save(
        &self,
        job: &Job,
        resume: &ResumeVersion,
    ) -> Result<MatchResult, RepositoryError> {
        let analyzed_profile = self.profile_analysis_service.analyze(&resume.raw_text);
        let description = job.description_text.to_ascii_lowercase();

        let mut matched_skills = Vec::new();
        let mut missing_skills = Vec::new();

        for skill in analyzed_profile.skills {
            if description.contains(&skill.to_ascii_lowercase()) {
                matched_skills.push(skill);
            } else {
                missing_skills.push(skill);
            }
        }

        matched_skills.sort();
        matched_skills.dedup();
        missing_skills.sort();
        missing_skills.dedup();

        let total = matched_skills.len() + missing_skills.len();
        let score = if total == 0 {
            0
        } else {
            ((matched_skills.len() * 100) / total) as i32
        };

        let notes = if matched_skills.is_empty() {
            "No overlapping skills were detected between the active resume and the job description."
                .to_string()
        } else {
            format!(
                "{} matched skills found in the job description.",
                matched_skills.len()
            )
        };

        self.repository
            .save(&MatchResult {
                id: String::new(),
                job_id: job.id.clone(),
                resume_id: resume.id.clone(),
                score,
                matched_skills,
                missing_skills,
                notes,
                created_at: String::new(),
            })
            .await
    }
}
