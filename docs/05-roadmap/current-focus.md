# Current Focus

Date: 2026-04-14

See also:
- `docs/05-roadmap/project-status.md`
- `docs/05-roadmap/backlog.md`
- `docs/05-roadmap/future-work.md`
- `docs/05-roadmap/milestones.md`

## Active milestone
- Milestone 9 — Ingestion and ML handoff

## Current focus
- [x] create the first ingestion-safe canonical job upsert path over DB-managed `search_vector`
- [x] persist the first source-specific raw snapshot / variant layer in Postgres
- [x] add first refresh/change-detection semantics for repeated source ingestion
- [ ] define the first read-only `ml` integration surface over canonical engine data
- [ ] document the service boundary after Phase 8 stabilization

## Focus details

### 1. Phase 8 outcome
Current state:
- `engine-api` already supports patching `status` and `due_date`
- `apps/web/src/api.ts` already exposes patch helpers for both fields
- `web` application detail now supports notes, contacts, offer, status, and due date
- DB-backed verification passed on a real local Postgres instance
- `scripts/verify_phase8_db.sh` now automates the verification flow
- the verification script was hardened to avoid port collisions with local dev servers

Verified in the DB-backed smoke run:
- latest migrations apply cleanly on Postgres
- seeded jobs are backfilled into `search_vector`
- trigger updates `search_vector` on insert and update
- notes, contacts, offer, search, status, and due date all round-trip through `engine-api`

### 2. Next architecture slice
Keep in scope:
- ingestion compatibility with DB-managed search indexing
- read-only ML integration over canonical engine data
- architecture docs for the stabilized service boundary

Done in the first ingestion slice:
- `apps/ingestion` now has a Rust binary with a canonical job upsert path
- ingestion writes canonical job fields into `jobs`
- ingestion now persists source-specific raw payloads into `job_variants`
- ingestion does not write `search_vector`
- real local verification confirmed that `engine-api` search sees ingestion-upserted jobs through the DB trigger
- ingestion now also has a first adapter-backed normalization path via `mock-source`
- real local verification confirmed that adapter-normalized jobs are searchable through the DB-managed index
- raw snapshots and canonical jobs are now separated in storage instead of only in memory
- repeat ingestion for the same `source + source_job_id` now reports `created` / `updated` / `unchanged` at the variant layer

Do not expand scope yet:
- no ingestion-owned search index writes outside DB-managed `search_vector`
- no ML-owned core writes
- no backend ownership changes away from `engine-api`

## After current focus
- harden search and ranking on seeded DB data
- extend ingestion into dedupe / refresh behavior on top of `job_variants`
- add the first read-only `ml` endpoint over canonical engine data

## Default rules
- keep `engine-api` as the only write authority
- do not introduce `offer_received` into `application.status`
- keep one offer aggregate per application for now
- prefer small, incremental changes
