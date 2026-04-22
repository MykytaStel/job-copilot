#[cfg(test)]
#[path = "profile_ml_metrics/stub.rs"]
mod stub;

#[cfg(test)]
use std::sync::Arc;

use crate::db::repositories::{ProfileMlMetricsRepository, RepositoryError};
use crate::domain::profile::ml::{CreateProfileMlMetric, ProfileMlMetricRecord};

#[cfg(test)]
pub use stub::ProfileMlMetricsServiceStub;

#[derive(Clone)]
enum ProfileMlMetricsServiceBackend {
    Repository(ProfileMlMetricsRepository),
    #[cfg(test)]
    Stub(Arc<ProfileMlMetricsServiceStub>),
}

#[derive(Clone)]
pub struct ProfileMlMetricsService {
    backend: ProfileMlMetricsServiceBackend,
}

impl ProfileMlMetricsService {
    pub fn new(repository: ProfileMlMetricsRepository) -> Self {
        Self {
            backend: ProfileMlMetricsServiceBackend::Repository(repository),
        }
    }

    pub async fn create(
        &self,
        input: CreateProfileMlMetric,
    ) -> Result<ProfileMlMetricRecord, RepositoryError> {
        match &self.backend {
            ProfileMlMetricsServiceBackend::Repository(repository) => repository.create(&input).await,
            #[cfg(test)]
            ProfileMlMetricsServiceBackend::Stub(stub) => stub.create(input),
        }
    }

    pub async fn list_recent(
        &self,
        profile_id: &str,
        limit: usize,
    ) -> Result<Vec<ProfileMlMetricRecord>, RepositoryError> {
        match &self.backend {
            ProfileMlMetricsServiceBackend::Repository(repository) => {
                repository.list_recent(profile_id, limit).await
            }
            #[cfg(test)]
            ProfileMlMetricsServiceBackend::Stub(stub) => stub.list_recent(profile_id, limit),
        }
    }

    #[cfg(test)]
    pub fn for_tests(stub: ProfileMlMetricsServiceStub) -> Self {
        Self {
            backend: ProfileMlMetricsServiceBackend::Stub(Arc::new(stub)),
        }
    }
}
