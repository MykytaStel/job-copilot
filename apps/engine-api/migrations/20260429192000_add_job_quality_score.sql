ALTER TABLE jobs
ADD COLUMN IF NOT EXISTS quality_score INTEGER
    CHECK (quality_score IS NULL OR (quality_score >= 0 AND quality_score <= 100));

CREATE INDEX IF NOT EXISTS jobs_quality_score_idx
    ON jobs (quality_score)
    WHERE quality_score IS NOT NULL;
