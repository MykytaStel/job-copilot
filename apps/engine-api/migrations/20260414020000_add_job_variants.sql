CREATE TABLE IF NOT EXISTS job_variants (
    id TEXT PRIMARY KEY,
    job_id TEXT NOT NULL REFERENCES jobs (id) ON DELETE CASCADE,
    source TEXT NOT NULL,
    source_job_id TEXT NOT NULL,
    source_url TEXT NOT NULL,
    raw_hash TEXT NOT NULL,
    raw_payload JSONB NOT NULL,
    fetched_at TIMESTAMPTZ NOT NULL
);

CREATE UNIQUE INDEX IF NOT EXISTS job_variants_source_source_job_id_idx
    ON job_variants (source, source_job_id);

CREATE INDEX IF NOT EXISTS job_variants_job_id_idx ON job_variants (job_id);
CREATE INDEX IF NOT EXISTS job_variants_fetched_at_idx ON job_variants (fetched_at DESC);
