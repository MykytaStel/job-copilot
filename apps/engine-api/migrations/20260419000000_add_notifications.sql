CREATE TABLE IF NOT EXISTS notifications (
    id TEXT PRIMARY KEY,
    profile_id TEXT NOT NULL REFERENCES profiles(id) ON DELETE CASCADE,
    type TEXT NOT NULL,
    title TEXT NOT NULL,
    body TEXT,
    payload JSONB,
    read_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS notifications_profile_id_read_at_idx
    ON notifications(profile_id, read_at);

CREATE INDEX IF NOT EXISTS notifications_profile_id_created_at_idx
    ON notifications(profile_id, created_at DESC);
