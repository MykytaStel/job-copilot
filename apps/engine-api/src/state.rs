use crate::config::Config;
use crate::db::Database;
use crate::db::repositories::{
    ActivitiesRepository, ApplicationsRepository, FeedbackRepository, FitScoresRepository,
    JobsRepository, NotificationsRepository, ProfileMlMetricsRepository, ProfileMlStateRepository,
    ProfilesRepository, ResumesRepository, TasksRepository, UserEventsRepository,
};
use crate::services::activities::ActivitiesService;
use crate::services::applications::ApplicationsService;
use crate::services::cv_tailoring::CvTailoringService;
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
use tracing::warn;

#[cfg(test)]
#[path = "state/test_support.rs"]
mod test_support;

#[derive(Clone)]
pub struct AppState {
    pub app_name: String,
    pub version: String,
    pub database: Database,
    pub ml_sidecar_base_url: String,
    pub profile_records: ProfileRecordsService,
    pub profile_ml_state: ProfileMlStateService,
    pub profile_ml_metrics: ProfileMlMetricsService,
    pub jobs_service: JobsService,
    pub search_ranking: SearchRankingService,
    pub applications_service: ApplicationsService,
    pub feedback_service: FeedbackService,
    pub activities_service: ActivitiesService,
    pub resumes_service: ResumesService,
    pub profile_analysis: ProfileAnalysisService,
    pub fit_scoring: FitScoringService,
    pub fit_scores_repository: FitScoresRepository,
    pub search_profile_builder: SearchProfileBuilder,
    pub followup_service: FollowUpService,
    pub notifications_service: NotificationsService,
    pub salary_service: SalaryService,
    pub user_events_service: UserEventsService,
    pub reranker_runtime_mode: RerankerRuntimeMode,
    pub learned_reranker_enabled: bool,
    pub trained_reranker_enabled: bool,
    pub trained_reranker_availability: TrainedRerankerAvailability,
    pub trained_reranker_model: Option<TrainedRerankerModel>,
    pub reranker_bootstrap: RerankerBootstrapService,
    pub cv_tailoring: CvTailoringService,
    pub jwt_secret: Option<String>,
    pub cors_allowed_origins: Vec<String>,
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
        let reranker_bootstrap = RerankerBootstrapService::new(
            config.ml_sidecar_base_url.clone(),
            config.ml_sidecar_timeout_seconds,
        )
        .expect("valid ML sidecar client configuration");
        let cv_tailoring = CvTailoringService::new(
            config.ml_sidecar_base_url.clone(),
            config.ml_sidecar_timeout_seconds,
            config.ml_sidecar_internal_token.clone(),
        )
        .expect("valid ML sidecar client configuration");

        let mut state = Self::new_with_rerankers(
            database,
            config.reranker_runtime_mode,
            config.learned_reranker_enabled,
            config.trained_reranker_enabled,
            trained_reranker_availability,
            trained_reranker_model,
            reranker_bootstrap,
            cv_tailoring,
            config.ml_sidecar_base_url.clone(),
        );
        state.jwt_secret = config.jwt_secret.clone();
        state.cors_allowed_origins = config.cors_allowed_origins.clone();
        state
    }

    fn new_with_rerankers(
        database: Database,
        reranker_runtime_mode: RerankerRuntimeMode,
        learned_reranker_enabled: bool,
        trained_reranker_enabled: bool,
        trained_reranker_availability: TrainedRerankerAvailability,
        trained_reranker_model: Option<TrainedRerankerModel>,
        reranker_bootstrap: RerankerBootstrapService,
        cv_tailoring: CvTailoringService,
        ml_sidecar_base_url: String,
    ) -> Self {
        let profiles_repository = ProfilesRepository::new(database.clone());
        let profile_ml_state_repository = ProfileMlStateRepository::new(database.clone());
        let profile_ml_metrics_repository = ProfileMlMetricsRepository::new(database.clone());
        let jobs_repository = JobsRepository::new(database.clone());
        let applications_repository = ApplicationsRepository::new(database.clone());
        let feedback_repository = FeedbackRepository::new(database.clone());
        let activities_repository = ActivitiesRepository::new(database.clone());
        let tasks_repository = TasksRepository::new(database.clone());
        let resumes_repository = ResumesRepository::new(database.clone());
        let fit_scores_repository = FitScoresRepository::new(database.clone());
        let notifications_repository = NotificationsRepository::new(database.clone());
        let user_events_repository = UserEventsRepository::new(database.clone());
        let profile_analysis = ProfileAnalysisService::new();
        let profile_records = ProfileRecordsService::new(profiles_repository);
        let search_ranking = SearchRankingService::new();
        let fit_scoring = FitScoringService::new();
        let search_profile_builder = SearchProfileBuilder::new();

        Self {
            app_name: "engine-api".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            database,
            ml_sidecar_base_url: ml_sidecar_base_url.trim_end_matches('/').to_string(),
            profile_records,
            profile_ml_state: ProfileMlStateService::new(profile_ml_state_repository),
            profile_ml_metrics: ProfileMlMetricsService::new(profile_ml_metrics_repository),
            jobs_service: JobsService::new(jobs_repository.clone()),
            search_ranking,
            applications_service: ApplicationsService::new(applications_repository),
            feedback_service: FeedbackService::new(feedback_repository),
            activities_service: ActivitiesService::new(activities_repository),
            resumes_service: ResumesService::new(resumes_repository),
            profile_analysis,
            fit_scoring,
            fit_scores_repository,
            search_profile_builder,
            followup_service: FollowUpService::new(tasks_repository),
            notifications_service: NotificationsService::new(notifications_repository),
            salary_service: SalaryService::new(jobs_repository.clone()),
            user_events_service: UserEventsService::new(user_events_repository),
            reranker_runtime_mode,
            learned_reranker_enabled,
            trained_reranker_enabled,
            trained_reranker_availability,
            trained_reranker_model,
            reranker_bootstrap,
            cv_tailoring,
            jwt_secret: None,
            cors_allowed_origins: Vec::new(),
        }
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
