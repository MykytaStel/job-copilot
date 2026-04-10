# Milestones

## Milestone 1 — Repository and migration foundation
Goal:
- clean repo
- stable docs
- new service structure
- legacy isolated

Success:
- `api-legacy` separated
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