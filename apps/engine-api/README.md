# engine-api

Rust backend for the frontend-facing API.

## Architecture

- Domain truth lives in Rust: all canonical identifiers, scoring, and role catalog live here.
- ML sidecar (`apps/ml`) is an enrichment layer called over HTTP — its output is validated and stored by engine-api, not treated as authoritative on its own.
- All state is in Postgres. engine-api opens a connection pool on startup and applies embedded SQL migrations.

## Runtime Configuration

| Variable | Default | Description |
|---|---|---|
| `PORT` | `8080` | HTTP listen port |
| `DATABASE_URL` | — | Postgres connection string. If unset, API starts but DB endpoints return 503 |
| `DATABASE_MAX_CONNECTIONS` | `5` | Max pool connections (docker-compose sets `20`) |
| `RUN_DB_MIGRATIONS` | `true` | Apply embedded migrations from `migrations/` on startup |
| `LEARNED_RERANKER_ENABLED` | `true` | Enable heuristic reranker |
| `TRAINED_RERANKER_ENABLED` | `false` | Enable ML-trained reranker (requires `TRAINED_RERANKER_MODEL_PATH`) |
| `TRAINED_RERANKER_MODEL_PATH` | — | Path to trained reranker JSON artifact |
| `ML_SIDECAR_BASE_URL` | — | Base URL for the ML sidecar (e.g. `http://ml:8000`) |
| `RUST_LOG` | `info` | Tracing filter — accepts `debug`, `info`, `warn`, `error` or module-specific filters like `engine_api=debug` |

Behavior:
- if `DATABASE_URL` is not set, `engine-api` still starts and reports database status as `disabled`
- if `DATABASE_URL` is set, `engine-api` opens a Postgres pool on startup with `min_connections=2`, `acquire_timeout=5s`, `idle_timeout=600s`
- if `RUN_DB_MIGRATIONS=true`, embedded SQL migrations from `migrations/` are applied on startup
- for the local Docker stack, `infra/docker-compose.yml` sets `DATABASE_MAX_CONNECTIONS=20`

## Logging & Tracing

Logs are emitted as structured JSON to stdout. Every request receives a UUID v7 `x-request-id` header. The same ID is forwarded to the ML sidecar and included in all log fields for correlation.

```bash
# Stream and pretty-print logs
docker compose logs -f engine-api | jq .
```

Key log fields:

| Field | Description |
|---|---|
| `request_id` | UUID v7, unique per HTTP request |
| `method` / `uri` | HTTP method and path |
| `status` | HTTP response status code |
| `latency_ms` | Request duration in milliseconds |

Adjust verbosity:
```bash
RUST_LOG=engine_api=debug cargo run
```

## Local Setup

Start Postgres:

```bash
pnpm db:up
```

Set the connection string:

```bash
export DATABASE_URL=postgres://jobcopilot:jobcopilot@localhost:5432/jobcopilot
```

Start the API:

```bash
cd apps/engine-api
DATABASE_URL=$DATABASE_URL RUN_DB_MIGRATIONS=true cargo run
```

Optional demo data:

```bash
psql "$DATABASE_URL" -f seeds/dev.sql
```

Stop Postgres and remove the local verification volume:

```bash
pnpm db:down
```

## Canonical API Flows

Health:

```bash
curl http://localhost:8080/health
```

Create a profile:

```bash
curl \
  -X POST http://localhost:8080/api/v1/profiles \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Jane Doe",
    "email": "jane@example.com",
    "location": "Kyiv",
    "raw_text": "Senior React Native developer with TypeScript, REST APIs, and product team experience."
  }'
```

Fetch a profile:

```bash
export PROFILE_ID=<profile-id>
curl http://localhost:8080/api/v1/profiles/$PROFILE_ID
```

Update a profile:

```bash
curl \
  -X PATCH http://localhost:8080/api/v1/profiles/$PROFILE_ID \
  -H "Content-Type: application/json" \
  -d '{
    "location": "Lviv",
    "raw_text": "Senior React Native developer with TypeScript, GraphQL, and product team experience.",
    "search_preferences": {
      "target_regions": ["ua", "eu_remote"],
      "work_modes": ["remote"],
      "preferred_roles": ["frontend_engineer"],
      "allowed_sources": ["djinni", "work_ua"],
      "include_keywords": ["product company"],
      "exclude_keywords": ["gambling"]
    }
  }'
```

Analyze a persisted profile:

```bash
curl \
  -X POST http://localhost:8080/api/v1/profiles/$PROFILE_ID/analyze
```

Build a search profile directly from raw text:

```bash
curl \
  -X POST http://localhost:8080/api/v1/search-profile/build \
  -H "Content-Type: application/json" \
  -d '{
    "raw_text": "Senior React Native developer with product experience",
    "preferences": {
      "target_regions": ["ua", "eu_remote"],
      "work_modes": ["remote"],
      "preferred_roles": ["frontend_developer"],
      "allowed_sources": ["djinni", "work_ua"],
      "include_keywords": ["product company"],
      "exclude_keywords": ["gambling"]
    }
  }'
```

