CREATE TABLE IF NOT EXISTS profile_job_feedback (
    profile_id TEXT NOT NULL REFERENCES profiles (id) ON DELETE CASCADE,
    job_id TEXT NOT NULL REFERENCES jobs (id) ON DELETE CASCADE,
    saved BOOLEAN NOT NULL DEFAULT FALSE,
    hidden BOOLEAN NOT NULL DEFAULT FALSE,
    bad_fit BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (profile_id, job_id)
);

CREATE TABLE IF NOT EXISTS profile_company_feedback (
    profile_id TEXT NOT NULL REFERENCES profiles (id) ON DELETE CASCADE,
    normalized_company_name TEXT NOT NULL,
    company_name TEXT NOT NULL,
    status TEXT NOT NULL CHECK (status IN ('whitelist', 'blacklist')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (profile_id, normalized_company_name)
);

CREATE INDEX IF NOT EXISTS profile_job_feedback_profile_hidden_idx
    ON profile_job_feedback (profile_id, hidden);
CREATE INDEX IF NOT EXISTS profile_job_feedback_profile_saved_idx
    ON profile_job_feedback (profile_id, saved);
CREATE INDEX IF NOT EXISTS profile_company_feedback_profile_status_idx
    ON profile_company_feedback (profile_id, status);
