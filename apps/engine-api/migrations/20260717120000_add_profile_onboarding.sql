CREATE TABLE IF NOT EXISTS profile_onboarding (
    profile_id TEXT PRIMARY KEY REFERENCES profiles(id) ON DELETE CASCADE,
    current_step TEXT NOT NULL DEFAULT 'cv'
        CHECK (current_step IN ('cv', 'profile', 'preferences', 'recommendations', 'complete')),
    completed_steps TEXT[] NOT NULL DEFAULT ARRAY[]::TEXT[],
    skipped_steps TEXT[] NOT NULL DEFAULT ARRAY[]::TEXT[],
    completed_at TIMESTAMPTZ,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

INSERT INTO profile_onboarding (
    profile_id,
    current_step,
    completed_steps,
    skipped_steps,
    completed_at,
    updated_at
)
SELECT
    id,
    'complete',
    ARRAY['cv', 'profile', 'preferences', 'recommendations']::TEXT[],
    ARRAY[]::TEXT[],
    NOW(),
    NOW()
FROM profiles
ON CONFLICT (profile_id) DO NOTHING;
