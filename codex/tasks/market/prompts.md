# Block H — Market Intelligence Expansion (8 tasks)

---

## H1 — Market page: read from snapshots (canonical task)

```
(See A13 in stability/prompts.md — this is the same task.
A13 is the authoritative prompt. Do not implement twice.)
```

---

## H2 — Company hiring velocity chart

```
You are working in the Job Copilot monorepo.
First read: CLAUDE.md, apps/engine-api/src/api/routes/market.rs,
apps/engine-api/migrations/ (market_snapshots or jobs table),
apps/web/src/pages/ (Market page)

## Goal
Add a "Company Hiring Velocity" section to the Market page showing:
- Top 10 companies by new job postings in the last 30 days
- Chart: bar chart (horizontal) with company name + job count
- Trend indicator: ↑ growing (more jobs this week vs last week) ↓ declining

Backend: add GET /api/v1/market/company-velocity that returns
[{ company: String, job_count: u32, trend: "growing"|"stable"|"declining" }]
Compute from jobs table: count new jobs per company in last 7 vs prior 7 days.

## Inspect first
- apps/engine-api/src/api/routes/market.rs — existing company-related route
- apps/engine-api/src/db/ — job count queries
- apps/web/src/pages/Market.tsx or MarketPage — existing chart patterns

## Likely files to modify
- apps/engine-api/src/api/routes/market.rs (add company-velocity endpoint)
- apps/web/src/pages/ (add velocity section to Market page)
- apps/web/src/api/market.ts (add velocity client)

## Rules
- Minimum 3 recent jobs per company to be shown (filter noise).
- No new DB tables — query from jobs with first_seen_at.
- Chart is static SVG or CSS bar chart — no charting library unless one already exists.

## Acceptance criteria
- [ ] GET /api/v1/market/company-velocity returns correct data
- [ ] Web shows horizontal bar chart with trend indicators
- [ ] Only companies with >= 3 recent jobs shown
- [ ] Cargo + web typecheck pass

## Verification commands
cargo test --manifest-path apps/engine-api/Cargo.toml
pnpm --dir apps/web typecheck

## Final response format
## What changed
## Why
## Files changed
## Verification
## Notes / follow-ups
```

---

## H3 — Market freeze signals

```
You are working in the Job Copilot monorepo.
First read: CLAUDE.md, apps/engine-api/src/api/routes/market.rs,
apps/engine-api/migrations/ (jobs table — inactivated_at column),
apps/web/src/pages/ (Market page)

## Goal
Show "Market Freeze Signals" — companies that recently stopped posting:
- Companies with >= 5 jobs posted in the past 60 days
- But 0 new jobs in the last 14 days
- Sorted by "silence duration" (longest silence first)

Backend: GET /api/v1/market/freeze-signals returning
[{ company: String, last_posted_at: DateTime, days_since_last_post: u32, historical_count: u32 }]

Web: add "Hiring Paused" section in Market page with a warning-colored card list.

## Inspect first
- apps/engine-api/src/api/routes/market.rs
- apps/engine-api/src/db/ — job date queries
- apps/web/src/pages/ — Market page

## Likely files to modify
- apps/engine-api/src/api/routes/market.rs (add freeze-signals endpoint)
- apps/web/src/pages/ (add section to Market page)
- apps/web/src/api/market.ts

## Rules
- Only companies with significant historical posting (>= 5 jobs) are flagged.
- Do not show companies with < 5 total historical jobs — too noisy.
- "days_since_last_post" computed as current_date - max(first_seen_at) for the company.

## Acceptance criteria
- [ ] Freeze signals endpoint returns correct companies
- [ ] Web shows "Hiring Paused" section
- [ ] Only companies with >= 5 historical jobs shown
- [ ] Sorted by longest silence first
- [ ] Cargo + web typecheck pass

## Verification commands
cargo test --manifest-path apps/engine-api/Cargo.toml
pnpm --dir apps/web typecheck

## Final response format
## What changed
## Why
## Files changed
## Verification
## Notes / follow-ups
```

---

## H4 — Tech stack demand chart

