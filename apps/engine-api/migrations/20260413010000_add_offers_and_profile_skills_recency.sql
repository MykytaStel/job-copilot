ALTER TABLE profiles
ADD COLUMN IF NOT EXISTS skills_updated_at TIMESTAMPTZ;

CREATE INDEX IF NOT EXISTS profiles_skills_updated_at_idx
ON profiles (skills_updated_at DESC NULLS LAST);

CREATE TABLE IF NOT EXISTS offers (
    id TEXT PRIMARY KEY,
    application_id TEXT NOT NULL UNIQUE REFERENCES applications (id) ON DELETE CASCADE,
    status TEXT NOT NULL DEFAULT 'draft',
    compensation_min INTEGER,
    compensation_max INTEGER,
    compensation_currency TEXT,
    starts_at TIMESTAMPTZ,
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS offers_status_idx ON offers (status);
CREATE INDEX IF NOT EXISTS offers_updated_at_idx ON offers (updated_at DESC);
