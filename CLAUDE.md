# CLAUDE.md

You are working in the Job Copilot monorepo.

Job Copilot is a candidate intelligence and action system, not a simple job board. It must understand the candidate, ingest and normalize jobs, rank jobs for that candidate, explain fit and gaps, support job-search actions, and learn from outcomes.

## Operating principles

### 1. Think before coding

Before implementing a non-trivial task:

- State the goal in one or two sentences.
- List the assumptions you are making.
- Name any ambiguity instead of silently choosing one interpretation.
- Prefer asking for clarification when the answer changes architecture, data contracts, or user-visible behavior.
- Push back when a smaller, safer slice solves the same goal.

For trivial one-line fixes, use judgment and keep the response short.

### 2. Simplicity first

Build the minimum correct slice.

- No speculative abstractions.
- No generic framework unless there is a real second use case.
- No broad rewrites while fixing a local bug.
- No new package unless the benefit is obvious and the dependency cost is acceptable.
- Prefer explicit DTOs, small services, testable helpers, and stable contracts.

Ask yourself before every implementation: would a senior engineer call this over-engineered? If yes, simplify.

### 3. Surgical changes

Touch only files that are required for the current slice.

- Do not reformat unrelated files.
- Do not rename public API fields unless the task explicitly changes the contract.
- Do not remove pre-existing dead code unless the task is a cleanup slice.
- If you notice unrelated dead code, mention it in notes instead of deleting it.
- Remove only the imports, variables, helpers, or tests made unused by your own changes.

Every changed line must trace back to the current user request.

### 4. Goal-driven execution

Convert each task into a verifiable goal.

Good pattern:

1. Define the expected behavior.
2. Add or update the smallest useful tests.
3. Implement the change.
4. Run targeted checks.
5. Report exactly what changed and what was verified.

Do not say “done” unless the relevant checks passed or you clearly explain what could not be run.

## Project architecture

Current repo shape:

- `apps/engine-api` — canonical Rust backend and domain authority.
- `apps/ingestion` — Rust source fetch, scrape, normalize, dedupe, lifecycle, and canonical job upserts.
- `apps/ml` — Python ML/LLM sidecar for enrichment, reranking, analytics, coaching, and future provider integration.
- `apps/web` — React + Vite + TypeScript operator UI / dashboard.
- `packages/contracts` — shared schemas and contracts.
- `docs` — product, architecture, domain, development, roadmap, and agent docs.
- `codex` — bounded implementation prompts for Codex.

Core rule: domain truth lives in Rust. The frontend and LLM sidecar may display, enrich, or request data, but they must not become the canonical source of domain state.

## Domain guardrails

Never violate these without an explicit architecture decision:

- Canonical identifiers, role catalog, scoring, lifecycle state, and validation belong in `apps/engine-api`.
- LLM output is enrichment, not authoritative state.
- ML output must be validated by Rust-side contracts before it affects persisted domain state.
- Role identity must use canonical role IDs / role catalog, not ad-hoc strings.
- Region, work mode, source, role, blacklist, whitelist, and profile filters must stay structured, not flattened into free text.
- Feedback behavior must remain explainable: saved, hidden, bad fit, whitelist, blacklist, application outcomes.
- Search/ranking responses should expose reasons and safe metadata, not raw internal errors or filesystem paths.
- Do not introduce mock data where real ingested data or deterministic fixtures can be used.

## Current product direction

Prioritize:

1. Robust profile understanding and search-profile preferences.
2. Source-aware ingestion and search.
3. Deterministic ranking baseline with clear fit explanations.
4. Lists and controls: saved, hidden, bad fit, whitelist, blacklist.
5. Learning loop: user events → labeled outcomes → reranker dataset → trained reranker.
6. Market intelligence from already-ingested data.
7. LLM/template enrichment that is useful but never canonical.

Avoid for now unless explicitly requested:

- Paid LLM dependency as the default path.
- Semantic embeddings before enough labeled data exists.
- Auth/multi-user expansion before the product flow is stable.
- Large UI redesigns mixed with backend/domain work.

## Backend rules: `apps/engine-api`

Use Rust as the source of truth.

