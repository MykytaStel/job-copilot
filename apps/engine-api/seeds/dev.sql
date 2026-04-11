INSERT INTO jobs (
    id,
    title,
    company_name,
    remote_type,
    seniority,
    description_text,
    salary_min,
    salary_max,
    salary_currency,
    posted_at,
    last_seen_at,
    is_active
)
VALUES
    (
        'job_backend_rust_001',
        'Backend Rust Engineer',
        'NovaLedger',
        'hybrid',
        'mid',
        'Build internal APIs for a multi-tenant hiring platform, improve Postgres query performance, and ship reliable integrations with ingestion services.',
        5500,
        7200,
        'USD',
        '2026-04-02T09:00:00Z',
        '2026-04-10T08:30:00Z',
        TRUE
    ),
    (
        'job_product_analytics_001',
        'Product Data Analyst',
        'LiftOrbit',
        'remote',
        'mid',
        'Own product funnel analysis, define hiring marketplace KPIs, and partner with product and growth teams on experiment design.',
        3200,
        4300,
        'USD',
        '2026-04-04T10:15:00Z',
        '2026-04-09T14:20:00Z',
        TRUE
    ),
    (
        'job_frontend_react_001',
        'Senior Frontend Engineer',
        'TalentFlow',
        'remote',
        'senior',
        'Lead the candidate dashboard experience, improve search result UX, and collaborate closely with design on application tracking workflows.',
        4500,
        6000,
        'USD',
        '2026-04-01T07:45:00Z',
        '2026-04-08T16:00:00Z',
        TRUE
    )
ON CONFLICT (id) DO NOTHING;

INSERT INTO resumes (
    id,
    version,
    filename,
    raw_text,
    is_active,
    uploaded_at
)
VALUES (
    'resume_backend_001',
    1,
    'resume_backend.txt',
    'Senior backend engineer with Rust, Postgres, distributed systems, API design, and cloud infrastructure experience.',
    TRUE,
    '2026-04-09T09:00:00Z'
)
ON CONFLICT (id) DO NOTHING;

INSERT INTO applications (
    id,
    job_id,
    resume_id,
    status,
    applied_at,
    due_date,
    updated_at
)
VALUES
    (
        'application_backend_rust_001',
        'job_backend_rust_001',
        NULL,
        'interview',
        '2026-04-07T11:00:00Z',
        '2026-04-15T12:00:00Z',
        '2026-04-10T09:45:00Z'
    ),
    (
        'application_product_analytics_001',
        'job_product_analytics_001',
        NULL,
        'applied',
        '2026-04-08T13:30:00Z',
        '2026-04-18T17:00:00Z',
        '2026-04-09T18:10:00Z'
    )
ON CONFLICT (id) DO NOTHING;
