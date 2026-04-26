# Block B — Scoring Improvements (12 tasks)

Each task below is a self-contained Codex prompt.

---

## B1 — Freshness decay for stale jobs

```
You are working in the Job Copilot monorepo.
First read: CLAUDE.md, apps/engine-api/src/services/ (scoring service),
apps/engine-api/src/api/routes/jobs.rs, apps/engine-api/src/api/routes/search.rs

## Goal
Jobs older than 14 days should receive a score penalty in the deterministic scoring layer.
Penalty should be proportional: -5 at 14 days, -10 at 21 days, -15 at 30+ days.
Jobs older than 60 days should be flagged as stale in the UI response.

## Inspect first
- apps/engine-api/src/services/ — find the scoring/ranking service
- apps/engine-api/src/domain/ — scoring rules, score weights
- apps/engine-api/src/api/routes/search.rs — where scoring is applied
- apps/engine-api/migrations/ — jobs table columns (first_seen_at, last_confirmed_active_at)

## Likely files to modify
- Scoring service file (wherever job score is computed)
- JobPresentationResponse (add stale: bool flag if not present)

## Rules
- Use last_confirmed_active_at if available, fall back to first_seen_at.
- Do not change the scoring interface/shape — add penalty as an internal signal.
- Add unit test for the decay function with fixture dates.

## Acceptance criteria
- [ ] Jobs 14-20 days old get -5 penalty
- [ ] Jobs 21-29 days old get -10 penalty
- [ ] Jobs 30+ days old get -15 penalty
- [ ] Jobs 60+ days old have stale=true in response
- [ ] Unit test covers all ranges
- [ ] Cargo tests pass

## Verification commands
cargo test --manifest-path apps/engine-api/Cargo.toml
cargo check --manifest-path apps/engine-api/Cargo.toml

## Final response format
## What changed
## Why
## Files changed
## Verification
## Notes / follow-ups
```

---

## B2 — Salary fit in scoring

```
You are working in the Job Copilot monorepo.
First read: CLAUDE.md, apps/engine-api/src/services/ (scoring),
apps/engine-api/src/domain/ (profile, job schemas),
apps/engine-api/migrations/ (salary columns)

## Goal
If the candidate profile has salary_min and salary_max set, compare against the job's
salary range. Scoring rules:
- Job salary fully within candidate range: +10
- Job salary overlaps candidate range: +5
- Job salary is below candidate min by >20%: -10
- Job has no salary info: 0 (no penalty, but flag missing_salary=true)

## Inspect first
- Profile domain struct — salary_min, salary_max fields
- Job domain struct — salary_min, salary_max, salary_currency fields
- Scoring service — where fit signals are aggregated

## Likely files to modify
- Scoring service (add salary_fit signal)
- JobPresentationResponse (add salary_fit label if not present)

## Rules
- Normalize currency before comparing (USD base).
- If profile has no salary expectation, skip this signal entirely.
- Add unit tests for overlap scenarios.

## Acceptance criteria
- [ ] Salary signal only activates when profile has salary range set
- [ ] Correct penalty/bonus for all 3 scenarios
- [ ] Currency normalization to USD (approximate conversion is fine)
- [ ] Unit tests pass

## Verification commands
cargo test --manifest-path apps/engine-api/Cargo.toml
cargo check --manifest-path apps/engine-api/Cargo.toml

## Final response format
## What changed
## Why
## Files changed
## Verification
## Notes / follow-ups
```

---

## B3 — Company reputation in scoring

```
You are working in the Job Copilot monorepo.
First read: CLAUDE.md, apps/engine-api/src/services/ (scoring),
apps/engine-api/src/api/routes/feedback.rs (whitelist/blacklist logic),
apps/engine-api/src/db/ (company feedback queries)

## Goal
When computing a job's score, check the candidate's whitelist/blacklist for the job's company.
- Company is whitelisted: +5 bonus
- Company is blacklisted: -20 penalty (job should effectively be suppressed)
This signal should be applied in the deterministic scoring layer.

## Inspect first
- Scoring service — where per-job score is assembled
- DB functions for whitelist/blacklist lookup by company name
- Job domain struct — company field

## Likely files to modify
- Scoring service (add company_reputation signal)
- May need to pass whitelist/blacklist set into scoring context

## Rules
- Company name comparison should be case-insensitive.
- Blacklisted jobs should still appear in the response (not filtered) — just ranked very low.
- Add unit test.

## Acceptance criteria
- [ ] Whitelisted company gives +5 to score
- [ ] Blacklisted company gives -20 to score
- [ ] Case-insensitive company matching
- [ ] Unit test covers both cases

## Verification commands
cargo test --manifest-path apps/engine-api/Cargo.toml
cargo check --manifest-path apps/engine-api/Cargo.toml

## Final response format
## What changed
## Why
## Files changed
## Verification
## Notes / follow-ups
```

