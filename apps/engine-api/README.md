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
docker run --name job-copilot-postgres \
  -e POSTGRES_USER=jobcopilot \
  -e POSTGRES_PASSWORD=jobcopilot \
  -e POSTGRES_DB=jobcopilot \
  -p 5432:5432 \
  -d postgres:16
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