- Keep route handlers thin: parse request, call service/domain layer, return typed response.
- Put scoring, role matching, ownership checks, feedback logic, and lifecycle behavior into testable services or domain modules.
- Preserve API error contract: validation errors should be explicit, missing resources should be distinct, and DB-unavailable paths should be clear.
- Add tests for new matching, ranking, feedback, ownership, lifecycle, migration, or contract logic.
- Do not expose raw filesystem paths, loader errors, secret values, or internal traces through public/debug endpoints.
- Use migrations for DB changes; avoid runtime schema assumptions without migration coverage.

Useful checks:

```bash
cargo test --manifest-path apps/engine-api/Cargo.toml
cargo check --manifest-path apps/engine-api/Cargo.toml
pnpm verify:phase8:db
```

## Ingestion rules: `apps/ingestion`

Ingestion is responsible for source fetch, scrape, normalize, dedupe, lifecycle, and canonical job upserts.

- Respect source boundaries and adapter responsibilities.
- Keep canonical job shape stable.
- Store source-specific payloads in variants when needed, but do not let source quirks leak into canonical domain fields.
- Preserve dedupe behavior and lifecycle semantics: active, inactive, reactivated.
- Do not write `search_vector` directly if DB triggers own search indexing.
- Do not make scraper changes without fixture or smoke coverage where possible.

Useful checks:

```bash
cargo test --manifest-path apps/ingestion/Cargo.toml
cargo check --manifest-path apps/ingestion/Cargo.toml
pnpm local:ingestion:demo
```

## ML rules: `apps/ml`

ML is an enrichment sidecar.

- Keep FastAPI route wiring separate from reusable scoring/enrichment/provider logic.
- Keep provider selection explicit: template, Ollama, OpenAI/Anthropic only when configured.
- Do not make paid API calls the default path.
- Do not add heavy ML dependencies unless the slice proves the need.
- Keep DTOs in dedicated model files; avoid inline untyped dict sprawl.
- Do not train from raw, ambiguous events if the engine already exports normalized outcome datasets.
- Preserve inspectability for reranker artifacts and training data.

Useful checks:

```bash
cd apps/ml
python -m pytest
```

## Frontend rules: `apps/web`

Web is a React + Vite + TypeScript dashboard.

- Do not move domain logic into UI components.
- Keep API transport code in domain-specific modules under `src/api/`; keep `src/api.ts` as a compatibility facade only.
- Prefer direct domain imports over barrel/facade imports when the guard requires it.
- Use React Query invalidation intentionally after save/hide/bad-fit/profile changes.
- Keep UI dense, quiet, and operator-focused: dark base, restrained gradients, readable cards, low-noise actions, fit/explanation first.
- Do not mix large visual redesigns with data contract work.

Useful checks:

```bash
pnpm --dir apps/web typecheck
pnpm --dir apps/web lint
pnpm --dir apps/web test
pnpm guard:web-api-imports
pnpm build
```

## Documentation rules

Update docs when a slice changes architecture, API contracts, data flow, or verification commands.

High-priority docs:

- `docs/00-master-plan.md`
- `docs/05-roadmap/current-focus.md`
- `docs/engineering-checklist.md`
- relevant `docs/01-product`, `docs/02-architecture`, `docs/03-domain`, or `docs/04-development` files
- task prompts in `codex/` when a repeatable implementation slice is created

Do not let docs claim a feature is complete unless tests or manual verification prove it.

## Recommended response format

For planning / architecture review:

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

For implementation summary:

```md
## What changed

## Why

## Files changed

## Verification

## Notes / follow-ups
```

## PR checklist

Before claiming a PR is ready, check what is relevant:

```bash
pnpm typecheck
pnpm build
pnpm --dir apps/web test
pnpm guard:web-api-imports
cargo test --manifest-path apps/engine-api/Cargo.toml
cargo check --manifest-path apps/engine-api/Cargo.toml
cargo test --manifest-path apps/ingestion/Cargo.toml
cargo check --manifest-path apps/ingestion/Cargo.toml
cd apps/ml && python -m pytest
```

If a check is not relevant or cannot be run, say so explicitly.

## Absolute don'ts

- Do not batch unrelated work into one pass.
- Do not hide uncertainty.
- Do not invent role IDs, DTO fields, endpoints, env vars, or database columns without checking existing code/docs first.
- Do not make LLM responses canonical.
- Do not reintroduce legacy-only screens into active routing unless requested.
- Do not hardcode demo values into production UI.
- Do not expose secrets or raw internal errors.
- Do not delete existing docs, hooks, agents, or prompts as “cleanup” unless the task asks for cleanup.
