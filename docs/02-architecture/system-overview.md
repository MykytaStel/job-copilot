# System Overview

## Services

### web
React frontend for:
- search
- job details
- saved jobs
- applications
- profile

### engine-api
Rust backend.
Owns:
- jobs API
- profile API
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
- services use Postgres
