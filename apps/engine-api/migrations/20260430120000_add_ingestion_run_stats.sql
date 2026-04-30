ALTER TABLE ingestion_runs
    ADD COLUMN IF NOT EXISTS jobs_attempted INTEGER NOT NULL DEFAULT 0 CHECK (jobs_attempted >= 0),
    ADD COLUMN IF NOT EXISTS jobs_failed INTEGER NOT NULL DEFAULT 0 CHECK (jobs_failed >= 0),
    ADD COLUMN IF NOT EXISTS errors_json JSONB NOT NULL DEFAULT '[]'::jsonb CHECK (jsonb_typeof(errors_json) = 'array');

UPDATE ingestion_runs
SET jobs_attempted = jobs_fetched
WHERE jobs_attempted = 0
  AND jobs_fetched > 0;
