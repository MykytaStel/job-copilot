-- Performance indexes for access patterns that were missing.
--
-- Rationale:
--   applications(job_id) — the UNIQUE constraint on job_id was dropped in
--     20260424120000 to allow one application per (profile, job). Without the
--     old constraint-backed index, lookups by job_id now require a full table
--     scan when joining applications to jobs.
--
--   fit_scores(job_id) — ranking queries join fit_scores by job_id; the table
--     only has an index on resume_id, not on job_id.

CREATE INDEX IF NOT EXISTS applications_job_id_idx
    ON applications (job_id);

CREATE INDEX IF NOT EXISTS fit_scores_job_id_idx
    ON fit_scores (job_id);
