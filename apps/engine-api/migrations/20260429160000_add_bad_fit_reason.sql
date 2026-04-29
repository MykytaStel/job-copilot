ALTER TABLE profile_job_feedback
    ADD COLUMN IF NOT EXISTS bad_fit_reason TEXT;
