# System Overview

## Services

### web
React frontend for:
- search
- job details
- saved jobs
- applications
- profile

### api-legacy
Existing Fastify TypeScript backend.
Used temporarily during migration.

### engine-api
New Rust backend.
Will gradually own:
- jobs API
- applications API
- ranking
- search endpoints
- domain logic

### ingestion
New Rust service for:
- fetching jobs
- storing raw snapshots
- normalization
- deduplication

### ml
New Python service for:
- extraction
- fit analysis
- reranking
- future LLM integration

## Data
- legacy uses SQLite
- new services should use Postgres

## Migration strategy
- keep web working
- keep legacy backend alive
- build new services beside it
- move responsibilities gradually