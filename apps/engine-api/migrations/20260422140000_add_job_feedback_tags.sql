CREATE TABLE IF NOT EXISTS profile_job_feedback_tags (
    profile_id TEXT NOT NULL REFERENCES profiles (id) ON DELETE CASCADE,
    job_id     TEXT NOT NULL REFERENCES jobs (id) ON DELETE CASCADE,
    tag        TEXT NOT NULL CHECK (tag IN (
        'salary_too_low', 'not_remote', 'too_junior', 'too_senior',
        'bad_tech_stack', 'suspicious_posting', 'already_applied',
        'duplicate_posting', 'bad_company_rep', 'wrong_city',
        'wrong_industry', 'visa_sponsorship_required',
        'interesting_challenge', 'great_company', 'good_salary',
        'remote_ok', 'good_tech_stack', 'fast_growth_company', 'nice_title'
    )),
    is_negative BOOLEAN NOT NULL GENERATED ALWAYS AS (
        tag IN (
            'salary_too_low', 'not_remote', 'too_junior', 'too_senior',
            'bad_tech_stack', 'suspicious_posting', 'already_applied',
            'duplicate_posting', 'bad_company_rep', 'wrong_city',
            'wrong_industry', 'visa_sponsorship_required'
        )
    ) STORED,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (profile_id, job_id, tag)
);

CREATE INDEX IF NOT EXISTS pfft_profile_job_idx ON profile_job_feedback_tags (profile_id, job_id);
CREATE INDEX IF NOT EXISTS pfft_profile_tag_idx  ON profile_job_feedback_tags (profile_id, tag);
