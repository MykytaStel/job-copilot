#[cfg(test)]
#[path = "activities/stub.rs"]
mod stub;

#[cfg(test)]
use std::sync::Arc;

use crate::db::repositories::{ActivitiesRepository, RepositoryError};
use crate::domain::application::model::{Activity, CreateActivity};

#[cfg(test)]
pub use stub::ActivitiesServiceStub;

#[derive(Clone)]
enum ActivitiesServiceBackend {
    Repository(ActivitiesRepository),
    #[cfg(test)]
    Stub(Arc<ActivitiesServiceStub>),
}

#[derive(Clone)]
pub struct ActivitiesService {
    backend: ActivitiesServiceBackend,
}

impl ActivitiesService {
    pub fn new(repository: ActivitiesRepository) -> Self {
        Self {
            backend: ActivitiesServiceBackend::Repository(repository),
        }
    }

    pub async fn create(&self, activity: CreateActivity) -> Result<Activity, RepositoryError> {
        match &self.backend {
            ActivitiesServiceBackend::Repository(repository) => repository.create(&activity).await,
            #[cfg(test)]
            ActivitiesServiceBackend::Stub(stub) => stub.create(activity),
        }
    }

    #[cfg(test)]
    pub fn for_tests(stub: ActivitiesServiceStub) -> Self {
        Self {
            backend: ActivitiesServiceBackend::Stub(Arc::new(stub)),
        }
    }
}
