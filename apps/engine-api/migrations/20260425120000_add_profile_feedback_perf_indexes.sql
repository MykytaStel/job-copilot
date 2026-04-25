-- Performance indexes for profile-scoped feedback and application queries.
--
-- Rationale:
--   applications(profile_id, updated_at DESC) — dashboard and board views sort
--     applications by recency per profile; without this index both columns
--     require a full scan + sort.
--
--   profile_job_feedback(profile_id, saved) WHERE saved = TRUE — saved-jobs list
--     filters on exactly this predicate; a partial index avoids scanning the
--     large unsaved majority.
--
--   profile_job_feedback(profile_id, hidden) WHERE hidden = TRUE — same pattern
--     for the hidden-jobs filter used by ranking to exclude dismissed jobs.

CREATE INDEX IF NOT EXISTS applications_profile_updated_idx
    ON applications (profile_id, updated_at DESC);

CREATE INDEX IF NOT EXISTS profile_job_feedback_saved_idx
    ON profile_job_feedback (profile_id, saved) WHERE saved = TRUE;

CREATE INDEX IF NOT EXISTS profile_job_feedback_hidden_idx
    ON profile_job_feedback (profile_id, hidden) WHERE hidden = TRUE;
