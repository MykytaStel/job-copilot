// Test double for ProfileMlMetricsService. The real backend uses ProfileMlMetricsRepository.
use std::sync::Mutex;

use crate::db::repositories::RepositoryError;
use crate::domain::profile::ml::{CreateProfileMlMetric, ProfileMlMetricRecord};

#[derive(Default)]
pub struct ProfileMlMetricsServiceStub {
    records: Mutex<Vec<ProfileMlMetricRecord>>,
}

impl ProfileMlMetricsServiceStub {
    pub fn with_record(self, record: ProfileMlMetricRecord) -> Self {
        self.records
            .lock()
            .expect("profile ml metrics stub mutex should not be poisoned")
            .push(record);
        self
    }

    pub(crate) fn create(
        &self,
        input: CreateProfileMlMetric,
    ) -> Result<ProfileMlMetricRecord, RepositoryError> {
        let mut records = self
            .records
            .lock()
            .expect("profile ml metrics stub mutex should not be poisoned");
        let record = ProfileMlMetricRecord {
            id: format!("ml-metric-{}", records.len() + 1),
            profile_id: input.profile_id,
            retrained_at: format!("2026-04-22T00:00:{:02}Z", records.len()),
            status: input.status,
            artifact_version: input.artifact_version,
            model_type: input.model_type,
            reason: input.reason,
            metrics_json: input.metrics_json,
            training_json: input.training_json,
            feature_importances_json: input.feature_importances_json,
            benchmark_json: input.benchmark_json,
        };
        records.push(record.clone());
        Ok(record)
    }

    pub(crate) fn list_recent(
        &self,
        profile_id: &str,
        limit: usize,
    ) -> Result<Vec<ProfileMlMetricRecord>, RepositoryError> {
        let mut rows = self
            .records
            .lock()
            .expect("profile ml metrics stub mutex should not be poisoned")
            .iter()
            .filter(|record| record.profile_id == profile_id)
            .cloned()
            .collect::<Vec<_>>();
        rows.reverse();
        rows.truncate(limit);
        Ok(rows)
    }
}
