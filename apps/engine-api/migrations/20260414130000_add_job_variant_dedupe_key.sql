ALTER TABLE job_variants
    ADD COLUMN IF NOT EXISTS dedupe_key TEXT;

UPDATE job_variants
SET dedupe_key = CONCAT(
    'title=',
    REGEXP_REPLACE(LOWER(TRIM(COALESCE(jobs.title, ''))), '\s+', ' ', 'g'),
    '|company=',
    REGEXP_REPLACE(LOWER(TRIM(COALESCE(jobs.company_name, ''))), '\s+', ' ', 'g'),
    '|location=',
    REGEXP_REPLACE(LOWER(TRIM(COALESCE(jobs.location, ''))), '\s+', ' ', 'g'),
    '|remote_type=',
    REGEXP_REPLACE(LOWER(TRIM(COALESCE(jobs.remote_type, ''))), '\s+', ' ', 'g'),
    '|seniority=',
    REGEXP_REPLACE(LOWER(TRIM(COALESCE(jobs.seniority, ''))), '\s+', ' ', 'g'),
    '|posted_on=',
    COALESCE(TO_CHAR(jobs.posted_at AT TIME ZONE 'UTC', 'YYYY-MM-DD'), '')
)
FROM jobs
WHERE jobs.id = job_variants.job_id
  AND (job_variants.dedupe_key IS NULL OR job_variants.dedupe_key = '');

ALTER TABLE job_variants
    ALTER COLUMN dedupe_key SET NOT NULL;

CREATE INDEX IF NOT EXISTS job_variants_dedupe_key_idx
    ON job_variants (dedupe_key);
