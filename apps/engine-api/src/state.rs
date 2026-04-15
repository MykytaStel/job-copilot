use crate::config::Config;
use crate::db::Database;
use crate::db::repositories::{
    ActivitiesRepository, ApplicationsRepository, FeedbackRepository, FitScoresRepository,
    JobsRepository, ProfilesRepository, ResumesRepository, TasksRepository, UserEventsRepository,
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
use crate::services::user_events::UserEventsService;

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
    pub user_events_service: UserEventsService,
    pub learned_reranker_enabled: bool,
}

impl AppState {
    pub fn new(database: Database) -> Self {
        Self::new_with_learned_reranker(database, learned_reranker_enabled_from_env())
    }

    pub fn new_with_config(database: Database, config: &Config) -> Self {
        Self::new_with_learned_reranker(database, config.learned_reranker_enabled)
    }

    fn new_with_learned_reranker(database: Database, learned_reranker_enabled: bool) -> Self {
        let profiles_repository = ProfilesRepository::new(database.clone());
        let jobs_repository = JobsRepository::new(database.clone());
        let salary_jobs_repository = JobsRepository::new(database.clone());
        let applications_repository = ApplicationsRepository::new(database.clone());
        let feedback_repository = FeedbackRepository::new(database.clone());
        let activities_repository = ActivitiesRepository::new(database.clone());
        let tasks_repository = TasksRepository::new(database.clone());
        let resumes_repository = ResumesRepository::new(database.clone());
        let fit_scores_repository = FitScoresRepository::new(database.clone());
        let user_events_repository = UserEventsRepository::new(database.clone());
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
            user_events_service: UserEventsService::new(user_events_repository),
            learned_reranker_enabled,
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
            user_events_service: UserEventsService::for_tests(
                crate::services::user_events::UserEventsServiceStub::default(),
            ),
            learned_reranker_enabled: true,
        }
    }

    #[cfg(test)]
    pub fn with_feedback_service(mut self, feedback_service: FeedbackService) -> Self {
        self.feedback_service = feedback_service;
        self
    }

    #[cfg(test)]
    pub fn with_user_events_service(mut self, user_events_service: UserEventsService) -> Self {
        self.user_events_service = user_events_service;
        self
    }

    #[cfg(test)]
    pub fn with_learned_reranker_enabled(mut self, enabled: bool) -> Self {
        self.learned_reranker_enabled = enabled;
        self
    }
}

fn learned_reranker_enabled_from_env() -> bool {
    std::env::var("LEARNED_RERANKER_ENABLED")
        .ok()
        .as_deref()
        .map(|value| {
            matches!(
                value.trim().to_ascii_lowercase().as_str(),
                "1" | "true" | "yes" | "on"
            )
        })
        .unwrap_or(true)
}
