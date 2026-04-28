ALTER TABLE profiles
ADD COLUMN IF NOT EXISTS work_mode_preference TEXT NOT NULL DEFAULT 'any'
CHECK (work_mode_preference IN ('remote_only', 'hybrid', 'onsite', 'any'));

UPDATE profiles
SET work_mode_preference = CASE
    WHEN LOWER(TRIM(preferred_work_mode)) IN ('remote', 'remote_only', 'full_remote', 'fully_remote', 'fully remote', 'full remote')
        THEN 'remote_only'
    WHEN LOWER(TRIM(preferred_work_mode)) = 'hybrid'
        THEN 'hybrid'
    WHEN LOWER(TRIM(preferred_work_mode)) IN ('onsite', 'on-site', 'office', 'in-office')
        THEN 'onsite'
    WHEN LOWER(TRIM(preferred_work_mode)) = 'any'
        THEN 'any'
    ELSE work_mode_preference
END
WHERE preferred_work_mode IS NOT NULL;
