# Block I — Ingestion Quality (8 tasks)

---

## I1 — Better seniority inference

```
You are working in the Job Copilot monorepo.
First read: CLAUDE.md, apps/ingestion/src/scrapers/mod.rs (normalization utilities),
apps/ingestion/src/scrapers/ (individual scrapers)

## Goal
Improve seniority_level inference from job titles and descriptions.
Current heuristics may miss many patterns. Add/improve regex patterns for:

Junior: "junior", "jr.", "entry level", "intern", "trainee", "початківець", "молодший"
Mid: "middle", "mid-level", "regular", "intermediate", "2-4 years"
Senior: "senior", "sr.", "досвідчений", "lead" (when not lead role), "3+ years", "5+ years"
Lead: "lead", "tech lead", "principal", "staff", "head of", "team lead"

Also infer from "X+ years" patterns in description:
- 0-1 years → junior
- 2-3 years → mid
- 4-6 years → senior
- 7+ years → lead

## Inspect first
- apps/ingestion/src/scrapers/mod.rs — infer_seniority function (or equivalent)
- apps/ingestion/src/scrapers/ — how each scraper calls inference

## Likely files to modify
- apps/ingestion/src/scrapers/mod.rs (expand seniority inference)
- apps/ingestion/tests/ or test module (add unit tests for new patterns)

## Rules
- Title takes precedence over description.
- Ukrainian and English patterns both covered.
- Existing correct inferences must not regress.

## Acceptance criteria
- [ ] Ukrainian junior/senior/mid terms recognized
- [ ] "X+ years" pattern extracts seniority
- [ ] Unit tests for all new patterns
- [ ] Cargo tests pass

## Verification commands
cargo test --manifest-path apps/ingestion/Cargo.toml
cargo check --manifest-path apps/ingestion/Cargo.toml

## Final response format
## What changed
## Why
## Files changed
## Verification
## Notes / follow-ups
```

---

## I2 — Salary normalization to USD/month

```
You are working in the Job Copilot monorepo.
First read: CLAUDE.md, apps/ingestion/src/scrapers/mod.rs (salary parsing),
apps/ingestion/src/ (scraper normalization)

## Goal
After parsing salary ranges (which can be in UAH/USD/EUR, per month/year/hour),
normalize all salaries to USD per month and store in salary_usd_min, salary_usd_max
columns alongside the original currency values.

Use fixed exchange rates (configurable in constants): 1 EUR = 1.10 USD, 1 UAH = 0.024 USD.
Hourly → monthly: multiply by 160 (40h/week × 4 weeks).
Annual → monthly: divide by 12.

## Inspect first
- apps/ingestion/src/scrapers/mod.rs — parse_salary function
- apps/engine-api/migrations/ — jobs table salary columns
- apps/ingestion/migrations/ or apps/engine-api/migrations/ — check if usd columns exist

## Likely files to modify
- apps/ingestion/src/scrapers/mod.rs (add normalization step)
- apps/engine-api/migrations/ (add salary_usd_min, salary_usd_max columns if missing)
- apps/ingestion/src/ (store normalized values during upsert)

## Rules
- Keep original salary_min/max/currency untouched.
- USD normalization is approximate — that's acceptable.
- Exchange rates as constants, not hardcoded magic numbers.

## Acceptance criteria
- [ ] salary_usd_min/max columns exist and are populated
- [ ] EUR and UAH salaries correctly converted
- [ ] Hourly and annual rates converted to monthly
- [ ] Unit tests for conversion logic
- [ ] Cargo tests pass

## Verification commands
cargo test --manifest-path apps/ingestion/Cargo.toml
cargo check --manifest-path apps/ingestion/Cargo.toml

## Final response format
## What changed
## Why
## Files changed
## Verification
## Notes / follow-ups
```

---

## I3 — Skills extraction from job descriptions

