# Current State Audit

Date: 2026-04-10

## Scope
Repository audit for incremental migration readiness.

Constraints followed:
- no folder renames
- no app logic changes
- no runtime behavior changes

## Current structure
- `apps/web` exists and is the active React frontend
- `apps/api-legacy` exists and is the active Fastify/TypeScript backend
- `apps/engine-api` exists as the new Rust backend skeleton
- `apps/engine-api` now has optional Postgres bootstrapping and embedded SQL migrations for the first shared runtime schema foundation
- `apps/ingestion` exists as a Rust service folder, but currently has no `Cargo.toml`
- `apps/ml` exists as the Python service folder
- `packages/contracts` is the workspace path for shared TypeScript contracts
- the package import name remains `@job-copilot/shared` for compatibility during migration

## Files and folders that should not be committed
- dependency installs: `node_modules/`, `.pnpm-store/`
- build outputs: `dist/`, `build/`, `coverage/`, `target/`
- local env and runtime data: `.env*`, `data/`, `*.sqlite*`
- Python cache and virtualenv files: `__pycache__/`, `*.pyc`, `.pytest_cache/`, `.mypy_cache/`, `.ruff_cache/`, `.venv/`, `venv/`, `.coverage`
- local machine artifacts: `.DS_Store`, `__MACOSX/`, `*.log`, `.claude/settings.local.json`

Observed local-only artifacts already present in the tree:
- `.DS_Store`
- `apps/.DS_Store`
- `apps/web/node_modules/`

## Naming state
Current naming after structural cleanup:
- use `apps/api-legacy` as the legacy backend path
- use `packages/contracts` as the shared contracts path
- keep `@job-copilot/shared` as the import/package name for now to avoid broad code churn

Remaining compatibility note:
- `pnpm-lock.yaml` has been refreshed and now matches `apps/api-legacy` and `packages/contracts`

## Additional findings
- `docs/04-development/coding-rules.md` now exists and aligns with the incremental migration approach
- `apps/api-legacy` remains the active CRUD backend on SQLite
- `apps/engine-api` can now initialize Postgres and apply embedded migrations, but it does not yet own the migrated data flows
- the repository root is not currently inside a Git working tree, so tracked-vs-untracked status could not be verified from this directory
- `pnpm typecheck` is currently blocked by missing local installs; `tsc` is not available in the workspace until dependencies are installed

## Recommended next step
Install workspace dependencies and rerun verification:
- run `pnpm install`
- run `pnpm typecheck`
- keep package import renaming as a separate, explicit follow-up if desired
