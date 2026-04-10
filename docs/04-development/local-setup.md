# Local Setup

## Goal
Run the project locally with minimal friction and a clear separation between legacy and new services.

## Services
Current and planned services:

- `apps/web` — React frontend
- `apps/api-legacy` — existing Fastify/TypeScript backend
- `apps/engine-api` — new Rust core backend
- `apps/ingestion` — new Rust ingestion workers
- `apps/ml` — new Python ML/LLM service

## Required tools

### Node.js
Used for:
- web app
- legacy API
- package scripts

Check:
```bash
node --version
npm --version
pnpm --version