---

## B4 — Remote work preference in scoring

```
You are working in the Job Copilot monorepo.
First read: CLAUDE.md, apps/engine-api/src/services/ (scoring),
apps/engine-api/src/domain/ (profile struct — work mode preference),
apps/engine-api/migrations/ (profile columns)

## Goal
If the candidate profile has a work_mode_preference (remote_only / hybrid / onsite / any),
apply a scoring signal:
- remote_only profile + remote job: +8
- remote_only profile + onsite job: -10
- hybrid profile + remote job: +3
- hybrid profile + onsite job: -3
- any preference: 0 signal (no opinion)

If the profile has no preference set, skip this signal.

## Inspect first
- Profile domain struct — work_mode_preference field (or similar)
- Job domain struct — remote_type field
- Scoring service — signal aggregation

## Likely files to modify
- Scoring service (add work_mode signal)
- Profile domain (add work_mode_preference if missing — needs migration)

## Rules
- If profile field doesn't exist yet, add it with a migration.
- Unit test all combinations.

## Acceptance criteria
- [ ] work_mode_preference field exists in profile
- [ ] All scoring combinations correct
- [ ] 0 signal when no preference
- [ ] Migration if field was added
- [ ] Tests pass

## Verification commands
cargo test --manifest-path apps/engine-api/Cargo.toml
cargo check --manifest-path apps/engine-api/Cargo.toml

## Final response format
## What changed
## Why
## Files changed
## Verification
## Notes / follow-ups
```

---

## B5 — Seniority fit in scoring

```
You are working in the Job Copilot monorepo.
First read: CLAUDE.md, apps/engine-api/src/services/ (scoring),
apps/engine-api/src/domain/ (profile — seniority, job — seniority_level),
apps/engine-api/src/domain/role/role_id.rs

## Goal
Match the candidate's seniority (junior/mid/senior/lead) against the job's inferred
seniority level. Scoring:
- Exact match: +10
- One level difference (e.g. senior candidate, mid job): -3
- Two level difference: -10
If job has no inferred seniority: 0 signal.

## Inspect first
- Profile seniority field (may be part of search_profile or candidate profile)
- Job seniority_level field and how it's inferred by ingestion
- Scoring service

## Likely files to modify
- Scoring service (add seniority_fit signal)

## Rules
- Seniority scale: junior < mid < senior < lead.
- If profile has no seniority set, skip this signal.
- Unit test.

## Acceptance criteria
- [ ] Exact match gives +10
- [ ] 1-level gap gives -3
- [ ] 2-level gap gives -10
- [ ] No-seniority-in-job gives 0
- [ ] Tests pass

## Verification commands
cargo test --manifest-path apps/engine-api/Cargo.toml

## Final response format
## What changed
## Why
## Files changed
## Verification
## Notes / follow-ups
```

---

## B6 — Language preference in scoring

```
You are working in the Job Copilot monorepo.
First read: CLAUDE.md, apps/engine-api/src/services/ (scoring),
apps/engine-api/src/domain/ (profile — preferred_languages),
apps/engine-api/migrations/

## Goal
If candidate profile has a language preference (Ukrainian/English/bilingual/any),
compare to job's language requirement:
- Match: +5
- Mismatch (e.g. English-only job, Ukrainian-only profile): -5
- Bilingual job or no preference: 0 signal

## Inspect first
- Profile struct — language preference field
- Job struct — language field (if inferred from job board source)
- Scoring service

## Likely files to modify
- Scoring service
- Profile domain (add preferred_language field + migration if missing)

## Acceptance criteria
- [ ] Language field exists in profile
- [ ] Scoring signal applied correctly
- [ ] 0 signal for bilingual/no-preference
- [ ] Tests pass

## Verification commands
cargo test --manifest-path apps/engine-api/Cargo.toml
cargo check --manifest-path apps/engine-api/Cargo.toml

## Final response format
## What changed
## Why
## Files changed
## Verification
## Notes / follow-ups
```

---

## B7 — Role matching improvement with aliases

