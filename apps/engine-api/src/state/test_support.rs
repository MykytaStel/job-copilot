use crate::db::Database;
use crate::db::repositories::{
    FitScoresRepository, JobsRepository, NotificationsRepository, TasksRepository,
};
use crate::services::activities::ActivitiesService;
use crate::services::applications::ApplicationsService;
use crate::services::feedback::FeedbackService;
use crate::services::fit_scoring::FitScoringService;
use crate::services::followup::FollowUpService;
use crate::services::jobs::JobsService;
use crate::services::notifications::NotificationsService;
use crate::services::profile_analysis::ProfileAnalysisService;
use crate::services::profile_ml_metrics::ProfileMlMetricsService;
use crate::services::profile_ml_state::ProfileMlStateService;
use crate::services::profile_records::ProfileRecordsService;
use crate::services::reranker_bootstrap::RerankerBootstrapService;
use crate::services::resumes::ResumesService;
use crate::services::salary::SalaryService;
use crate::services::search_profile_builder::SearchProfileBuilder;
use crate::services::search_ranking::SearchRankingService;
use crate::services::search_ranking::runtime::{RerankerRuntimeMode, TrainedRerankerAvailability};
use crate::services::trained_reranker::TrainedRerankerModel;
use crate::services::user_events::UserEventsService;

use super::AppState;

impl AppState {
    pub fn without_database() -> Self {
        Self::new(Database::disabled())
    }

    pub fn for_services(
        profile_records: ProfileRecordsService,
        jobs_service: JobsService,
        applications_service: ApplicationsService,
        resumes_service: ResumesService,
    ) -> Self {
        let profile_analysis = ProfileAnalysisService::new();

        Self {
            app_name: "engine-api".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            database: Database::disabled(),
            profile_records,
            profile_ml_state: ProfileMlStateService::for_tests(
                crate::services::profile_ml_state::ProfileMlStateServiceStub::default(),
            ),
            profile_ml_metrics: ProfileMlMetricsService::for_tests(
                crate::services::profile_ml_metrics::ProfileMlMetricsServiceStub::default(),
            ),
            jobs_service,
            search_ranking: SearchRankingService::new(),
            applications_service,
            feedback_service: FeedbackService::for_tests(
                crate::services::feedback::FeedbackServiceStub::default(),
            ),
            activities_service: ActivitiesService::for_tests(
                crate::services::activities::ActivitiesServiceStub::default(),
            ),
            resumes_service,
            profile_analysis,
            fit_scoring: FitScoringService::new(),
            fit_scores_repository: FitScoresRepository::new(Database::disabled()),
            search_profile_builder: SearchProfileBuilder::new(),
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
            reranker_bootstrap: RerankerBootstrapService::new(
                "http://localhost:8000".to_string(),
                15,
            )
            .expect("test ML sidecar client should build"),
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

    pub fn with_profile_ml_state_service(
        mut self,
        profile_ml_state: ProfileMlStateService,
    ) -> Self {
        self.profile_ml_state = profile_ml_state;
        self
    }

    pub fn with_profile_ml_metrics_service(
        mut self,
        profile_ml_metrics: ProfileMlMetricsService,
    ) -> Self {
        self.profile_ml_metrics = profile_ml_metrics;
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

    pub fn with_reranker_bootstrap_service(
        mut self,
        reranker_bootstrap: RerankerBootstrapService,
    ) -> Self {
        self.reranker_bootstrap = reranker_bootstrap;
        self
    }
}
