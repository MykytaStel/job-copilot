# Backlog

See also:
- `docs/05-roadmap/current-focus.md`
- `docs/05-roadmap/project-status.md`
- `docs/05-roadmap/future-work.md`

## Current focus
See `docs/05-roadmap/current-focus.md` for the active near-term slice.

## Core architecture
- [ ] isolate legacy backend
- [x] create Rust engine-api
- [x] create Rust ingestion service
- [x] create Python ML service
- [x] introduce Postgres
- [x] define contracts package

## Domain
- [ ] finalize entities
- [ ] define ranking input/output models
- [ ] define change event models
- [x] define source adapter interfaces

## Ingestion
- [x] raw snapshot model
- [ ] source fetch abstraction
- [ ] parser abstraction
- [x] normalization pipeline
- [ ] dedupe pipeline
- [ ] refresh scheduler

## Search
- [x] keyword search
- [ ] filters
- [ ] ranking formula v1
- [ ] ranking explanation
- [ ] recency score
- [ ] remote/seniority fit
- [x] verify PostgreSQL full-text search on real DB data
- [x] confirm `search_vector` trigger/backfill behavior

## ML
- [ ] extraction endpoint
- [ ] fit analysis endpoint
- [ ] reranking endpoint
- [ ] structured outputs
- [ ] experiment dataset design

## Frontend
- [ ] search page cleanup
- [x] job details page
- [ ] save job flow
- [ ] application status flow
- [ ] profile/preferences UI
- [x] application detail write flow for notes, contacts, and offer
- [x] application detail write flow for status and due date
- [ ] remove remaining legacy API assumptions after Phase 3

## Quality
- [ ] unit tests for ranking
- [ ] integration tests for jobs/applications
- [x] ingestion tests
- [ ] E2E smoke flow
- [x] DB-backed smoke tests for notes, contacts, offers, search, and application patching