Build a search profile from a persisted profile:

```bash
curl \
  -X POST http://localhost:8080/api/v1/profiles/$PROFILE_ID/search-profile/build \
  -H "Content-Type: application/json" \
  -d '{
    "preferences": {
      "target_regions": ["ua", "eu_remote"],
      "work_modes": ["remote"],
      "preferred_roles": ["frontend_engineer"],
      "allowed_sources": ["djinni", "work_ua"],
      "include_keywords": ["product company"],
      "exclude_keywords": ["gambling"]
    }
  }'
```

If the profile already has stored `search_preferences`, the same endpoint also accepts an empty
`preferences` object and rebuilds from the persisted values.

Other read APIs:

```bash
curl http://localhost:8080/api/v1/jobs/recent
curl http://localhost:8080/api/v1/applications/recent
curl http://localhost:8080/api/v1/roles
```

`GET /api/v1/jobs/recent` now returns:
- canonical job fields
- lifecycle fields: `first_seen_at`, `last_seen_at`, `inactivated_at`, `reactivated_at`
- source metadata for the most recent variant
- feed summary counts for `active`, `inactive`, and `reactivated` jobs

Local ingestion lifecycle demo:

```bash
pnpm local:ingestion:demo
```

The demo runs three ingestion passes:
1. initial source snapshot
2. refresh where one job disappears and becomes inactive
3. refresh where the job reappears and becomes reactivated

Create and update an application:

```bash
curl \
  -X POST http://localhost:8080/api/v1/applications \
  -H "Content-Type: application/json" \
  -d '{
    "job_id": "job_backend_rust_001",
    "status": "saved",
    "profile_id": "'$PROFILE_ID'"
  }'

export APPLICATION_ID=<application-id>

curl \
  -X PATCH http://localhost:8080/api/v1/applications/$APPLICATION_ID \
  -H "Content-Type: application/json" \
  -d '{
    "status": "applied",
    "due_date": "2026-04-20T12:00:00Z"
  }'

curl http://localhost:8080/api/v1/applications/$APPLICATION_ID
```

`profile_id` is optional for application persistence. When provided, application creation also
emits a profile-scoped `application_created` user event, which makes the job eligible for
`GET /api/v1/profiles/:id/reranker-dataset` as a positive training example.

Resume and match flows:

```bash
curl \
  -X POST http://localhost:8080/api/v1/resume/upload \
  -H "Content-Type: application/json" \
  -d '{
    "filename": "resume_backend.txt",
    "raw_text": "Senior backend engineer with Rust, Postgres, and API design experience."
  }'

curl http://localhost:8080/api/v1/resumes
curl http://localhost:8080/api/v1/resumes/active
curl -X POST http://localhost:8080/api/v1/jobs/job_backend_rust_001/match
curl http://localhost:8080/api/v1/jobs/job_backend_rust_001/match
```

Search:

```bash
curl "http://localhost:8080/api/v1/search?q=rust"
```

## Phase 8 DB Verification

Automated smoke verification for:
- notes
- contacts
- offers
- search
- application `status`
- application `due_date`
- `search_vector` backfill and trigger behavior

Run it from repo root:

```bash
pnpm verify:phase8:db
```

What the script does:
- starts an isolated local Postgres container for the verification run
- runs `engine-api` with migrations enabled
- seeds demo data
- verifies `search_vector` backfill and trigger updates directly in Postgres
- exercises the latest Phase 3 write/read APIs against a real database

Useful flags:
- `KEEP_POSTGRES=1 pnpm verify:phase8:db` keeps the Postgres container running after verification
- by default the verification script uses `PGPORT=15432` to avoid collisions with the local app stack
- `PGPORT=15433 pnpm verify:phase8:db` runs the verification Postgres on a different local port
- by default the verification script uses `PORT=18080` to avoid collisions with local dev servers
- `PORT=18081 pnpm verify:phase8:db` runs `engine-api` on a different local port

## Database Diagnostics

Enable slow query logging by setting `log_min_duration_statement=200` on Postgres (already set in `infra/docker-compose.yml`). To analyze query performance:

```sql
-- Top slow queries (requires pg_stat_statements extension)
SELECT query, calls, total_exec_time/calls AS avg_ms, rows
FROM pg_stat_statements ORDER BY avg_ms DESC LIMIT 20;

-- Missing index candidates for key tables
SELECT schemaname, tablename, attname, n_distinct, correlation
FROM pg_stats WHERE tablename IN ('jobs','applications','job_feedback')
ORDER BY n_distinct DESC;
```

Connect to the local Postgres:
```bash
psql postgres://jobcopilot:jobcopilot@localhost:5432/jobcopilot
```

## Error Contract

Validation errors return `400`.
Missing resources return `404`.
Missing database connectivity returns `503`.

Example:

```json
{
  "code": "invalid_profile_input",
  "message": "Field 'email' must contain '@'",
  "details": {
    "field": "email"
  }
}
```