```
You are working in the Job Copilot monorepo.
First read: CLAUDE.md, apps/engine-api/src/api/routes/market.rs,
apps/engine-api/migrations/ (jobs table — skills/description columns),
apps/web/src/pages/ (Market page)

## Goal
Show "Tech Skills in Demand" chart on the Market page — count how many active jobs
mention each technology in the last 30 days. Show top 20 technologies as a horizontal
bar chart.

Backend: GET /api/v1/market/tech-demand returning
[{ skill: String, job_count: u32, percentage: f32 }] (percentage of total active jobs)

Skills to count: React, Vue, Angular, TypeScript, JavaScript, Node.js, Python, Rust,
Go, Java, Kotlin, PostgreSQL, Redis, Docker, Kubernetes, AWS, GCP, Next.js, GraphQL,
FastAPI, Django, Spring Boot. Search job description + title.

## Inspect first
- apps/engine-api/src/api/routes/market.rs
- apps/engine-api/src/db/ — jobs table, description/skills columns
- apps/web/src/pages/ — Market page, existing chart patterns

## Likely files to modify
- apps/engine-api/src/api/routes/market.rs (add tech-demand endpoint)
- apps/web/src/pages/ (add tech demand section)
- apps/web/src/api/market.ts

## Rules
- Keyword search in job description (case-insensitive SQL ILIKE or regex).
- Percentage is relative to total active jobs in period.
- Sort by count descending.

## Acceptance criteria
- [ ] tech-demand endpoint returns non-empty results
- [ ] Top 20 technologies shown
- [ ] Percentages sum to <= (some technologies co-occur in same job)
- [ ] Chart renders correctly
- [ ] Cargo + web typecheck pass

## Verification commands
cargo test --manifest-path apps/engine-api/Cargo.toml
pnpm --dir apps/web typecheck

## Final response format
## What changed
## Why
## Files changed
## Verification
## Notes / follow-ups
```

---

## H5 — Role demand by region

```
You are working in the Job Copilot monorepo.
First read: CLAUDE.md, apps/engine-api/src/api/routes/market.rs,
apps/engine-api/migrations/ (jobs table — location/remote columns),
apps/web/src/pages/ (Market page)

## Goal
Add a "Role Demand by Region" breakdown to the Market page:
- Remote: X jobs
- Kyiv: X jobs
- Lviv: X jobs
- Other Ukraine: X jobs
- Abroad/Relocation: X jobs

Backend: GET /api/v1/market/region-breakdown returning
[{ region: String, job_count: u32, top_roles: [String] }]
Derive regions from jobs.location field using pattern matching.

## Inspect first
- apps/engine-api/src/api/routes/market.rs
- apps/engine-api/migrations/ — jobs table location column
- apps/web/src/pages/ — Market page

## Likely files to modify
- apps/engine-api/src/api/routes/market.rs (add region-breakdown endpoint)
- apps/web/src/pages/ (add region section)
- apps/web/src/api/market.ts

## Rules
- Location matching case-insensitive.
- "Remote" = remote_type = remote.
- "Kyiv" = location ILIKE '%kyiv%' OR '%київ%'.
- "Abroad" = location contains country names outside Ukraine.

## Acceptance criteria
- [ ] Region breakdown endpoint returns correct counts
- [ ] Web shows region breakdown section
- [ ] Cargo + web typecheck pass

## Verification commands
cargo test --manifest-path apps/engine-api/Cargo.toml
pnpm --dir apps/web typecheck

## Final response format
## What changed
## Why
## Files changed
## Verification
## Notes / follow-ups
```

---

## H6 — Salary by seniority level

```
You are working in the Job Copilot monorepo.
First read: CLAUDE.md, apps/engine-api/src/api/routes/market.rs,
apps/engine-api/migrations/ (jobs table — salary + seniority columns),
apps/web/src/pages/ (Market page — salary section)

## Goal
Add or improve a "Salary by Seniority" section in Market page:
- Junior: median salary range (USD/month)
- Mid: median salary range
- Senior: median salary range
- Lead/Staff: median salary range

Backend: GET /api/v1/market/salary-by-seniority returning
[{ seniority: String, median_min: u32, median_max: u32, sample_size: u32 }]
Only include seniorities with >= 10 data points.

## Inspect first
- apps/engine-api/src/api/routes/market.rs — existing salary-related routes
- apps/engine-api/migrations/ — jobs table seniority_level, salary_min, salary_max
- apps/web/src/pages/ — Market salary section

## Likely files to modify
- apps/engine-api/src/api/routes/market.rs (add/update salary-by-seniority endpoint)
- apps/web/src/pages/ (add/improve seniority salary section)
- apps/web/src/api/market.ts

## Rules
- Only include jobs with both salary AND seniority inferred.
- Filter jobs > 60 days old from calculation (stale salary data).
- Normalize to USD for comparison (use fixed rates: 1 EUR=1.1 USD, 1 UAH=0.024 USD).

## Acceptance criteria
- [ ] salary-by-seniority returns correct medians
- [ ] Only seniorities with >= 10 data points shown
- [ ] Salary normalized to USD
- [ ] Web renders salary table/chart by seniority
- [ ] Cargo + web typecheck pass

## Verification commands
cargo test --manifest-path apps/engine-api/Cargo.toml
pnpm --dir apps/web typecheck

## Final response format
## What changed
## Why
## Files changed
## Verification
## Notes / follow-ups
```

