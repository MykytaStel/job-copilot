# Current State Audit

Date: 2026-04-14

## Scope
Repository audit after the latest Phase 3 backend and application-detail workflow changes.

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

## Current implementation state
- `web` routes through `engine-api` for the active product screens
- profile CRUD and profile analysis are persisted in `engine-api`
- jobs read flows, application read flows, and search are available from `engine-api`
- application detail now supports notes, contacts, offer, `status`, and `due_date` writes through `engine-api`
- PostgreSQL full-text search and `search_vector` trigger/backfill logic are now in the backend migration layer
- local DB-backed verification exists for notes, contacts, offer, search, `status`, and `due_date`
- `apps/ingestion` now supports:
  - canonical job upsert into `jobs`
  - one adapter-backed normalization path
  - raw snapshot persistence into `job_variants`

## Remaining gap before the next architecture slice
- production source adapters, dedupe, and refresh logic are not implemented yet
- ML remains a sidecar skeleton and is not part of the core product flow
- contracts and roadmap docs need to stay aligned with the ingestion boundary as it expands

## Recommended next step
- extend ingestion beyond the first adapter-backed path
- keep `job_variants` as the raw/source-owned layer and `jobs` as the canonical read layer
- only then define the first read-only ML integration surface over canonical engine data

## See also
- `docs/05-roadmap/project-status.md`
- `docs/05-roadmap/current-focus.md`
- `docs/05-roadmap/future-work.md`
