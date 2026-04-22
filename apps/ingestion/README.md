# ingestion

Rust ingestion service for source fetch, scrape, normalize, dedupe, lifecycle, and canonical job upserts.

Current slice:
- scrapes real sources: Djinni, Work.ua, Dou.ua, Robota.ua
- supports daemon mode across all configured sources
- still supports adapter-backed file demos such as `mock-source`
- upserts canonical jobs into Postgres
- persists source-specific raw snapshots into `job_variants`
- resolves adapter-backed canonical job ids through `job_variants` using a normalized dedupe fingerprint
- reports whether source variants were created, updated, or unchanged on repeat runs
- applies engine-api migrations when run standalone
- refreshes `market_snapshots` after successful ingestion writes
- does not write `search_vector`
- relies on the database trigger in `engine-api` migrations to maintain search indexing

## Runtime

Environment variables:
- `DATABASE_URL` required Postgres connection string
- `INGESTION_DAEMON_INTERVAL_MINUTES` default `60`
- `INGESTION_MAX_PAGES_PER_SOURCE` default `3`

## Input format

The binary accepts normalized input as either:
- a JSON array of jobs
- or an object with `{ "jobs": [...] }`

It also accepts one adapter-backed source format:
- `--input-format mock-source`
- payload shape: `{ "fetched_at": "...", "jobs": [...] }`
- source fields are normalized into canonical job fields before upsert
- the adapter also stores the latest source payload in `job_variants`
- adapter-backed canonical ids are no longer tied to `source_job_id`; ingestion first keeps existing variant mappings and otherwise groups new variants by `dedupe_key`

Each job uses canonical fields:
- `id`
- `title`
- `company_name`
- `location`
- `remote_type`
- `seniority`
- `description_text`
- `salary_min`
- `salary_max`
- `salary_currency`
- `posted_at`
- `last_seen_at`
- `is_active`

## Run

```bash
cd apps/ingestion
DATABASE_URL=postgres://jobcopilot:jobcopilot@127.0.0.1:5432/jobcopilot \
  cargo run -- --input examples/sample_jobs.json
```

Run the adapter-backed demo:

```bash
cd apps/ingestion
DATABASE_URL=postgres://jobcopilot:jobcopilot@127.0.0.1:5432/jobcopilot \
  cargo run -- --input examples/mock_source_jobs.json --input-format mock-source
```

Run the updated payload demo for the same source job:

```bash
cd apps/ingestion
DATABASE_URL=postgres://jobcopilot:jobcopilot@127.0.0.1:5432/jobcopilot \
  cargo run -- --input examples/mock_source_jobs_updated.json --input-format mock-source
```

Run the full local stack from repo root:

```bash
pnpm local:ingestion:demo
```

Run real scrapers once:

```bash
cd apps/ingestion
DATABASE_URL=postgres://jobcopilot:jobcopilot@127.0.0.1:5432/jobcopilot \
  cargo run -- scrape
```

Run daemon mode:

```bash
cd apps/ingestion
DATABASE_URL=postgres://jobcopilot:jobcopilot@127.0.0.1:5432/jobcopilot \
  cargo run -- daemon
```

The local demo:
- starts Postgres in Docker
- starts `engine-api` with migrations
- runs the first mock ingestion pass
- runs an updated second pass for the same `source_job_id`
- runs the updated payload again to show the `unchanged` outcome
- prints ready URLs and `job_variants` rows

## Rule

- `ingestion` may upsert canonical jobs
- `ingestion` may persist source-specific variants / raw snapshots
- `ingestion` must not manage `search_vector` directly
- `engine-api` remains the canonical backend for product-facing writes
