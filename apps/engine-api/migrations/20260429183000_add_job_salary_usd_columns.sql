ALTER TABLE jobs ADD COLUMN IF NOT EXISTS salary_usd_min INTEGER;
ALTER TABLE jobs ADD COLUMN IF NOT EXISTS salary_usd_max INTEGER;

CREATE INDEX IF NOT EXISTS jobs_salary_usd_min_idx
    ON jobs (salary_usd_min)
    WHERE salary_usd_min IS NOT NULL;
