use crate::db::Database;
use crate::db::repositories::{
    ApplicationsRepository, JobsRepository, MatchResultsRepository, ProfilesRepository,
    ResumesRepository,
};
use crate::services::applications::ApplicationsService;
use crate::services::jobs::JobsService;
use crate::services::matching::MatchService;
use crate::services::profile::service::ProfileAnalysisService;
use crate::services::profiles::ProfilesService;
use crate::services::ranking::RankingService;
use crate::services::resumes::ResumesService;
use crate::services::search_profile::service::SearchProfileService;

#[derive(Clone)]
pub struct AppState {
    pub app_name: String,
    pub version: String,
    pub database: Database,
    pub profiles_service: ProfilesService,
    pub jobs_service: JobsService,
    pub applications_service: ApplicationsService,
    pub resumes_service: ResumesService,
    pub match_service: MatchService,
    pub profile_analysis_service: ProfileAnalysisService,
    pub ranking_service: RankingService,
    pub search_profile_service: SearchProfileService,
}

impl AppState {
    pub fn new(database: Database) -> Self {
        let profiles_repository = ProfilesRepository::new(database.clone());
        let jobs_repository = JobsRepository::new(database.clone());
        let applications_repository = ApplicationsRepository::new(database.clone());
        let resumes_repository = ResumesRepository::new(database.clone());
        let match_results_repository = MatchResultsRepository::new(database.clone());
        let profile_analysis_service = ProfileAnalysisService::new();

        Self {
            app_name: "engine-api".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            database,
            profiles_service: ProfilesService::new(profiles_repository),
            jobs_service: JobsService::new(jobs_repository),
            applications_service: ApplicationsService::new(applications_repository),
            resumes_service: ResumesService::new(resumes_repository),
            match_service: MatchService::new(
                match_results_repository,
                profile_analysis_service.clone(),
            ),
            profile_analysis_service,
            ranking_service: RankingService::new(),
            search_profile_service: SearchProfileService::new(),
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
            applications_service,
            resumes_service,
            match_service: MatchService::new(
                MatchResultsRepository::new(Database::disabled()),
                profile_analysis_service.clone(),
            ),
            profile_analysis_service,
            ranking_service: RankingService::new(),
            search_profile_service: SearchProfileService::new(),
        }
    }
}
