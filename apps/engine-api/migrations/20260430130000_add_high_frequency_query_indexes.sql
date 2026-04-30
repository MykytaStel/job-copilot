-- High-frequency query index audit.
--
-- Existing coverage verified:
--   jobs.is_active -> jobs_is_active_idx
--   jobs recency -> jobs_last_seen_at_idx, jobs_first_seen_at_idx,
--                   jobs_active_last_seen_posted_idx, jobs_active_first_seen_idx
--   user_events(profile_id, created_at) -> user_events_profile_created_idx
--
-- Schema note:
--   jobs.source and jobs.created_at are not columns in the canonical jobs table.
--   Source is stored on job_variants.source, and job recency uses first_seen_at/last_seen_at.

CREATE INDEX IF NOT EXISTS profile_job_feedback_profile_created_idx
    ON profile_job_feedback (profile_id, created_at DESC, job_id ASC);

CREATE INDEX IF NOT EXISTS job_variants_source_job_idx
    ON job_variants (source, job_id);
