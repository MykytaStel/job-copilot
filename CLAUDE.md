# CLAUDE.md

You are working in the **Job Copilot** monorepo.

Job Copilot is a candidate intelligence and action system, not a simple job board. It should understand the candidate, ingest and normalize jobs, rank jobs for that candidate, explain fit and gaps, support job-search actions, and learn from outcomes over time.

This file is the main operating guide for Claude. It should stay short enough to be read often, but specific enough to protect the architecture.

---

## Product intent

Job Copilot should help a user answer these questions quickly:

1. What jobs are actually relevant to me?
2. Why is this job a good or bad fit?
3. What skills, salary, language, region, or work-mode gaps matter?
4. What should I save, hide, apply to, follow up on, or reject?
5. What does my behavior say about my search preferences?
6. How can my CV/profile be improved for a specific job?

The product direction is **operator-focused**: dense, useful, explainable, and low-noise.

---

## Current repo shape

```txt
apps/
  engine-api/   # Rust backend; canonical domain authority
  ingestion/    # Rust ingestion service; source fetch, scrape, normalize, dedupe, lifecycle
  ml/           # Python ML/LLM sidecar; enrichment, reranking, analytics, coaching
  web/          # React + Vite + TypeScript dashboard/operator UI
packages/
  contracts/    # shared schemas/contracts
shared/
  rust/         # shared Rust crates/utilities where applicable
docs/           # product, architecture, domain, development, roadmap, agent docs
codex/          # Codex rules, templates, and bounded task prompts
.claude/        # Claude-specific skills and reusable planning prompts
```

Important runtime notes:

- Active web shell source of truth: `apps/web/src/App.tsx` -> `apps/web/src/AppShell.tsx`.
- `apps/web/src/AppShellNew.tsx` is a compatibility re-export alias, not the active source of truth.
- `market_snapshots` are refreshed after successful ingestion upserts, but some market readers may still query live `jobs` directly.
- Runtime ML provider defaults and Docker Compose provider defaults may differ; check `apps/ml/app/settings.py` and `infra/docker-compose.yml` before changing provider behavior.

---

## Architecture rules

These are project invariants. Do not violate them unless the task explicitly asks for an architecture change and the change is documented.

1. **Rust `apps/engine-api` owns canonical domain truth.**
   - Profiles, jobs, applications, feedback, role IDs, scoring rules, lifecycle state, ownership checks, validation, and stable API contracts belong here.

2. **`apps/ingestion` owns job supply and source lifecycle.**
   - It fetches, scrapes, normalizes, dedupes, tracks lifecycle, and upserts canonical job data.
   - Source quirks should stay inside source adapters or source-specific variants.

3. **`apps/ml` is an enrichment sidecar, not canonical state.**
   - LLM/ML output may enrich, rank, summarize, coach, or suggest.
   - It must not become authoritative domain state without Rust-side validation.

4. **`apps/web` is an operator UI, not a domain owner.**
   - UI can display, filter, trigger actions, and orchestrate API calls.
   - It should not own scoring, matching, ownership, lifecycle, or canonical profile logic.

5. **Contracts must remain explicit and stable.**
   - Do not invent DTO fields, endpoints, env vars, role IDs, migrations, or database columns.
   - Search existing code/docs first.

6. **Ranking and feedback must stay explainable.**
   - Saved, hidden, bad fit, whitelist, blacklist, application outcomes, and reranker behavior should produce understandable reasons or safe metadata.

7. **Do not expose unsafe internals.**
   - Do not expose secrets, raw filesystem paths, raw loader errors, internal traces, or token values in public/debug responses.

---

## Working model

Use one vertical slice at a time.

A good slice has:

- a clear goal;
- explicit scope;
- files to inspect first;
- likely files to modify;
- files/directories not to touch;
- acceptance criteria;
- verification commands;
- a final summary of what changed and what passed.

Do not batch unrelated work. Security, ML provider behavior, UI redesign, ingestion changes, docs cleanup, and architecture changes should usually be separate PRs.

---

## Claude responsibilities

Use Claude for:

- planning;
- architecture review;
- deciding where a change belongs;
- reducing scope;
- reviewing a Codex diff;
- writing Codex-ready prompts;
- explaining code and tradeoffs;
- turning vague ideas into small implementation slices.

