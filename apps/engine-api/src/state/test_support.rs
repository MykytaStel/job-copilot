use crate::db::Database;
use crate::db::repositories::{
    FitScoresRepository, JobsRepository, NotificationsRepository, TasksRepository,
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

use super::AppState;

impl AppState {
    pub fn without_database() -> Self {
        Self::new(Database::disabled())
    }

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

    pub fn with_feedback_service(mut self, feedback_service: FeedbackService) -> Self {
        self.feedback_service = feedback_service;
        self
    }

    pub fn with_user_events_service(mut self, user_events_service: UserEventsService) -> Self {
        self.user_events_service = user_events_service;
        self
    }

    pub fn with_notifications_service(
        mut self,
        notifications_service: NotificationsService,
    ) -> Self {
        self.notifications_service = notifications_service;
        self
    }

    pub fn with_learned_reranker_enabled(mut self, enabled: bool) -> Self {
        self.reranker_runtime_mode =
            RerankerRuntimeMode::default_from_flags(enabled, self.trained_reranker_enabled);
        self.learned_reranker_enabled = enabled;
        self
    }

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

    pub fn with_trained_reranker_availability(
        mut self,
        availability: TrainedRerankerAvailability,
    ) -> Self {
        self.trained_reranker_availability = availability;
        self
    }

    pub fn with_reranker_runtime_mode(mut self, mode: RerankerRuntimeMode) -> Self {
        self.reranker_runtime_mode = mode;
        self
    }
}
