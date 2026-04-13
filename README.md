# Job Copilot UA Starter

Monorepo for a job search platform focused on the Ukrainian market.

## Stack

- Web: React + Vite + TypeScript
- Backend: Rust `engine-api`
- Ingestion: Rust
- ML: Python
- Contracts: shared schemas in `packages/contracts`

## Project Structure

```text
job-copilot-ua-starter/
  apps/
    engine-api/ # canonical backend API
    ingestion/  # ingestion service
    ml/         # ML/LLM service
    web/        # frontend
  packages/
    contracts/
  docs/
```

## First Run

Install workspace dependencies:

```bash
pnpm install
```

Start the frontend:

```bash
pnpm --filter web dev
```

Start the backend:

```bash
cd apps/engine-api
cargo run
```

Or start both together from the repo root:

```bash
pnpm dev
```

Default URLs:
- Web: `http://localhost:5173`
- Engine API: `http://localhost:8080`

## Engine API

`engine-api` is the backend contract for frontend integration.

Useful endpoints:
- `GET /health`
- `GET /api/v1/jobs/recent`
- `GET /api/v1/applications/recent`
- `GET /api/v1/roles`
- `POST /api/v1/profiles`
- `GET /api/v1/profiles/:id`
- `PATCH /api/v1/profiles/:id`
- `POST /api/v1/profiles/:id/analyze`
- `POST /api/v1/profiles/:id/search-profile/build`

For local Postgres setup and request examples, see [apps/engine-api/README.md](/Users/mykyta/Documents/projects/job-copilot-ua-starter/apps/engine-api/README.md).

## Current Frontend Scope

The current `web` app is wired to `engine-api` for:
- dashboard job lifecycle demo
- job details
- application board read view
- persisted profile CRUD + analysis

Legacy-only screens have been removed from the active router so the repository can move fully onto `engine-api`.
