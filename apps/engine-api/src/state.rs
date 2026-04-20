use crate::config::Config;
use crate::db::Database;
use crate::db::repositories::{
    ActivitiesRepository, ApplicationsRepository, FeedbackRepository, FitScoresRepository,
    JobsRepository, NotificationsRepository, ProfilesRepository, ResumesRepository,
    TasksRepository, UserEventsRepository,
};
use crate::services::activities::ActivitiesService;
use crate::services::applications::ApplicationsService;
use crate::services::feedback::FeedbackService;
use crate::services::followup::FollowUpService;
use crate::services::jobs::JobsService;
use crate::services::matching::SearchMatchingService;
use crate::services::notifications::NotificationsService;
use crate::services::profile::service::ProfileAnalysisService;
use crate::services::profiles::ProfilesService;
use crate::services::ranking::RankingService;
use crate::services::ranking::runtime::{RerankerRuntimeMode, TrainedRerankerAvailability};
use crate::services::resumes::ResumesService;
use crate::services::salary::SalaryService;
use crate::services::search_profile::service::SearchProfileService;
use crate::services::trained_reranker::TrainedRerankerModel;
use crate::services::user_events::UserEventsService;
use tracing::warn;

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
    pub notifications_service: NotificationsService,
    pub salary_service: SalaryService,
    pub user_events_service: UserEventsService,
    pub reranker_runtime_mode: RerankerRuntimeMode,
    pub learned_reranker_enabled: bool,
    pub trained_reranker_enabled: bool,
    pub trained_reranker_availability: TrainedRerankerAvailability,
    pub trained_reranker_model: Option<TrainedRerankerModel>,
}

impl AppState {
    pub fn new(database: Database) -> Self {
        Self::new_with_config(database, &Config::from_env())
    }

    pub fn new_with_config(database: Database, config: &Config) -> Self {
        let (trained_reranker_model, trained_reranker_availability) = load_trained_reranker_model(
            config.trained_reranker_enabled,
            config.trained_reranker_model_path.as_deref(),
        );

        Self::new_with_rerankers(
            database,
            config.reranker_runtime_mode,
            config.learned_reranker_enabled,
            config.trained_reranker_enabled,
            trained_reranker_availability,
            trained_reranker_model,
        )
    }

    fn new_with_rerankers(
        database: Database,
        reranker_runtime_mode: RerankerRuntimeMode,
        learned_reranker_enabled: bool,
        trained_reranker_enabled: bool,
        trained_reranker_availability: TrainedRerankerAvailability,
        trained_reranker_model: Option<TrainedRerankerModel>,
    ) -> Self {
        let profiles_repository = ProfilesRepository::new(database.clone());
        let jobs_repository = JobsRepository::new(database.clone());
        let salary_jobs_repository = JobsRepository::new(database.clone());
        let applications_repository = ApplicationsRepository::new(database.clone());
        let feedback_repository = FeedbackRepository::new(database.clone());
        let activities_repository = ActivitiesRepository::new(database.clone());
        let tasks_repository = TasksRepository::new(database.clone());
        let resumes_repository = ResumesRepository::new(database.clone());
        let fit_scores_repository = FitScoresRepository::new(database.clone());
        let notifications_repository = NotificationsRepository::new(database.clone());
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
            notifications_service: NotificationsService::new(notifications_repository),
            salary_service: SalaryService::new(salary_jobs_repository),
            user_events_service: UserEventsService::new(user_events_repository),
            reranker_runtime_mode,
            learned_reranker_enabled,
            trained_reranker_enabled,
            trained_reranker_availability,
            trained_reranker_model,
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
            notifications_service: NotificationsService::new(NotificationsRepository::new(
                Database::disabled(),
            )),
            salary_service: SalaryService::new(JobsRepository::new(Database::disabled())),
            user_events_service: UserEventsService::for_tests(
                crate::services::user_events::UserEventsServiceStub::default(),
            ),
            reranker_runtime_mode: RerankerRuntimeMode::Learned,
            learned_reranker_enabled: true,
            trained_reranker_enabled: false,
            trained_reranker_availability: TrainedRerankerAvailability::DisabledByFlag,
            trained_reranker_model: None,
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
    pub fn with_notifications_service(
        mut self,
        notifications_service: NotificationsService,
    ) -> Self {
        self.notifications_service = notifications_service;
        self
    }

    #[cfg(test)]
    pub fn with_learned_reranker_enabled(mut self, enabled: bool) -> Self {
        self.reranker_runtime_mode =
            RerankerRuntimeMode::default_from_flags(enabled, self.trained_reranker_enabled);
        self.learned_reranker_enabled = enabled;
        self
    }

    #[cfg(test)]
    pub fn with_trained_reranker(
        mut self,
        enabled: bool,
        model: Option<TrainedRerankerModel>,
    ) -> Self {
        self.reranker_runtime_mode =
            RerankerRuntimeMode::default_from_flags(self.learned_reranker_enabled, enabled);
        self.trained_reranker_enabled = enabled;
        self.trained_reranker_availability = if !enabled {
            TrainedRerankerAvailability::DisabledByFlag
        } else if model.is_some() {
            TrainedRerankerAvailability::Ready
        } else {
            TrainedRerankerAvailability::MissingPath
        };
        self.trained_reranker_model = model;
        self
    }

    #[cfg(test)]
    pub fn with_trained_reranker_availability(
        mut self,
        availability: TrainedRerankerAvailability,
    ) -> Self {
        self.trained_reranker_availability = availability;
        self
    }

    #[cfg(test)]
    pub fn with_reranker_runtime_mode(mut self, mode: RerankerRuntimeMode) -> Self {
        self.reranker_runtime_mode = mode;
        self
    }
}

fn load_trained_reranker_model(
    enabled: bool,
    path: Option<&str>,
) -> (Option<TrainedRerankerModel>, TrainedRerankerAvailability) {
    if !enabled {
        return (None, TrainedRerankerAvailability::DisabledByFlag);
    }

    let Some(path) = path else {
        warn!("TRAINED_RERANKER_ENABLED is set but TRAINED_RERANKER_MODEL_PATH is empty");
        return (None, TrainedRerankerAvailability::MissingPath);
    };

    match TrainedRerankerModel::load(path) {
        Ok(model) => (Some(model), TrainedRerankerAvailability::Ready),
        Err(error) => {
            warn!(
                error = %error,
                model_path = path,
                "failed to load trained reranker artifact; continuing without v2 layer"
            );
            (None, TrainedRerankerAvailability::InvalidArtifact(error))
        }
    }
}
