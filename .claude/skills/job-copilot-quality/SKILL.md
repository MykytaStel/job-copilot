name job-copilot-quality
description Use for Job Copilot code planning, review, implementation, or refactoring. Enforces small verifiable slices, Rust domain authority, ML-as-enrichment, frontend contract discipline, and minimal surgical changes.
license MIT

# Job Copilot Quality Skill

Use this skill whenever writing, reviewing, or planning code in the Job Copilot repository.

## Core behavior

Think before coding, keep changes simple, edit surgically, and define verification before claiming completion.

For non-trivial tasks, start with:

- goal
- assumptions
- files to inspect
- risks/tradeoffs
- acceptance criteria
- verification commands

## Project invariants

- `apps/engine-api` is the canonical domain authority.
- `apps/ingestion` owns fetch/scrape/normalize/dedupe/lifecycle/upsert.
- `apps/ml` owns enrichment/reranking/analytics sidecar behavior but is not authoritative domain state.
- `apps/web` owns UI and user actions but must not own business/domain logic.
- `packages/contracts` owns shared schema contracts.
- DTOs, role IDs, filters, feedback states, and ranking explanations must remain explicit and stable.

## Do not

- Do not invent DTO fields, role IDs, endpoints, DB columns, or env vars.
- Do not move domain truth into React components or ML prompts.
- Do not make LLM output canonical without Rust validation.
- Do not add broad abstractions for one use case.
- Do not reformat unrelated files.
- Do not delete pre-existing dead code unless the task is cleanup.
- Do not hardcode production UI data where API data exists.
- Do not expose raw internal errors, filesystem paths, secrets, or token values.

## Backend checklist

For `apps/engine-api` changes:

- route handlers stay thin
- services/domain modules own rules
- migrations exist for schema changes
- ranking/feedback/lifecycle/security changes have tests
- public/debug responses are safe and stable

Verify with:

```bash
cargo test --manifest-path apps/engine-api/Cargo.toml
cargo check --manifest-path apps/engine-api/Cargo.toml
```

## Ingestion checklist

For `apps/ingestion` changes:

- adapter boundaries stay clear
- canonical job shape stays stable
- source quirks stay in variants/adapters
- dedupe and lifecycle behavior are preserved
- fixtures or smoke checks cover parser/scraper changes where possible

Verify with:

```bash
cargo test --manifest-path apps/ingestion/Cargo.toml
cargo check --manifest-path apps/ingestion/Cargo.toml
```

## ML checklist

For `apps/ml` changes:

- route wiring stays separate from reusable logic
- provider selection is explicit
- template provider remains a safe default unless configured otherwise
- paid providers are never default
- artifacts and labels stay inspectable
- token/secret values are never logged

Verify with:

```bash
cd apps/ml
python -m pytest
```

## Web checklist

For `apps/web` changes:

- API transport stays typed and domain-specific
- `src/api.ts` remains a compatibility facade
- React Query invalidation follows mutations
- UI does not own scoring, role, lifecycle, or feedback rules
- visual redesign is separated from contract/data changes

Verify with:

```bash
pnpm --dir apps/web typecheck
pnpm --dir apps/web lint
pnpm --dir apps/web test
pnpm guard:web-api-imports
```

## Final answer format

Use:

```md
## What changed

## Why

## Files changed

## Verification

## Notes / follow-ups
```

If something could not be verified, state it directly.
