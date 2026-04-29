CREATE TABLE IF NOT EXISTS ingestion_runs (
    id BIGSERIAL PRIMARY KEY,
    source TEXT NOT NULL,
    run_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    jobs_fetched INTEGER NOT NULL DEFAULT 0 CHECK (jobs_fetched >= 0),
    jobs_upserted INTEGER NOT NULL DEFAULT 0 CHECK (jobs_upserted >= 0),
    errors INTEGER NOT NULL DEFAULT 0 CHECK (errors >= 0),
    duration_ms BIGINT NOT NULL DEFAULT 0 CHECK (duration_ms >= 0),
    status TEXT NOT NULL CHECK (status IN ('ok', 'partial', 'failed'))
);

CREATE INDEX IF NOT EXISTS ingestion_runs_source_run_at_idx
    ON ingestion_runs (source, run_at DESC, id DESC);

CREATE INDEX IF NOT EXISTS ingestion_runs_run_at_idx
    ON ingestion_runs (run_at DESC);