Claude should not produce broad unverified rewrites. When asked to implement something non-trivial, first convert it into a safe slice.

Recommended planning output:

```md
## Goal
## Current understanding
## Assumptions / open questions
## Proposed slice
## Files likely touched
## Acceptance criteria
## Verification commands
## Risks / tradeoffs
```

Recommended implementation/review summary:

```md
## What changed
## Why
## Files changed
## Verification
## Notes / follow-ups
```

---

## Codex responsibilities

Codex should be used only for bounded implementation slices.

Before sending work to Codex, prefer using:

```txt
codex/CODEX.md
codex/_template-implementation-slice.md
codex/_template-review-diff.md
```

Task-specific prompts should live under:

```txt
codex/tasks/<area>/<task-name>.md
```

Examples:

```txt
codex/tasks/security/profile-ownership-and-ml-token-slice.md
codex/tasks/ml/cv-tailoring-endpoint.md
codex/tasks/web/settings-notification-preferences.md
codex/tasks/ingestion/analytics-freshness-widget.md
```

Codex prompts must include:

- exact goal;
- exact files/directories to inspect first;
- allowed modification scope;
- disallowed scope;
- acceptance criteria;
- verification commands;
- expected final response format.

---

## Backend rules: `apps/engine-api`

Use Rust as the source of truth.

Rules:

- Keep route handlers thin: parse request, call service/domain layer, return typed response.
- Put scoring, role matching, ownership checks, feedback logic, and lifecycle behavior into testable services/domain modules.
- Preserve API error semantics: validation errors, forbidden ownership, missing resources, and DB-unavailable paths should stay distinct.
- Add tests for matching, ranking, feedback, ownership, lifecycle, migrations, and contract logic.
- New DB state requires migrations.
- Public/debug response fields must be safe, stable, and not leak internals.
- Do not expose raw filesystem paths, loader errors, secret values, internal traces, or token values.

Useful checks:

```bash
cargo test --manifest-path apps/engine-api/Cargo.toml
cargo check --manifest-path apps/engine-api/Cargo.toml
pnpm verify:phase8:db
```

---

## Ingestion rules: `apps/ingestion`

Ingestion owns source fetch, scrape, normalize, dedupe, lifecycle, and canonical job upserts.

Rules:

- Respect source boundaries and adapter responsibilities.
- Keep canonical job shape stable.
- Store source-specific payloads in variants when needed.
- Do not let source quirks leak into canonical domain fields.
- Preserve lifecycle semantics: active, inactive, reactivated, first seen, last confirmed active.
- Do not write `search_vector` directly if DB triggers own indexing.
- Add fixture/smoke coverage for scraper/parser changes where possible.

Useful checks:

```bash
cargo test --manifest-path apps/ingestion/Cargo.toml
cargo check --manifest-path apps/ingestion/Cargo.toml
pnpm local:ingestion:demo
```

---

## ML rules: `apps/ml`

ML is an enrichment sidecar.

Rules:

- Keep FastAPI route wiring separate from reusable scoring/enrichment/provider logic.
- Keep provider selection explicit: template, Ollama, OpenAI/Anthropic only when configured.
- Do not make paid API calls the default path.
- Do not add heavy ML dependencies unless the slice proves the need.
- Keep DTOs in dedicated model files; avoid inline untyped dict sprawl.
- Do not train from raw ambiguous events when the engine exports normalized outcome datasets.
- Preserve inspectability for reranker artifacts and training data.
- Validate production security settings at startup.
- Never print token values.

Useful checks:

```bash
cd apps/ml && python -m pytest
```

---

## Frontend rules: `apps/web`

Web is a React + Vite + TypeScript dashboard.

Rules:

- Do not move domain logic into UI components.
- Keep API transport code in domain-specific modules under `src/api/`.
- Keep `src/api.ts` as a compatibility facade only when needed.
- Prefer direct domain imports over barrel/facade imports when the guard requires it.
- Use React Query invalidation intentionally after save/hide/bad-fit/profile changes.
- Keep UI dense, quiet, and operator-focused: dark base, restrained gradients, readable cards, low-noise actions, fit/explanation first.
- Do not mix large visual redesigns with data contract work.
- Do not render raw internal UUIDs to users; IDs may exist in URL paths or React keys, but not visible text.

