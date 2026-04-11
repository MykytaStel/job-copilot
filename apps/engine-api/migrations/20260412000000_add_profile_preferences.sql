-- Add salary and work mode preference columns to profiles.
-- These are optional and default to NULL (neutral: 0.5 in fit score).
ALTER TABLE profiles ADD COLUMN IF NOT EXISTS salary_min_usd INTEGER;
ALTER TABLE profiles ADD COLUMN IF NOT EXISTS salary_max_usd INTEGER;
ALTER TABLE profiles ADD COLUMN IF NOT EXISTS preferred_work_mode TEXT;

-- Persist fast local fit scores for cross-job analytics.
-- Unique per (job, resume) pair; ON CONFLICT updates in place.
CREATE TABLE IF NOT EXISTS fit_scores (
    id TEXT PRIMARY KEY,
    job_id TEXT NOT NULL REFERENCES jobs(id) ON DELETE CASCADE,
    resume_id TEXT NOT NULL REFERENCES resumes(id) ON DELETE CASCADE,
    total INTEGER NOT NULL,
    skill_overlap REAL NOT NULL,
    seniority_alignment REAL NOT NULL,
    salary_overlap REAL NOT NULL,
    work_mode_match REAL NOT NULL,
    matched_skills JSONB NOT NULL DEFAULT '[]',
    missing_skills JSONB NOT NULL DEFAULT '[]',
    computed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (job_id, resume_id)
);

CREATE INDEX IF NOT EXISTS fit_scores_resume_id_idx ON fit_scores (resume_id);
