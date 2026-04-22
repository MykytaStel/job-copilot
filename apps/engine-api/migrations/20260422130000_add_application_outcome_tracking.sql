ALTER TABLE applications
    ADD COLUMN IF NOT EXISTS outcome TEXT
        CHECK (outcome IN (
            'phone_screen', 'technical_interview', 'final_interview',
            'offer_received', 'rejected', 'ghosted', 'withdrew'
        )),
    ADD COLUMN IF NOT EXISTS outcome_date TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS rejection_stage TEXT
        CHECK (rejection_stage IN (
            'applied', 'phone_screen', 'technical_interview', 'final_interview'
        ));

CREATE INDEX IF NOT EXISTS applications_outcome_idx ON applications (outcome);