```
You are working in the Job Copilot monorepo.
First read: CLAUDE.md, apps/engine-api/src/domain/role/role_id.rs,
apps/engine-api/src/services/ (role matching),
apps/engine-api/src/api/routes/roles.rs

## Goal
Improve role alias matching so that "Full Stack Developer" maps to both frontend
and backend role IDs, "Node.js Developer" maps to backend, "ML Engineer" maps to
data/ml role, etc. Expand the alias table in role_id.rs or wherever aliases live.

Add at least 20 new aliases covering common Ukrainian job board title variations:
- "Fullstack" / "Full-Stack" → frontend + backend
- "Node Developer" → backend
- "Vue Developer" / "React Developer" → frontend
- "DevOps Engineer" / "SRE" → devops
- "Data Scientist" / "ML Engineer" → data_ml
- "QA Engineer" / "Tester" / "SDET" → qa
- "Business Analyst" / "BA" → business_analysis

## Inspect first
- apps/engine-api/src/domain/role/role_id.rs — existing alias map
- Test files for role matching

## Likely files to modify
- apps/engine-api/src/domain/role/role_id.rs (alias table)
- Role matching tests

## Rules
- Aliases are case-insensitive.
- No new role IDs — only new aliases for existing roles.
- Existing aliases must not break.

## Acceptance criteria
- [ ] 20+ new aliases added
- [ ] All new aliases resolve to correct role IDs
- [ ] Existing alias tests still pass
- [ ] Cargo tests pass

## Verification commands
cargo test --manifest-path apps/engine-api/Cargo.toml

## Final response format
## What changed
## Why
## Files changed
## Verification
## Notes / follow-ups
```

---

## B8 — Skill gap weight: must-have vs nice-to-have

```
You are working in the Job Copilot monorepo.
First read: CLAUDE.md, apps/engine-api/src/services/ (scoring, skill matching),
apps/ml/app/scoring_routes.py (fit analyze)

## Goal
Currently skill matching may treat all missing skills equally. Introduce a distinction:
- Skills in the job's required/must-have section: missing = larger penalty
- Skills in the preferred/nice-to-have section: missing = smaller penalty
- Candidate has the skill regardless of section: bonus

If the job description doesn't distinguish sections, use frequency and position heuristics
(skills mentioned first or multiple times = more important).

## Inspect first
- apps/engine-api/src/services/ — skill gap calculation
- apps/ml/app/scoring_routes.py — fit analyze signal weights
- Job domain struct — required_skills vs optional_skills fields (if they exist)

## Likely files to modify
- Scoring service (skill weight adjustments)
- ML fit analyze (adjust weight constants)

## Rules
- Backwards compatible — if no distinction available, use equal weights.
- Add unit test for differentiated scoring.

## Acceptance criteria
- [ ] Must-have skill gap penalizes more than nice-to-have gap
- [ ] Both Rust and Python scoring respect the distinction
- [ ] Unit tests pass

## Verification commands
cargo test --manifest-path apps/engine-api/Cargo.toml
cd apps/ml && python -m pytest

## Final response format
## What changed
## Why
## Files changed
## Verification
## Notes / follow-ups
```

---

## B9 — Application outcome signal in scoring

```
You are working in the Job Copilot monorepo.
First read: CLAUDE.md, apps/engine-api/src/services/ (scoring),
apps/engine-api/src/api/routes/applications.rs,
apps/engine-api/src/domain/ (application status enum)

## Goal
When the candidate has received offers or passed interviews for jobs with specific
role IDs / skills / companies, give a small boost to similar jobs:
- Application status = offer: +3 boost to jobs with same role_id
- Application status = interview: +1 boost to jobs with same role_id
This signal is profile-scoped and read from the applications table.

## Inspect first
- Scoring service — where per-job signals are assembled
- Applications table/struct — status field, role_id on the job
- DB query patterns

## Likely files to modify
- Scoring service (add outcome_signal)
- Service may need a new DB query to fetch application outcomes for profile

## Rules
- Only positive outcomes (interview, offer) create boost.
- Signal is small — it should not dominate deterministic score.
- Cache the outcome lookup per profile per request (not per job).

## Acceptance criteria
- [ ] Jobs with same role_id as offered/interviewed get boost
- [ ] No boost for rejected/saved/applied status
- [ ] Unit test with fixture applications
- [ ] Cargo tests pass

## Verification commands
cargo test --manifest-path apps/engine-api/Cargo.toml
cargo check --manifest-path apps/engine-api/Cargo.toml

## Final response format
## What changed
## Why
## Files changed
## Verification
## Notes / follow-ups
```

---

## B10 — Score explanation labels in Job Detail

