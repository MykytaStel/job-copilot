use std::collections::HashMap;
use std::sync::Mutex;

use crate::db::repositories::RepositoryError;
use crate::domain::application::model::{Activity, CreateActivity};

#[derive(Default)]
pub struct ActivitiesServiceStub {
    activities: Mutex<HashMap<String, Activity>>,
}

impl ActivitiesServiceStub {
    pub(crate) fn create(&self, activity: CreateActivity) -> Result<Activity, RepositoryError> {
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
