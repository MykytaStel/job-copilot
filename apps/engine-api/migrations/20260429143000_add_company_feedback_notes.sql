ALTER TABLE profile_company_feedback
    ADD COLUMN IF NOT EXISTS notes TEXT NOT NULL DEFAULT '';

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1
        FROM pg_constraint
        WHERE conname = 'profile_company_feedback_notes_length_check'
    ) THEN
        ALTER TABLE profile_company_feedback
            ADD CONSTRAINT profile_company_feedback_notes_length_check
            CHECK (char_length(notes) <= 500);
    END IF;
END $$;