```
You are working in the Job Copilot monorepo.
First read: CLAUDE.md, apps/ingestion/src/scrapers/mod.rs,
apps/engine-api/migrations/ (jobs table — skills columns)

## Goal
Extract a structured list of skills from the job description during ingestion.
Store as extracted_skills: Vec<String> in a JSONB column on the jobs table.

Extraction approach: keyword matching against a predefined skill dictionary
(same list used in H4: React, Vue, TypeScript, Node.js, Python, Rust, Go, Java,
Kotlin, PostgreSQL, MongoDB, Redis, Docker, Kubernetes, AWS, GCP, Azure, Git,
CI/CD, GraphQL, REST, FastAPI, Django, Spring Boot, Terraform, Linux, etc.)

Also extract from explicit skill lists in the description
(lines starting with "Required:", "Skills:", "Вимоги:", "Технології:").

## Inspect first
- apps/ingestion/src/scrapers/mod.rs — where description is processed
- apps/engine-api/migrations/ — jobs table
- apps/ingestion/src/ — upsert logic

## Likely files to modify
- apps/engine-api/migrations/ (add extracted_skills JSONB column)
- apps/ingestion/src/scrapers/mod.rs (add extract_skills function)
- apps/ingestion/src/ (populate during upsert)

## Rules
- Case-insensitive matching.
- No duplicates in extracted list.
- Skills dictionary as a constant array in mod.rs.

## Acceptance criteria
- [ ] extracted_skills column exists in jobs
- [ ] Skills extracted correctly from sample job descriptions
- [ ] Unit tests with fixture descriptions
- [ ] Cargo tests pass

## Verification commands
cargo test --manifest-path apps/ingestion/Cargo.toml
cargo check --manifest-path apps/ingestion/Cargo.toml

## Final response format
## What changed
## Why
## Files changed
## Verification
## Notes / follow-ups
```

---

## I4 — Job quality scoring filter

```
You are working in the Job Copilot monorepo.
First read: CLAUDE.md, apps/ingestion/src/scrapers/mod.rs,
apps/engine-api/migrations/ (jobs table — quality/flag columns)

## Goal
During ingestion, compute a quality_score (0-100) for each job and store it.
Quality factors:
- Has description >= 200 chars: +30
- Has salary info: +20
- Has extracted_skills >= 3: +20
- Has seniority inferred: +10
- Has remote_type inferred: +10
- Has valid company name (not empty/generic): +10

Add quality_score column to jobs. In the engine-api feed endpoint, optionally filter
out jobs with quality_score < 20.

## Inspect first
- apps/ingestion/src/scrapers/mod.rs — normalization step
- apps/engine-api/migrations/ — jobs table
- apps/engine-api/src/api/routes/jobs.rs — feed query (where to add filter)

## Likely files to modify
- apps/engine-api/migrations/ (add quality_score INT column)
- apps/ingestion/src/scrapers/mod.rs (compute quality_score)
- apps/engine-api/src/api/routes/jobs.rs or search.rs (add optional quality filter)

## Rules
- quality_score column nullable (existing rows: null = unscored).
- Feed filter: only apply if quality_min param provided in request (not by default).
- Unit test quality scoring function.

## Acceptance criteria
- [ ] quality_score column populated during ingestion
- [ ] Scoring formula matches spec
- [ ] Feed accepts optional ?quality_min=X param
- [ ] Unit tests for quality scoring
- [ ] Cargo tests pass

## Verification commands
cargo test --manifest-path apps/ingestion/Cargo.toml
cargo check --manifest-path apps/ingestion/Cargo.toml
cargo test --manifest-path apps/engine-api/Cargo.toml

## Final response format
## What changed
## Why
## Files changed
## Verification
## Notes / follow-ups
```

---

## I5 — Company info enrichment from existing data

```
You are working in the Job Copilot monorepo.
First read: CLAUDE.md, apps/ingestion/src/scrapers/,
apps/engine-api/migrations/ (jobs table — company columns)

## Goal
Improve company data stored with each job. Current data: just company name string.
Enrich with:
- company_size: inferred from description patterns ("50-200 employees", "startup", "enterprise")
- company_industry: inferred from description ("fintech", "edtech", "e-commerce", "outsourcing")
- company_url: scrape from job detail page if linked

Store in new JSONB column company_meta: { size_hint: String, industry_hint: String, url: Option<String> }

## Inspect first
- apps/ingestion/src/scrapers/ — detail page enrichment for Djinni/Work.ua/Robota.ua
- apps/engine-api/migrations/ — jobs table
- apps/ingestion/src/scrapers/mod.rs — normalization

## Likely files to modify
- apps/engine-api/migrations/ (add company_meta JSONB column)
- apps/ingestion/src/scrapers/mod.rs (infer company size/industry)
- apps/ingestion/src/scrapers/ (extract company URL from detail pages)

## Rules
- All inferences are hints/labels, not canonical facts.
- Nullable — if inference fails, store null.
- Unit tests for size/industry inference patterns.

## Acceptance criteria
- [ ] company_meta column exists
- [ ] Size hint inferred for "startup"/"enterprise" mentions
- [ ] Industry hint inferred for common sectors
- [ ] Unit tests pass
- [ ] Cargo tests pass

## Verification commands
cargo test --manifest-path apps/ingestion/Cargo.toml
cargo check --manifest-path apps/ingestion/Cargo.toml

## Final response format
## What changed
## Why
## Files changed
## Verification
## Notes / follow-ups
```

