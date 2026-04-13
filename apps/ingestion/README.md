# ingestion

Rust ingestion service foundation.

Current slice:
- reads normalized jobs from a JSON file
- supports one adapter-backed `mock-source` normalization flow
- upserts canonical jobs into Postgres
- persists source-specific raw snapshots into `job_variants`
- reports whether source variants were created, updated, or unchanged on repeat runs
- does not write `search_vector`
- relies on the database trigger in `engine-api` migrations to maintain search indexing

## Runtime

Environment variables:
- `DATABASE_URL` required Postgres connection string

## Input format

The binary accepts normalized input as either:
- a JSON array of jobs
- or an object with `{ "jobs": [...] }`

It also accepts one adapter-backed source format:
- `--input-format mock-source`
- payload shape: `{ "fetched_at": "...", "jobs": [...] }`
- source fields are normalized into canonical job fields before upsert
- the adapter also stores the latest source payload in `job_variants`

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
