# Project Status

Date: 2026-04-14

## What is already done

### Repository / architecture
- monorepo structure is in place
- `apps/web` is the active frontend
- `apps/engine-api` is the canonical backend
- `packages/contracts` exists for shared TypeScript contracts
- docs structure is already split by product / architecture / domain / roadmap

### `engine-api`
- health endpoint exists
- jobs read APIs exist
- applications read/write basics exist
- profile create / update / analyze exists
- resume upload / activate / list exists
- match / fit flows exist
- application notes write API exists:
  - `POST /api/v1/applications/{id}/notes`
- contacts API exists:
  - `GET /api/v1/contacts`
  - `POST /api/v1/contacts`
  - `POST /api/v1/applications/{id}/contacts`
- offer API exists:
  - `PUT /api/v1/applications/{id}/offer`
- application patching already supports:
  - `status`
  - `due_date`
- PostgreSQL full-text search exists for jobs
- search pagination exists
- profile skill freshness exists via `skills_updated_at`
- unit / route tests are in place for the new Phase 3 backend flows

### `web`
- dashboard works against `engine-api`
- job details works against `engine-api`
- application board read flow works
- application detail read flow works
- profile CRUD + analyze flow works
- application detail write flow now supports:
  - add note
  - create contact
  - link contact
  - create/update offer
  - edit status
  - set / clear due date

### shared contracts
- `packages/contracts` is present
- Phase 3 contracts were aligned for:
  - paginated search
  - optional application offer
  - profile `skillsUpdatedAt`
  - contact relationships

## What is partially done

### application workflow
- application detail is already the main read/write page
- notes / contacts / offer / status / due date are wired
- real Postgres verification for the full write surface now exists via the local Phase 8 smoke flow

### search and ranking
- backend search implementation exists
- pagination exists
- ranking logic exists
- Phase 3 behavior is covered by unit tests
- real Postgres verification now exists via the local Phase 8 smoke flow

### docs / roadmap
- near-term roadmap exists
- future work exists
- project-wide status was still unclear before this document

## What is not done yet

### `ingestion`
- service folder exists
- first Rust ingestion binary now exists
- canonical job upsert into Postgres now exists
- first source-adapter and normalization foundation now exists
- raw snapshot persistence now exists via `job_variants`
- repeat source ingestion now has basic change detection via variant `created` / `updated` / `unchanged` outcomes
- ingestion is verified to stay compatible with DB-managed `search_vector`
- production source adapters / normalization / dedupe pipeline are not implemented yet

### `ml`
- service folder exists
- minimal Python skeleton exists
- no real extraction / reranking integration in product flow yet

### full system integration
- a verified real-DB smoke run now exists for the latest Phase 3 changes
- no end-to-end staging flow for the newest contacts / offer / search changes yet
- architecture handoff for ingestion + ML on top of stabilized contracts is still ahead

## What to do next

### Next step now
See `docs/05-roadmap/current-focus.md` for the active near-term slice.

### After that
- update architecture docs after DB verification
- move to ingestion compatibility and ML read-only integration

## Short summary
- `engine-api` and `web` are already usable for the core MVP-style flows
- Phase 3 is mostly implemented in code
- Phase 3 is now verified against a real local Postgres flow
- the first ingestion compatibility slice is now in place, including canonical jobs plus raw source variants
- the bigger missing pieces for the project are still `ingestion` and `ml`
