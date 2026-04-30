CREATE TABLE IF NOT EXISTS profile_auth_credentials (
    profile_id TEXT PRIMARY KEY REFERENCES profiles (id) ON DELETE CASCADE,
    email TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS profile_auth_credentials_email_idx ON profile_auth_credentials (email);