Useful checks:

```bash
pnpm --dir apps/web typecheck
pnpm --dir apps/web lint
pnpm --dir apps/web test
pnpm guard:web-api-imports
pnpm build
```

---

## Documentation rules

Update docs when a slice changes architecture, API contracts, data flow, verification commands, or current product state.

High-priority docs:

```txt
docs/00-master-plan.md
docs/02-architecture/current-state.md
docs/02-architecture/adr-template.md
docs/04-development/verification-matrix.md
docs/05-roadmap/current-focus.md
docs/engineering-checklist.md
```

Agent/workflow docs:

```txt
AGENTS.md
CLAUDE.md
.claude/skills/job-copilot-quality/SKILL.md
codex/CODEX.md
codex/_template-implementation-slice.md
codex/_template-review-diff.md
docs/06-agents/ai-agent-operating-guide.md
```

Do not let docs claim a feature is complete unless tests or manual verification prove it.

---

## Current product direction

Prioritize:

1. Robust profile understanding and persisted search preferences.
2. Source-aware ingestion and search.
3. Deterministic ranking baseline with clear fit explanations.
4. Feedback controls: saved, hidden, bad fit, whitelist, blacklist.
5. Learning loop: user events -> labeled outcomes -> reranker dataset -> trained reranker.
6. Market intelligence from ingested data.
7. LLM/template enrichment that is useful but never canonical.
8. CV/profile tailoring for specific jobs.

Avoid for now unless explicitly requested:

- paid LLM dependency as the default path;
- semantic embeddings before enough labeled data exists;
- auth/multi-user expansion before the product flow is stable;
- Stripe/monetization before auth and product flow are stable;
- large UI redesigns mixed with backend/domain work.

---

## Current recommended next slices

Prefer this order unless the user chooses otherwise:

1. **PR #30 cleanup and merge**
   - Normalize AI-agent docs formatting.
   - Remove hidden/bidirectional Unicode characters.
   - Keep `CLAUDE.md`, `AGENTS.md`, `codex/`, `.claude/skills`, and `docs/06-agents` consistent.

2. **Security: profile ownership guards**
   - Profile-scoped engine routes must verify that `AuthUser.profile_id` matches path `{id}`.
   - Mismatched owner gets `403`.
   - Missing profile remains a missing-resource response.

3. **Security: ML internal token production validation**
   - Production ML startup must fail fast if the internal token is absent.
   - Do not log token values.

4. **CV Tailoring v1**
   - ML endpoint + engine/web entrypoint.
   - Keep suggestions explainable and non-canonical unless persisted through validated engine contracts.

5. **Settings expansion**
   - Dedicated notification preferences.
   - More profile preference controls beyond the minimal settings route.

6. **Analytics freshness widget**
   - Show ingestion recency/feed freshness in analytics.

7. **Market snapshot readers**
   - Gradually move market read-side from live `jobs` queries to snapshots when it gives a clear benefit.

---

## PR checklist

Before saying a PR is ready, run what is relevant.

General docs/process PR:

```bash
git diff --check
```

Frontend:

```bash
pnpm typecheck
pnpm build
pnpm --dir apps/web test
pnpm guard:web-api-imports
```

Engine API:

```bash
cargo test --manifest-path apps/engine-api/Cargo.toml
cargo check --manifest-path apps/engine-api/Cargo.toml
```

Ingestion:

```bash
cargo test --manifest-path apps/ingestion/Cargo.toml
cargo check --manifest-path apps/ingestion/Cargo.toml
```

ML:

```bash
cd apps/ml && python -m pytest
```

If a check is not relevant or cannot be run, say so explicitly.

---

## Absolute don'ts

- Do not batch unrelated work into one pass.
- Do not hide uncertainty.
- Do not invent role IDs, DTO fields, endpoints, env vars, migrations, or DB columns without checking existing code/docs first.
- Do not make LLM responses canonical.
- Do not move domain truth into web or ML.
- Do not reintroduce legacy-only screens into active routing unless requested.
- Do not hardcode demo values into production UI.
- Do not expose secrets, raw internal errors, token values, file paths, or loader traces.
- Do not delete existing docs, hooks, agents, or prompts as cleanup unless the task is explicitly a cleanup slice.
