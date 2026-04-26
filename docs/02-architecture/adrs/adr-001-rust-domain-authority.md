# ADR-001: Rust engine-api owns canonical domain logic

## Status

Accepted

## Context

Job Copilot has three backend services: a Rust engine-api, a Rust ingestion service,
and a Python ML sidecar. Early development could have distributed domain logic across
these services or placed it in the web layer. This creates ambiguity about where
authoritative state lives, makes validation inconsistent, and makes it harder to
reason about correctness.

The system needs one clear source of truth for: profiles, jobs, applications, feedback,
ranking rules, lifecycle interpretation, ownership checks, and stable API contracts.

## Decision

`apps/engine-api` is the canonical domain authority for all user-facing domain state.

- **Profiles** — create, read, update, analyze, ownership.
- **Jobs** — canonical job records, lifecycle state, variants, presentation layer.
- **Applications** — Kanban state, application detail, notes, contacts, offers.
- **Feedback** — save, hide, bad fit, whitelist, blacklist, company signals.
- **Ranking rules** — deterministic scoring, behavior scoring, trained reranker integration.
- **Lifecycle interpretation** — active/inactive/reactivated semantics.
- **API contracts** — stable, typed DTOs. No undocumented fields invented downstream.

`apps/ingestion` supplies normalized job data to PostgreSQL. It does not own user-facing
domain decisions, ranking, or lifecycle interpretation beyond the ingest lifecycle fields
it writes.

`apps/ml` is an enrichment sidecar. It can produce fit scores, rerank suggestions, and
enrichment output, but this output does not become canonical domain state without
validation or persistence through engine-api.

`apps/web` is an operator UI. It displays, filters, and triggers actions. It must not
invent scoring logic, ownership rules, or profile matching rules client-side.

## Consequences

**Easier:**
- A single place to audit domain correctness, security guards, and API contracts.
- ML output failures or provider changes do not corrupt domain state.
- Web UI changes do not accidentally shift domain authority.
- Tests for domain logic have one home.

**Harder:**
- New domain features always require a Rust implementation in engine-api first.
- ML enrichment cannot shortcut the engine-api contract to write state directly.
- Ingestion must remain disciplined about staying within its supply/normalization role.

**Constraints created:**
- Do not add scoring, ownership checks, or lifecycle rules to `apps/web` or `apps/ml`.
- Do not invent DTO fields, endpoints, or DB columns without first checking engine-api.
- Profile-scoped routes must enforce ownership via engine-api middleware.

## Current State

**Implemented:**
- Engine-api owns profiles, jobs, applications, feedback, ranking, presentation layer,
  analytics, notifications, and market endpoints.
- Ingestion writes normalized jobs directly to PostgreSQL and does not call engine-api.
- ML sidecar enrichment is consumed by engine-api and treated as non-canonical input.
- Web reads and writes exclusively through engine-api HTTP routes.

**Partial / gaps:**
- Profile ownership guards exist in engine-api middleware but are not yet applied to all
  profile-scoped route handlers. See [security-model.md](../security-model.md).
- Market route handlers still query the live `jobs` table directly rather than
  reading from `market_snapshots`. See [ADR-006](adr-006-market-snapshots-read-path.md).

## Related Docs

- [current-state.md](../current-state.md)
- [security-model.md](../security-model.md)
- [service-communication.md](../service-communication.md)
- [ADR-002: ML enrichment sidecar](adr-002-ml-enrichment-sidecar.md)
