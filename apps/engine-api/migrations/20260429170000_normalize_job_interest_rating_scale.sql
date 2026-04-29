ALTER TABLE profile_job_feedback
    DROP CONSTRAINT IF EXISTS profile_job_feedback_interest_rating_check;

UPDATE profile_job_feedback
SET interest_rating = NULL
WHERE interest_rating IS NOT NULL
  AND interest_rating NOT BETWEEN 1 AND 5;

ALTER TABLE profile_job_feedback
    ADD CONSTRAINT profile_job_feedback_interest_rating_check
    CHECK (interest_rating BETWEEN 1 AND 5);
