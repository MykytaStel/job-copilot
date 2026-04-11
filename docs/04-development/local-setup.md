# Local Setup

## Goal
Run the project locally with minimal friction around the canonical service set.

## Services
Current and planned services:

- `apps/web` — React frontend
- `apps/engine-api` — Rust core backend
- `apps/ingestion` — new Rust ingestion workers
- `apps/ml` — new Python ML/LLM service

## Required tools

### Node.js
Used for:
- web app
- package scripts

Check:
```bash
node --version
npm --version
pnpm --version
