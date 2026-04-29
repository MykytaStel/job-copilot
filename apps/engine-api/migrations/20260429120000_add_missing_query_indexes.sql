-- Missing query indexes found by auditing engine-api migrations and sqlx query patterns.
--
-- Added indexes:
--   jobs_active_last_seen_posted_idx
--     Supports active job feeds and search fallback ordering by last_seen_at/posted_at.
--
--   jobs_active_first_seen_idx
--     Supports market live fallback windows that count active jobs by first_seen_at.
--
--   jobs_active_company_last_seen_idx
--     Supports market company aggregation over active jobs with usable company names.
--
--   job_variants_job_source_idx
--     Supports source-filtered job feed EXISTS checks by job_id + source.
--
--   job_variants_job_latest_idx
--     Supports lateral latest-variant lookups by job_id ordered by fetched_at/last_seen_at/source.
--
--   profile_job_feedback_profile_updated_idx
--     Supports profile feedback export/analytics lists ordered by updated_at.
--
--   profile_company_feedback_profile_updated_idx
--     Supports profile company feedback lists ordered by updated_at.
--
--   notifications_unread_profile_created_idx
--     Supports unread notification counts with a smaller partial index.
--
--   market_snapshots_type_created_idx
--     Supports fresh snapshot reads by snapshot_type ordered by created_at.

CREATE INDEX IF NOT EXISTS jobs_active_last_seen_posted_idx
    ON jobs (last_seen_at DESC, posted_at DESC)
    WHERE is_active = TRUE;

CREATE INDEX IF NOT EXISTS jobs_active_first_seen_idx
    ON jobs (first_seen_at DESC)
    WHERE is_active = TRUE;

CREATE INDEX IF NOT EXISTS jobs_active_company_last_seen_idx
    ON jobs (company_name, last_seen_at DESC)
    WHERE is_active = TRUE;

CREATE INDEX IF NOT EXISTS job_variants_job_source_idx
    ON job_variants (job_id, source);

CREATE INDEX IF NOT EXISTS job_variants_job_latest_idx
    ON job_variants (job_id, fetched_at DESC, last_seen_at DESC, source ASC);

CREATE INDEX IF NOT EXISTS profile_job_feedback_profile_updated_idx
    ON profile_job_feedback (profile_id, updated_at DESC, job_id ASC);

CREATE INDEX IF NOT EXISTS profile_company_feedback_profile_updated_idx
    ON profile_company_feedback (profile_id, updated_at DESC, normalized_company_name ASC);

CREATE INDEX IF NOT EXISTS notifications_unread_profile_created_idx
    ON notifications (profile_id, created_at DESC)
    WHERE read_at IS NULL;

CREATE INDEX IF NOT EXISTS market_snapshots_type_created_idx
    ON market_snapshots (snapshot_type, created_at DESC);
