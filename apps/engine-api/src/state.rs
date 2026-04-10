use crate::db::Database;
use crate::services::jobs::service::JobsService;
use crate::services::profile::service::ProfileAnalysisService;
use crate::services::search_profile::service::SearchProfileService;

#[derive(Clone)]
pub struct AppState {
    pub app_name: String,
    pub version: String,
    pub database: Database,
    pub jobs_service: JobsService,
    pub profile_analysis_service: ProfileAnalysisService,
    pub search_profile_service: SearchProfileService,
}

impl AppState {
    pub fn new(database: Database) -> Self {
        Self {
            app_name: "engine-api".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            database,
            jobs_service: JobsService::new(),
            profile_analysis_service: ProfileAnalysisService::new(),
            search_profile_service: SearchProfileService::new(),
        }
    }

    #[cfg(test)]
    pub fn without_database() -> Self {
        Self::new(Database::disabled())
    }
}
