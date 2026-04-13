ALTER TABLE jobs
    ADD COLUMN IF NOT EXISTS first_seen_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS inactivated_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS reactivated_at TIMESTAMPTZ;

UPDATE jobs
SET
    first_seen_at = COALESCE(first_seen_at, posted_at, last_seen_at),
    inactivated_at = CASE
        WHEN is_active THEN NULL
        ELSE COALESCE(inactivated_at, last_seen_at)
    END
WHERE first_seen_at IS NULL OR (NOT is_active AND inactivated_at IS NULL);

ALTER TABLE jobs
    ALTER COLUMN first_seen_at SET NOT NULL;

ALTER TABLE job_variants
    ADD COLUMN IF NOT EXISTS last_seen_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS is_active BOOLEAN NOT NULL DEFAULT TRUE,
    ADD COLUMN IF NOT EXISTS inactivated_at TIMESTAMPTZ;

UPDATE job_variants
SET
    last_seen_at = COALESCE(last_seen_at, fetched_at),
    inactivated_at = CASE
        WHEN is_active THEN NULL
        ELSE COALESCE(inactivated_at, fetched_at)
    END
WHERE last_seen_at IS NULL OR (NOT is_active AND inactivated_at IS NULL);

ALTER TABLE job_variants
    ALTER COLUMN last_seen_at SET NOT NULL;

CREATE INDEX IF NOT EXISTS jobs_first_seen_at_idx ON jobs (first_seen_at DESC);
CREATE INDEX IF NOT EXISTS jobs_inactivated_at_idx ON jobs (inactivated_at DESC);
CREATE INDEX IF NOT EXISTS jobs_reactivated_at_idx ON jobs (reactivated_at DESC);
CREATE INDEX IF NOT EXISTS job_variants_is_active_idx ON job_variants (is_active);
CREATE INDEX IF NOT EXISTS job_variants_last_seen_at_idx ON job_variants (last_seen_at DESC);
