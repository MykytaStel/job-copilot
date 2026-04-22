# PostgreSQL Extensions — Self-Hosted PG16

Updated: 2026-04-22

This repo currently runs on self-hosted PostgreSQL 16. The recommendations below are intentionally narrow: install only what has a clear query, diagnostics, or maintenance use for the current stack.

## Install Now

- `pg_stat_statements`
  - baseline query visibility for `engine-api` and `ingestion`
  - required for meaningful slow-query review
  - requires `shared_preload_libraries`
- `pg_trgm`
  - useful for fuzzy text search, title/company matching, and future search improvements
- `unaccent`
  - useful for normalization and accent-insensitive search

## Install When Needed

- `auto_explain`
  - capture slow query plans during investigation
  - requires `shared_preload_libraries`
- `amcheck`
  - index integrity checks
- `pgstattuple`
  - inspect table and index bloat
- `pg_visibility`
  - investigate visibility map and vacuum issues
- `pg_cron`
  - only if recurring jobs move into PostgreSQL itself
  - likely candidate if `market_snapshots` aggregation becomes DB-scheduled
- `HypoPG`
  - only for index what-if analysis during tuning
- `pg_repack`
  - only for low-downtime bloat remediation

## Do Not Install Yet

- `pgvector`
  - add only when there is a concrete embeddings or vector-search path
- broad extension bundles with no measured need

## Notes

- Enable `pg_stat_statements` first before performance tuning so real query evidence exists.
- Prefer measuring with live `engine-api` and `ingestion` workloads before adding operational complexity.
- If `market_snapshots` stays an application-side job, `pg_cron` is optional rather than required.
