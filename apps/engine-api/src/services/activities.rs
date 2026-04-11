#[cfg(test)]
use std::collections::HashMap;
#[cfg(test)]
use std::sync::{Arc, Mutex};

use crate::db::repositories::{ActivitiesRepository, RepositoryError};
use crate::domain::application::model::{Activity, CreateActivity};

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
            ActivitiesServiceBackend::Repository(repository) => {
                repository.create(&activity).await
            }
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

#[cfg(test)]
#[derive(Default)]
pub struct ActivitiesServiceStub {
    activities: Mutex<HashMap<String, Activity>>,
}

#[cfg(test)]
impl ActivitiesServiceStub {
    fn create(&self, activity: CreateActivity) -> Result<Activity, RepositoryError> {
        let created = Activity {
            id: format!("activity-{}", uuid::Uuid::now_v7()),
            application_id: activity.application_id,
            activity_type: activity.activity_type,
            description: activity.description,
            happened_at: activity.happened_at,
            created_at: "2026-04-11T00:00:00+00:00".to_string(),
        };

        self.activities
            .lock()
            .expect("activities stub mutex should not be poisoned")
            .insert(created.id.clone(), created.clone());

        Ok(created)
    }
}
