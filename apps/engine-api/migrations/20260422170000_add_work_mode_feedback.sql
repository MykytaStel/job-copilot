ALTER TABLE profile_job_feedback
    ADD COLUMN IF NOT EXISTS work_mode_signal TEXT
        CHECK (work_mode_signal IN (
            'matches_preference', 'would_accept', 'deal_breaker'
        ));
