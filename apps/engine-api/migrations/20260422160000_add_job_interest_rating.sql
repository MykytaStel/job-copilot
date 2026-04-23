ALTER TABLE profile_job_feedback
    ADD COLUMN IF NOT EXISTS interest_rating SMALLINT
        CHECK (interest_rating BETWEEN -2 AND 2);
