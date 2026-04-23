use serde_json::Value;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ProfileMlState {
    pub profile_id: String,
    pub last_retrained_at: Option<String>,
    pub examples_since_retrain: usize,
    pub last_artifact_version: Option<String>,
    pub last_training_status: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProfileMlRetrainCandidate {
    pub profile_id: String,
    pub examples_since_retrain: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CreateProfileMlMetric {
    pub profile_id: String,
    pub status: String,
    pub artifact_version: Option<String>,
    pub model_type: Option<String>,
    pub reason: Option<String>,
    pub metrics_json: Option<Value>,
    pub training_json: Option<Value>,
    pub feature_importances_json: Option<Value>,
    pub benchmark_json: Option<Value>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProfileMlMetricRecord {
    pub id: String,
    pub profile_id: String,
    pub retrained_at: String,
    pub status: String,
    pub artifact_version: Option<String>,
    pub model_type: Option<String>,
    pub reason: Option<String>,
    pub metrics_json: Option<Value>,
    pub training_json: Option<Value>,
    pub feature_importances_json: Option<Value>,
    pub benchmark_json: Option<Value>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct UpdateProfileMlState {
    pub last_retrained_at: Option<Option<String>>,
    pub examples_since_retrain: Option<usize>,
    pub last_artifact_version: Option<Option<String>>,
    pub last_training_status: Option<Option<String>>,
}
