# Conventions

## Repository conventions
- `apps/` contains runnable services
- `packages/` contains shared contracts and reusable code
- `docs/` contains product, architecture, and workflow documentation
- `infra/` contains scripts, compose files, and infrastructure helpers

## Service ownership
- `web` owns presentation
- `api-legacy` owns temporary legacy behavior
- `engine-api` owns future domain APIs
- `ingestion` owns source synchronization
- `ml` owns ML/LLM-specific processing

## Domain conventions
Use canonical domain language:
- Job
- JobVariant
- SearchProfile
- Application
- RankingInput
- RankingResult
- FitScore

Avoid mixing UI names with domain names.

## Source conventions
Every job source should have:
- source name
- source-specific external id
- canonical URL if possible
- raw snapshot
- normalized output

## Change tracking conventions
When a job changes, prefer explicit event types:
- discovered
- updated
- closed
- reopened
- removed

## Status conventions
Application statuses:
- saved
- applied
- interview
- rejected
- offer

## Documentation conventions
Keep docs:
- short
- focused
- versionable
- easy to reference by path

## Agent conventions
All coding agents should:
- read only the minimum required docs
- work in small scope
- report files changed
- avoid broad refactors unless explicitly requested