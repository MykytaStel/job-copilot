use serde::Serialize;
use serde_json::Value;

use crate::domain::profile::ml::{ProfileMlMetricRecord, ProfileMlState};

#[derive(Debug, Serialize)]
pub struct ProfileMlStateResponse {
    pub profile_id: String,
    pub last_retrained_at: Option<String>,
    pub examples_since_retrain: usize,
    pub last_artifact_version: Option<String>,
    pub last_training_status: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ProfileMlMetricRecordResponse {
    pub id: String,
    pub profile_id: String,
    pub retrained_at: String,
    pub status: String,
    pub artifact_version: Option<String>,
    pub model_type: Option<String>,
    pub reason: Option<String>,
    pub metrics: Option<Value>,
    pub training: Option<Value>,
    pub feature_importances: Option<Value>,
    pub benchmark: Option<Value>,
}

#[derive(Debug, Serialize)]
pub struct RerankerMetricsSummaryResponse {
    pub run_count: usize,
    pub trained_run_count: usize,
    pub skipped_run_count: usize,
    pub failed_run_count: usize,
    pub warning_run_count: usize,
    pub last_warning_reason: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct RerankerMetricsResponse {
    pub profile_id: String,
    pub state: ProfileMlStateResponse,
    pub summary: RerankerMetricsSummaryResponse,
    pub runs: Vec<ProfileMlMetricRecordResponse>,
}

impl From<ProfileMlState> for ProfileMlStateResponse {
    fn from(value: ProfileMlState) -> Self {
        Self {
            profile_id: value.profile_id,
            last_retrained_at: value.last_retrained_at,
            examples_since_retrain: value.examples_since_retrain,
            last_artifact_version: value.last_artifact_version,
            last_training_status: value.last_training_status,
        }
    }
}

impl From<ProfileMlMetricRecord> for ProfileMlMetricRecordResponse {
    fn from(value: ProfileMlMetricRecord) -> Self {
        Self {
            id: value.id,
            profile_id: value.profile_id,
            retrained_at: value.retrained_at,
            status: value.status,
            artifact_version: value.artifact_version,
            model_type: value.model_type,
            reason: value.reason,
            metrics: value.metrics_json,
            training: value.training_json,
            feature_importances: value.feature_importances_json,
            benchmark: value.benchmark_json,
        }
    }
}
