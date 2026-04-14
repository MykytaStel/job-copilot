use crate::db::Database;
use crate::db::repositories::{
    ActivitiesRepository, ApplicationsRepository, FeedbackRepository, FitScoresRepository,
    JobsRepository, ProfilesRepository, ResumesRepository, TasksRepository,
};
use crate::services::activities::ActivitiesService;
use crate::services::applications::ApplicationsService;
use crate::services::feedback::FeedbackService;
use crate::services::followup::FollowUpService;
use crate::services::jobs::JobsService;
use crate::services::matching::SearchMatchingService;
use crate::services::profile::service::ProfileAnalysisService;
use crate::services::profiles::ProfilesService;
use crate::services::ranking::RankingService;
use crate::services::resumes::ResumesService;
use crate::services::salary::SalaryService;
use crate::services::search_profile::service::SearchProfileService;

#[derive(Clone)]
pub struct AppState {
    pub app_name: String,
    pub version: String,
    pub database: Database,
    pub profiles_service: ProfilesService,
    pub jobs_service: JobsService,
    pub search_matching_service: SearchMatchingService,
    pub applications_service: ApplicationsService,
    pub feedback_service: FeedbackService,
    pub activities_service: ActivitiesService,
    pub resumes_service: ResumesService,
    pub profile_analysis_service: ProfileAnalysisService,
    pub ranking_service: RankingService,
    pub fit_scores_repository: FitScoresRepository,
    pub search_profile_service: SearchProfileService,
    pub followup_service: FollowUpService,
    pub salary_service: SalaryService,
}

impl AppState {
    pub fn new(database: Database) -> Self {
        let profiles_repository = ProfilesRepository::new(database.clone());
        let jobs_repository = JobsRepository::new(database.clone());
        let salary_jobs_repository = JobsRepository::new(database.clone());
        let applications_repository = ApplicationsRepository::new(database.clone());
        let feedback_repository = FeedbackRepository::new(database.clone());
        let activities_repository = ActivitiesRepository::new(database.clone());
        let tasks_repository = TasksRepository::new(database.clone());
        let resumes_repository = ResumesRepository::new(database.clone());
        let fit_scores_repository = FitScoresRepository::new(database.clone());
        let profile_analysis_service = ProfileAnalysisService::new();

        Self {
            app_name: "engine-api".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            database,
            profiles_service: ProfilesService::new(profiles_repository),
            jobs_service: JobsService::new(jobs_repository),
            search_matching_service: SearchMatchingService::new(),
            applications_service: ApplicationsService::new(applications_repository),
            feedback_service: FeedbackService::new(feedback_repository),
            activities_service: ActivitiesService::new(activities_repository),
            resumes_service: ResumesService::new(resumes_repository),
            profile_analysis_service,
            ranking_service: RankingService::new(),
            fit_scores_repository,
            search_profile_service: SearchProfileService::new(),
            followup_service: FollowUpService::new(tasks_repository),
            salary_service: SalaryService::new(salary_jobs_repository),
        }
    }

    #[cfg(test)]
    pub fn without_database() -> Self {
        Self::new(Database::disabled())
    }

    #[cfg(test)]
    pub fn for_services(
        profiles_service: ProfilesService,
        jobs_service: JobsService,
        applications_service: ApplicationsService,
        resumes_service: ResumesService,
    ) -> Self {
        let profile_analysis_service = ProfileAnalysisService::new();

        Self {
            app_name: "engine-api".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            database: Database::disabled(),
            profiles_service,
            jobs_service,
            search_matching_service: SearchMatchingService::new(),
            applications_service,
            feedback_service: FeedbackService::for_tests(
                crate::services::feedback::FeedbackServiceStub::default(),
            ),
            activities_service: ActivitiesService::for_tests(
                crate::services::activities::ActivitiesServiceStub::default(),
            ),
            resumes_service,
            profile_analysis_service,
            ranking_service: RankingService::new(),
            fit_scores_repository: FitScoresRepository::new(Database::disabled()),
            search_profile_service: SearchProfileService::new(),
            followup_service: FollowUpService::new(TasksRepository::new(Database::disabled())),
            salary_service: SalaryService::new(JobsRepository::new(Database::disabled())),
        }
    }

    #[cfg(test)]
    pub fn with_feedback_service(mut self, feedback_service: FeedbackService) -> Self {
        self.feedback_service = feedback_service;
        self
    }
}
