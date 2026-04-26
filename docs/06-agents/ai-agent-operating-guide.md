# AI Agent Operating Guide — Job Copilot

This guide explains how to use Claude and Codex in the Job Copilot repo without creating bloated diffs or breaking contracts.

## Why this exists

The repo has several moving parts:

- Rust `engine-api` as canonical domain authority
- Rust `ingestion` for source scraping and lifecycle
- Python `ml` sidecar for enrichment/reranking/analytics
- React/Vite `web` dashboard
- shared contracts and docs

AI coding agents can be useful here only if each task is small, explicit, and verifiable.

## Assistant roles

### Claude

Best for:

- architecture planning
- product/technical tradeoffs
- reviewing current repo state
- creating implementation prompts
- explaining code
- reviewing Codex diffs

Claude output should normally be a plan, checklist, review, or prompt. Claude should not propose a giant rewrite unless explicitly requested.

### Codex

Best for:

- bounded implementation tasks
- tests and small refactors
- fixing a clear bug
- applying a prepared prompt

Codex should receive one task at a time with exact acceptance criteria.

## Standard workflow

1. Ask Claude for a plan.
2. Convert the plan into one Codex prompt.
3. Run Codex on the smallest slice.
4. Run verification commands.
5. Ask Claude to review the diff if the slice touches contracts, ranking, DB, auth, or ML.
6. Merge only when checks are clear.

## Good task size

Good:

- “Add profile ownership guard to profile-scoped routes and tests.”
- “Fix React Query invalidation after save/hide/bad-fit.”
- “Add freshness decay to deterministic ranking and tests.”
- “Split ML provider implementation without changing behavior.”

Too broad:

- “Improve the whole engine.”
- “Refactor web.”
- “Add AI learning.”
- “Clean legacy everywhere.”

If the task sounds broad, split it into vertical slices.

## Required prompt fields for Codex

Each Codex prompt should include:

- Goal
- Scope
- Inspect first
- Likely files to modify
- Files not allowed to modify
- Rules
- Acceptance criteria
- Verification commands
- Final response format

Templates are available in:

- `codex/_template-implementation-slice.md`
- `codex/_template-review-diff.md`

## Verification matrix

Use only the relevant commands for the current slice.

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
pnpm verify:phase8:db
pnpm local:ingestion:demo
```

## Review checklist

Before merge, check:

- Is the diff limited to the requested slice?
- Did any public contract change unintentionally?
- Did any domain truth move out of Rust?
- Did any LLM/ML output become canonical without validation?
- Are role IDs and filters structured?
- Are ranking and feedback reasons explainable?
- Are new DB changes migrated?
- Are tests updated?
- Are docs updated for architecture/contract changes?

## Recommended PR text

```md
## What changed

-

## Why

-

## How I verified it

- [ ] `pnpm typecheck`
- [ ] `pnpm build`
- [ ] `pnpm --dir apps/web test`
- [ ] `pnpm guard:web-api-imports`
- [ ] `cargo test --manifest-path apps/engine-api/Cargo.toml`
- [ ] `cargo check --manifest-path apps/engine-api/Cargo.toml`
- [ ] `cargo test --manifest-path apps/ingestion/Cargo.toml`
- [ ] `cargo check --manifest-path apps/ingestion/Cargo.toml`
- [ ] `cd apps/ml && python -m pytest`

## Risk level

- [ ] Low
- [ ] Medium
- [ ] High

## Notes

-
```
