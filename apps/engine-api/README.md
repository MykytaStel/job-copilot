# engine-api

Rust backend for the frontend-facing API.

## Runtime

Environment variables:
- `PORT` default `8080`
- `DATABASE_URL` optional Postgres connection string
- `DATABASE_MAX_CONNECTIONS` default `5`
- `RUN_DB_MIGRATIONS` default `true`

Behavior:
- if `DATABASE_URL` is not set, `engine-api` still starts and reports database status as `disabled`
- if `DATABASE_URL` is set, `engine-api` opens a Postgres pool on startup
- if `RUN_DB_MIGRATIONS=true`, embedded SQL migrations from `migrations/` are applied on startup

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
    "raw_text": "Senior React Native developer with TypeScript, GraphQL, and product team experience."
  }'
```

Analyze a persisted profile:

```bash
curl \
  -X POST http://localhost:8080/api/v1/profiles/$PROFILE_ID/analyze
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
      "preferred_roles": ["frontend_developer"],
      "include_keywords": ["product company"],
      "exclude_keywords": ["gambling"]
    }
  }'
```

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
    "status": "saved"
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
- starts local Postgres via `infra/docker-compose.postgres.yml`
- runs `engine-api` with migrations enabled
- seeds demo data
- verifies `search_vector` backfill and trigger updates directly in Postgres
- exercises the latest Phase 3 write/read APIs against a real database

Useful flags:
- `KEEP_POSTGRES=1 pnpm verify:phase8:db` keeps the Postgres container running after verification
- by default the verification script uses `PORT=18080` to avoid collisions with local dev servers
- `PORT=18081 pnpm verify:phase8:db` runs `engine-api` on a different local port

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
