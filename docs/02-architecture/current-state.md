# Current State Audit

Date: 2026-04-11

## Scope
Repository audit after switching the active frontend path to `engine-api`.

## Current structure
- `apps/web` is the active React frontend
- `apps/engine-api` is the canonical backend API for frontend integration
- `apps/ingestion` exists as the Rust ingestion service folder
- `apps/ml` exists as the Python ML service folder
- `packages/contracts` remains the shared TypeScript contracts workspace

## Files and folders that should not be committed
- dependency installs: `node_modules/`, `.pnpm-store/`
- build outputs: `dist/`, `build/`, `coverage/`, `target/`
- local env and runtime data: `.env*`, `data/`
- Python cache and virtualenv files: `__pycache__/`, `*.pyc`, `.pytest_cache/`, `.mypy_cache/`, `.ruff_cache/`, `.venv/`, `venv/`, `.coverage`
- local machine artifacts: `.DS_Store`, `__MACOSX/`, `*.log`, `.claude/settings.local.json`

## Naming state
- use `apps/engine-api` as the backend service path
- use `packages/contracts` as the shared contracts path
- keep `@job-copilot/shared` as the import/package name

## Additional findings
- `web` now routes only through `engine-api`-backed screens
- profile CRUD and analysis are now persisted in `engine-api`
- jobs and applications read flows are available from `engine-api`
- several non-MVP legacy screens have been removed from the active router pending replacement APIs

## Recommended next step
- remove leftover legacy-only source files and docs
- continue migrating missing application write flows into `engine-api`
- add integration coverage for the new profile endpoints against Postgres