```
You are working in the Job Copilot monorepo.
First read: CLAUDE.md, apps/engine-api/src/api/routes/jobs.rs (job detail response),
apps/engine-api/src/services/ (scoring),
apps/web/src/components/job-detail/ or apps/web/src/pages/JobDetail.tsx

## Goal
In the Job Detail page, show the top 3 scoring signals that contributed to this job's
rank. Examples: "Strong skill match (+12)", "Salary within range (+10)",
"Company on whitelist (+5)", "Job is 21 days old (-10)".

Backend: extend the JobPresentationResponse (or a sub-field) with
score_signals: [{ label: String, delta: i32 }] (top 3, sorted by abs(delta) desc).
Web: render these as small chips/tags in the Match tab.

## Inspect first
- apps/engine-api/src/api/routes/jobs.rs — JobPresentationResponse struct
- Scoring service — where signals are computed (they must be collected, not just summed)
- apps/web/src/components/job-detail/ — Match tab component
- packages/contracts/src/jobs.ts — shared TS types

## Likely files to modify
- Scoring service (collect signals into Vec<ScoreSignal>)
- JobPresentationResponse (add score_signals field)
- apps/engine-api/src/api/routes/jobs.rs
- packages/contracts/src/jobs.ts
- apps/web/src/components/job-detail/ (Match tab)

## Rules
- score_signals is optional — return empty vec if scoring was skipped.
- Label strings should be human-readable, not code names.
- No breaking change to existing fields.

## Acceptance criteria
- [ ] score_signals in API response for job detail
- [ ] Top 3 signals (by absolute delta) included
- [ ] Web renders chips in Match tab
- [ ] Positive delta = green, negative = red
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

## B11 — Bootstrap training data from user_events

```
You are working in the Job Copilot monorepo.
First read: CLAUDE.md, apps/ml/app/ (bootstrap files),
apps/engine-api/src/api/routes/reranker_dataset.rs,
apps/engine-api/src/api/routes/events.rs

## Goal
Create or improve a script/service that generates labeled training examples for the
reranker from user_events:
- event_type = "save" or "apply" → label = 1 (positive)
- event_type = "hide" or "bad_fit" → label = 0 (negative)
- event_type = "view" with short dwell time → label = 0
- event_type = "view" with long dwell time → label = 1

The bootstrap endpoint already exists at POST /ml/api/v1/reranker/bootstrap.
This task is to ensure it correctly reads from the engine's outcome dataset endpoint
(GET /api/v1/reranker/dataset) and generates meaningful labeled examples.

## Inspect first
- apps/ml/app/ — bootstrap_training.py or similar
- apps/engine-api/src/api/routes/reranker_dataset.rs — dataset endpoint response shape
- apps/ml/app/scoring_routes.py — bootstrap route handler

## Likely files to modify
- apps/ml/app/bootstrap_training.py or similar file
- apps/ml/tests/ (update bootstrap tests)

## Rules
- Do not train on fewer than 10 examples — log warning and skip.
- Preserve existing bootstrap interface.
- Do not use raw ambiguous events (view without dwell time is unreliable alone).

## Acceptance criteria
- [ ] Bootstrap correctly labels save/apply as positive, hide/bad_fit as negative
- [ ] Skips training if < 10 labeled examples, logs warning
- [ ] ML pytest passes

## Verification commands
cd apps/ml && python -m pytest tests/test_bootstrap*.py -v

## Final response format
## What changed
## Why
## Files changed
## Verification
## Notes / follow-ups
```

---

## B12 — Automatic reranker retrain pipeline

```
You are working in the Job Copilot monorepo.
First read: CLAUDE.md, apps/ml/app/scoring_routes.py (bootstrap endpoint),
apps/ml/app/ (reranker training code),
apps/engine-api/src/api/routes/reranker_dataset.rs

## Goal
After bootstrap training completes, if the new model has validation_accuracy > 0.65
and was trained on >= 30 examples, automatically persist it as the active model
(replace the current model weights file). If accuracy is below threshold, keep the
old model and log a warning.

Also: add a GET /ml/api/v1/reranker/status endpoint that returns:
{ model_version: str, trained_at: Optional[datetime], example_count: int,
  accuracy: Optional[float], is_functional: bool }
(is_functional = trained on >= 30 examples AND accuracy >= 0.65)

## Inspect first
- apps/ml/app/scoring_routes.py — bootstrap workflow
- apps/ml/app/ — model persistence, where weights are stored
- apps/ml/tests/test_bootstrap*.py — existing tests

## Likely files to modify
- apps/ml/app/scoring_routes.py (auto-persist logic)
- apps/ml/app/ (new status endpoint or extend existing metrics route)
- apps/ml/tests/

## Rules
- Model file paths must come from settings, not be hardcoded.
- Do not replace model mid-request (atomic swap after training).
- Threshold constants should be in settings.py.

## Acceptance criteria
- [ ] Model auto-persisted after successful bootstrap with >= 30 examples and accuracy >= 0.65
- [ ] Old model kept if threshold not met
- [ ] GET /ml/api/v1/reranker/status returns correct state
- [ ] ML pytest passes

## Verification commands
cd apps/ml && python -m pytest

## Final response format
## What changed
## Why
## Files changed
## Verification
## Notes / follow-ups
```
