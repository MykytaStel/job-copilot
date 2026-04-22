// Shared interim market role-demand heuristic for `engine-api` and
// `ingestion`.
//
// This is intentionally a market analytics fallback, not canonical domain
// truth. It groups jobs by title patterns only until the market pipeline reads
// from a proper `RoleId`-aware aggregate.

pub const MARKET_ROLE_GROUPS_VALUES_SQL: &str = r#"
('Frontend'),
('Backend'),
('Fullstack'),
('DevOps'),
('Data/ML'),
('QA'),
('Design'),
('Management')
"#;

pub const MARKET_ROLE_GROUP_ORDER_ARRAY_SQL: &str =
    "ARRAY['Frontend', 'Backend', 'Fullstack', 'DevOps', 'Data/ML', 'QA', 'Design', 'Management']::text[]";

pub const MARKET_ROLE_GROUP_CLASSIFIER_CASE_SQL: &str = r#"
CASE
    WHEN title ILIKE '%engineering manager%'
      OR title ILIKE '%product manager%'
      OR title ILIKE '%project manager%'
      OR title ILIKE '%program manager%'
      OR title ILIKE '%delivery manager%'
      OR title ILIKE '%product owner%'
      OR title ILIKE '%tech lead%'
      OR title ILIKE '%technical lead%'
      OR title ILIKE '%head of engineering%'
      OR title ILIKE '%vp of engineering%'
    THEN 'Management'
    WHEN title ILIKE '%product designer%'
      OR title ILIKE '%ui/ux designer%'
      OR title ILIKE '%ux designer%'
      OR title ILIKE '%ui designer%'
      OR title ILIKE '%interaction designer%'
      OR title ILIKE '%graphic designer%'
    THEN 'Design'
    WHEN title ~* '(^|[^a-z])(qa|sdet|tester)([^a-z]|$)'
      OR title ILIKE '%quality assurance%'
      OR title ILIKE '%test engineer%'
      OR title ILIKE '%automation qa%'
    THEN 'QA'
    WHEN title ILIKE '%devops%'
      OR title ILIKE '%site reliability%'
      OR title ~* '(^|[^a-z])sre([^a-z]|$)'
      OR title ILIKE '%cloud engineer%'
      OR title ILIKE '%cloud architect%'
      OR title ILIKE '%infrastructure%'
      OR title ILIKE '%platform engineer%'
    THEN 'DevOps'
    WHEN title ILIKE '%machine learning%'
      OR title ILIKE '%ml engineer%'
      OR title ILIKE '%ai engineer%'
      OR title ILIKE '%data scientist%'
      OR title ILIKE '%data engineer%'
      OR title ILIKE '%data analyst%'
      OR title ILIKE '%analytics engineer%'
      OR title ILIKE '%analyst%'
    THEN 'Data/ML'
    WHEN title ILIKE '%fullstack%'
      OR title ILIKE '%full-stack%'
      OR title ILIKE '%full stack%'
    THEN 'Fullstack'
    WHEN title ILIKE '%frontend%'
      OR title ILIKE '%front-end%'
      OR title ILIKE '%front end%'
      OR title ILIKE '%react%'
      OR title ILIKE '%vue%'
      OR title ILIKE '%angular%'
      OR title ILIKE '%next.js%'
      OR title ILIKE '%nextjs%'
      OR title ILIKE '%typescript%'
      OR title ILIKE '%javascript%'
    THEN 'Frontend'
    WHEN title ILIKE '%backend%'
      OR title ILIKE '%back-end%'
      OR title ILIKE '%back end%'
      OR title ILIKE '%server-side%'
      OR title ILIKE '%api developer%'
      OR title ILIKE '%rust engineer%'
      OR title ILIKE '%rust developer%'
      OR title ILIKE '%golang%'
      OR title ILIKE '%go developer%'
      OR title ILIKE '%java developer%'
      OR title ILIKE '%python developer%'
      OR title ILIKE '%php developer%'
      OR title ILIKE '%node.js%'
      OR title ILIKE '%nodejs%'
    THEN 'Backend'
    ELSE NULL
END
"#;
