ALTER TABLE profiles
    ADD COLUMN IF NOT EXISTS ml_last_retrained_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS ml_examples_since_retrain INTEGER NOT NULL DEFAULT 0,
    ADD COLUMN IF NOT EXISTS ml_last_artifact_version TEXT,
    ADD COLUMN IF NOT EXISTS ml_last_training_status TEXT;

CREATE TABLE IF NOT EXISTS profile_ml_labeled_jobs (
    profile_id TEXT NOT NULL REFERENCES profiles(id) ON DELETE CASCADE,
    job_id TEXT NOT NULL,
    first_labeled_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (profile_id, job_id)
);

CREATE TABLE IF NOT EXISTS profile_ml_metrics (
    id TEXT PRIMARY KEY,
    profile_id TEXT NOT NULL REFERENCES profiles(id) ON DELETE CASCADE,
    retrained_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    status TEXT NOT NULL,
    artifact_version TEXT,
    model_type TEXT,
    reason TEXT,
    metrics_json JSONB,
    training_json JSONB,
    feature_importances_json JSONB,
    benchmark_json JSONB
);

CREATE INDEX IF NOT EXISTS profile_ml_metrics_profile_id_retrained_at_idx
    ON profile_ml_metrics (profile_id, retrained_at DESC);