---

## H7 — Market alert: new companies hiring for your role

```
You are working in the Job Copilot monorepo.
First read: CLAUDE.md, apps/engine-api/src/api/routes/market.rs,
apps/engine-api/src/api/routes/notifications.rs,
apps/engine-api/src/domain/ (profile — role preferences)

## Goal
When the ingestion daemon runs and finds jobs from companies that have not posted
in > 30 days but match the candidate's target roles, create a notification:
"[Company] is hiring again for [Role]"

Implementation: after ingestion upsert batch, query for companies with:
- new jobs in this batch (first_seen_at = now)
- no jobs in prior 30 days
- job role_id matches profile's target roles

Create notifications for matching profiles.

## Inspect first
- apps/ingestion/src/ — post-upsert hooks or where to add notification logic
- apps/engine-api/src/api/routes/notifications.rs — create notification endpoint
- apps/engine-api/src/domain/ — profile target roles

## Likely files to modify
- apps/engine-api/src/api/routes/ (add internal endpoint for creating market alerts)
  OR apps/ingestion/src/ (call notification creation after batch)
- apps/engine-api/src/db/ (notification creation query)

## Rules
- Notifications are per-profile (only relevant profiles get the alert).
- Create notification only once per company resume (not every new job post).
- Do not spam — max 3 market alerts per profile per day.

## Acceptance criteria
- [ ] Market alert created when company resumes posting after 30-day gap
- [ ] Only for profiles whose target roles match
- [ ] Max 3 per profile per day
- [ ] Notification visible in /notifications page
- [ ] Cargo tests pass

## Verification commands
cargo test --manifest-path apps/engine-api/Cargo.toml
cargo check --manifest-path apps/ingestion/Cargo.toml

## Final response format
## What changed
## Why
## Files changed
## Verification
## Notes / follow-ups
```

---

## H8 — Company profile page

```
You are working in the Job Copilot monorepo.
First read: CLAUDE.md, apps/web/src/pages/ (Market page — companies section),
apps/engine-api/src/api/routes/market.rs,
apps/web/src/App.tsx (route registration)

## Goal
Add a company detail page at /market/companies/:company_slug showing:
- Company name + hiring stats (total jobs, active jobs, avg salary)
- All active job listings from this company
- Hiring velocity trend (7-day chart)
- Whether on whitelist/blacklist

Accessible by clicking a company name anywhere in the app (job cards, market page).

Backend: GET /api/v1/market/companies/:company_slug returning company stats + job list.

## Inspect first
- apps/engine-api/src/api/routes/market.rs — existing company endpoints
- apps/web/src/App.tsx — route registration
- apps/web/src/pages/ — Market page and job card company links

## Likely files to modify
- apps/engine-api/src/api/routes/market.rs (add :company_slug endpoint)
- apps/web/src/App.tsx (add /market/companies/:slug route)
- New file: apps/web/src/pages/CompanyDetail.tsx
- apps/web/src/api/market.ts (add company detail client)
- Job card component (make company name a link)

## Rules
- company_slug = lowercased, hyphenated company name (derive from company field).
- Job list in company page uses existing job card component.
- Whitelist/blacklist status shown with quick toggle.

## Acceptance criteria
- [ ] /market/companies/:slug loads company detail
- [ ] Shows company stats + active jobs
- [ ] Whitelist/blacklist status visible
- [ ] Company names in job cards link to company page
- [ ] Cargo + web typecheck pass

## Verification commands
cargo test --manifest-path apps/engine-api/Cargo.toml
pnpm --dir apps/web typecheck
pnpm guard:web-api-imports

## Final response format
## What changed
## Why
## Files changed
## Verification
## Notes / follow-ups
```
