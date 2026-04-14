# CLAUDE.md

You are working in the Job Copilot monorepo.

## Product intent
Job Copilot is not a simple job board. It is a candidate intelligence and action system:
1. understand the user
2. understand what they want now
3. ingest and normalize jobs
4. rank jobs for that user
5. explain fit/gaps
6. support actions: save, hide, apply, follow up, tailor CV, learn from outcomes

## Current repo shape
- `engine-api/` — canonical backend and domain authority
- `ingestion/` — source fetch, scrape, normalize, dedupe, lifecycle
- `ml/` — Python enrichment sidecar, reranking, analytics, future LLM orchestration
- `web/` — operator UI / dashboard

## Architecture rules
- Domain truth lives in Rust, not in UI and not in LLM prompts.
- LLM is an enrichment layer, not the source of canonical truth.
- Internal role identity must use a canonical role model (`RoleId` / role catalog), not free-form strings.
- Search filters such as region/work mode/source must stay structured, not be flattened into free text.
- Prefer explicit DTOs, small services, testable helpers, and stable contracts.

## Immediate priorities
1. Canonical role catalog in domain.
2. Search preferences + search profile.
3. Source filtering for parsed jobs (which sources to ingest/search from).
4. Fix web navigation stale-data behavior so pages refresh correctly on transitions.
5. Ranking v2: deterministic baseline + explainable fit.
6. Lists and controls: saved, hidden, bad, whitelist, blacklist.
7. ML/LLM enrichment with strict validation back in Rust.

## UX direction
Quiet operator dashboard:
- dark base
- restrained gradients
- dense but readable cards
- low-noise interaction
- fit/explanation first
- act quickly from any list

## Guardrails
- Do not invent new role IDs outside the canonical catalog.
- Do not bypass DTO validation.
- Do not move domain truth into the frontend.
- Do not make LLM output authoritative without Rust-side validation.
- Do not add broad abstractions unless a real second use-case exists.

## Preferred working style
- Make one vertical slice at a time.
- Update docs when the slice changes architecture or contracts.
- Add tests for all new matching/ranking/merge logic.
- Keep comments short and useful.
- Prefer maintainability over cleverness.

## Good next slices
- canonical role catalog
- search-profile/build
- source filters for search + ingestion
- web query invalidation / route refresh fixes
- company list statuses and job list labels
- analytics endpoints for timeline / charts
