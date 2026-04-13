# Future Work

Date: 2026-04-14

See also:
- `docs/05-roadmap/current-focus.md`
- `docs/05-roadmap/project-status.md`
- `docs/05-roadmap/backlog.md`

## Done recently
- `engine-api` is the canonical backend
- Phase 3 backend groundwork is in place:
  - notes API
  - contacts API
  - offer domain/table
  - PostgreSQL full-text search
  - search pagination
  - `skills_updated_at` for profile skill freshness
- `web` is wired for:
  - add note
  - create contact
  - link contact to application
  - create/update offer from application detail
  - edit application status from application detail
  - set or clear application due date from application detail
- Phase 8 DB-backed verification now exists via:
  - local Postgres in Docker
  - `scripts/verify_phase8_db.sh`
  - real smoke coverage for notes, contacts, offers, search, status, and due date
- first ingestion compatibility slice now exists via:
  - `apps/ingestion`
  - canonical job upsert into Postgres
  - source-specific raw snapshot persistence into `job_variants`
  - DB-managed `search_vector` compatibility verified through `engine-api` search
  - first `mock-source` adapter-backed normalization path
- shared contracts were aligned with the current Phase 3 flow

## Current focus
See `docs/05-roadmap/current-focus.md` for the active near-term slice and verification checklist.

## After current focus

### 1. Harden search and ranking
- validate search quality with seeded DB data:
  - title match
  - company match
  - description match
  - punctuation-heavy queries
- confirm ranking still behaves correctly when:
  - `skills_updated_at` is absent
  - `job.posted_at` is absent
  - profile analysis is stale

### 2. Prepare next architecture slice
- `ingestion`: extend beyond the first adapter-backed normalization path into more source adapters, dedupe, and refresh rules over `job_variants`
- `ml`: add read-only integration around canonical engine data
- docs: update architecture docs for the stabilized post-Phase-8 boundary

## Default rules
- keep `engine-api` as the only write authority
- do not introduce `offer_received` into `application.status`
- keep one offer aggregate per application for now
- prefer small, incremental changes
