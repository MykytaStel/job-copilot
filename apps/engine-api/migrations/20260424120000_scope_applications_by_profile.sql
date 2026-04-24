ALTER TABLE applications
    ADD COLUMN IF NOT EXISTS profile_id TEXT REFERENCES profiles (id) ON DELETE CASCADE;

ALTER TABLE applications
    DROP CONSTRAINT IF EXISTS applications_job_id_key;

DROP INDEX IF EXISTS applications_job_id_key;

CREATE INDEX IF NOT EXISTS applications_profile_id_idx
    ON applications (profile_id);

CREATE UNIQUE INDEX IF NOT EXISTS applications_profile_job_unique_idx
    ON applications ((COALESCE(profile_id, '__global__')), job_id);
