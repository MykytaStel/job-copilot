# Milestones

See also:
- `docs/05-roadmap/current-focus.md`
- `docs/05-roadmap/project-status.md`
- `docs/05-roadmap/future-work.md`

## Current milestone state
- Milestones 1-7: mostly in place at repository / API / frontend level
- Milestone 8: complete
- Milestone 9: current active milestone

## Milestone 1 — Repository and backend foundation
Goal:
- clean repo
- stable docs
- new service structure
- backend foundation ready

Success:
- `engine-api` created
- `ingestion` created
- `ml` created
- docs split into small files

## Milestone 2 — Core domain and API skeleton
Goal:
- define domain entities
- create Rust API skeleton
- connect Postgres
- expose basic endpoints

Success:
- health endpoint
- jobs list endpoint
- job details endpoint
- application CRUD basics

## Milestone 3 — First ingestion flow
Goal:
- define source adapter structure
- ingest one source
- store raw snapshots
- normalize into jobs

Success:
- one source works end-to-end
- raw and normalized layers are separated

## Milestone 4 — Search and ranking v1
Goal:
- basic search
- filters
- ranking formula v1
- user profile influence

Success:
- user sees ranked jobs
- ranking is explainable

## Milestone 5 — Core intelligence layer
Goal:
- title normalization
- seniority inference
- remote type inference
- fit score v1

Success:
- search quality improves over plain keyword matching

## Milestone 6 — ML sidecar integration
Goal:
- Python service online
- one extraction route
- one reranking route
- optional explanation generation

Success:
- ML enriches the core instead of replacing it

## Milestone 7 — Stable staging demo
Goal:
- web integrated with new APIs
- basic tests
- staging environment works

Success:
- demo flow works end-to-end

## Milestone 8 — Phase 3 stabilization
Goal:
- verify Phase 3 on real Postgres
- lock shared contracts
- complete application detail write workflow

Success:
- contacts, offers, and search are DB-verified
- `web` and `engine-api` use the same Phase 3 contracts
- application detail supports the main write actions

## Milestone 9 — Ingestion and ML handoff
Goal:
- keep ingestion compatible with DB-managed search indexing
- define read-only ML integration over canonical engine data
- refresh architecture docs after stabilization

Success:
- ingestion does not break `search_vector`
- ML integration does not own core writes
- future service boundaries are documented

Blocked by:
- no hard blocker at the repo level

Progress so far:
- first ingestion-safe canonical job upsert path exists
- local DB verification confirmed `engine-api` search sees ingestion-upserted jobs through DB-managed `search_vector`
- first adapter-backed normalization path now exists in `apps/ingestion`
- raw snapshots now persist in `job_variants` alongside canonical job upserts
- first repeated-ingestion refresh semantics now exist at the `job_variants` layer
