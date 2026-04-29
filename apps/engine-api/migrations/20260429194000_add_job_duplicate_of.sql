ALTER TABLE jobs
ADD COLUMN IF NOT EXISTS duplicate_of TEXT REFERENCES jobs (id) ON DELETE SET NULL;

CREATE INDEX IF NOT EXISTS jobs_duplicate_of_idx
    ON jobs (duplicate_of)
    WHERE duplicate_of IS NOT NULL;

CREATE INDEX IF NOT EXISTS jobs_feed_primary_idx
    ON jobs (last_seen_at DESC, posted_at DESC)
    WHERE duplicate_of IS NULL;