---

## I6 — Dedup improvement with fuzzy matching

```
You are working in the Job Copilot monorepo.
First read: CLAUDE.md, apps/ingestion/src/ (dedup logic),
apps/engine-api/migrations/ (jobs table — dedupe_key column)

## Goal
Current dedup uses (source, source_job_id) as primary key. Improve with a secondary
fuzzy dedup check: if a new job from a different source has the same (company, title, ~salary)
as an existing job posted within 7 days, flag it as a potential duplicate instead of
creating a new entry. Store in duplicate_of: Option<Uuid> field.

This is cross-source dedup — Djinni and Work.ua may both have the same job posting.

## Inspect first
- apps/ingestion/src/ — dedup logic and upsert flow
- apps/engine-api/migrations/ — jobs table
- apps/engine-api/src/api/routes/jobs.rs — how variants are handled

## Likely files to modify
- apps/engine-api/migrations/ (add duplicate_of UUID nullable column)
- apps/ingestion/src/ (add cross-source fuzzy check before upsert)

## Rules
- Fuzzy match = same company (case-insensitive) + title similarity > 80% + within 7 days.
- Title similarity: Levenshtein distance or Jaccard on word bags (implement without external crate if possible, or use existing similarity utilities).
- If duplicate detected: store duplicate_of = existing_job_id but still insert the record (for variant tracking).
- Do not deduplicate same-source jobs (that's handled by existing dedup_key).

## Acceptance criteria
- [ ] duplicate_of column exists in jobs
- [ ] Cross-source duplicates detected and flagged
- [ ] Original job still shown in feed, duplicate variant accessible
- [ ] Unit test for fuzzy matching function
- [ ] Cargo tests pass

## Verification commands
cargo test --manifest-path apps/ingestion/Cargo.toml
cargo check --manifest-path apps/ingestion/Cargo.toml

## Final response format
## What changed
## Why
## Files changed
## Verification
## Notes / follow-ups
```

---

## I7 — Ingestion stats API

```
(See A11 in stability/prompts.md — this covers the same endpoint.
A11/A10 is the authoritative prompt if the endpoint is needed standalone.)
```

---

## I8 — Source health monitoring

```
You are working in the Job Copilot monorepo.
First read: CLAUDE.md, apps/ingestion/src/ (scraper daemon),
apps/engine-api/src/api/routes/ (health or sources endpoint)

## Goal
After each scraping run, record per-source health metrics:
{ source: String, run_at: DateTime, jobs_fetched: u32, jobs_upserted: u32,
  errors: u32, duration_ms: u64, status: "ok"|"partial"|"failed" }

Store in a new ingestion_runs table (rolling 7-day retention).
Expose GET /api/v1/sources/health showing last run stats per source.
If a source fetched 0 jobs in its last 3 runs: status = "degraded" in response.

## Inspect first
- apps/ingestion/src/ — scraper daemon run loop
- apps/engine-api/src/api/routes/sources.rs — existing sources endpoint
- apps/engine-api/migrations/ — check if ingestion_runs table exists

## Likely files to modify
- apps/engine-api/migrations/ (add ingestion_runs table)
- apps/ingestion/src/ (record run stats after each source completes)
  OR apps/engine-api/src/api/routes/sources.rs (compute on-the-fly from jobs table)
- apps/engine-api/src/api/routes/sources.rs (add /health endpoint)

## Rules
- Ingestion writes stats via engine-api internal endpoint or direct DB insert.
- 7-day rolling retention: delete runs older than 7 days on each insert.
- "degraded" status visible in web dashboard header as a small warning badge.

## Acceptance criteria
- [ ] ingestion_runs table exists
- [ ] Stats recorded after each source run
- [ ] GET /api/v1/sources/health returns per-source status
- [ ] "degraded" flag when source returns 0 jobs in last 3 runs
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
