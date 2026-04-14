CREATE TABLE IF NOT EXISTS user_events (
    id TEXT PRIMARY KEY,
    profile_id TEXT NOT NULL REFERENCES profiles (id) ON DELETE CASCADE,
    event_type TEXT NOT NULL CHECK (
        event_type IN (
            'job_impression',
            'job_opened',
            'job_saved',
            'job_unsaved',
            'job_hidden',
            'job_unhidden',
            'job_bad_fit',
            'job_bad_fit_removed',
            'company_whitelisted',
            'company_blacklisted',
            'search_run',
            'fit_explanation_requested',
            'application_coach_requested',
            'cover_letter_draft_requested',
            'interview_prep_requested',
            'application_created'
        )
    ),
    job_id TEXT,
    company_name TEXT,
    source TEXT,
    role_family TEXT,
    payload_json JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS user_events_profile_created_idx
    ON user_events (profile_id, created_at DESC);
CREATE INDEX IF NOT EXISTS user_events_profile_type_created_idx
    ON user_events (profile_id, event_type, created_at DESC);
CREATE INDEX IF NOT EXISTS user_events_profile_job_created_idx
    ON user_events (profile_id, job_id, created_at DESC);
