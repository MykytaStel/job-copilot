ALTER TABLE profile_job_feedback
    ADD COLUMN IF NOT EXISTS salary_signal TEXT
        CHECK (salary_signal IN (
            'above_expectation', 'at_expectation', 'below_expectation', 'not_shown'
        ));
