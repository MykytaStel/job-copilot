ALTER TABLE profile_job_feedback
    ADD COLUMN IF NOT EXISTS legitimacy_signal TEXT
        CHECK (legitimacy_signal IN (
            'looks_real', 'suspicious', 'spam', 'duplicate'
        ));
