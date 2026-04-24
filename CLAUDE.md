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

Active runtime paths:
- `apps/web/src/App.tsx` -> `apps/web/src/AppShell.tsx` is the real shell entrypoint.
- `apps/web/src/AppShellNew.tsx` is only a compatibility re-export alias, not the source of truth.

## Architecture rules
- Domain truth lives in Rust, not in UI and not in LLM prompts.
- LLM is an enrichment layer, not the source of canonical truth.
- Internal role identity must use a canonical role model (`RoleId` / role catalog), not free-form strings.
- Search filters such as region/work mode/source must stay structured, not be flattened into free text.
- Prefer explicit DTOs, small services, testable helpers, and stable contracts.

## Immediate priorities
1. ~~Canonical role catalog in domain.~~ — DONE (`domain/role/catalog.rs`)
2. Search profile persistence — persisted `search_preferences` are now stored on profiles; structured search profiles still build on demand.
3. ~~Source filtering for parsed jobs.~~ — DONE (`domain/source/catalog.rs`, `services/matching/filters.rs`)
4. ~~Fix web navigation stale-data behavior.~~ — DONE (TanStack Query invalidation in place)
5. ~~Ranking v2: deterministic baseline + explainable fit.~~ — DONE (scoring.rs, salary.rs, reranking.rs)
6. ~~Lists and controls: saved, hidden, bad, whitelist, blacklist.~~ — DONE (FeedbackCenter + ApplicationBoard)
7. ~~ML training pipeline quality.~~ — DONE (temporal decay, temporal split, sample weights by signal quality, BPR pairwise training, feature importances, auto-retrain at 15 examples/6h, model health dashboard, data drift detection all shipped; `ARTIFACT_VERSION=trained_reranker_v3`). Next: LightGBM backend (`lgbm_model.py`) when ≥50 labeled examples.
8. Market Intelligence is partially live — overview, companies, salary trends, and role demand ship from live `jobs` queries; snapshot aggregation is still missing.
9. ~~Profile compensation and languages.~~ — DONE end-to-end in schema, API, persistence, and UI.
10. ~~Notifications and global search.~~ — DONE in web + engine-api.
11. ~~Structured logging + X-Request-ID correlation.~~ — DONE (JSON logs, tower-http request ID propagation, ML middleware, log correlation across engine-api → ML).
12. ~~Silent failure fixes + error enrichment.~~ — DONE (feature_stats.py, api.py ready-check, task_store.py, engine-api error logging).
13. ~~PostgreSQL optimization.~~ — DONE (performance indexes migration, pool tuning min/acquire/idle, slow-query logging in docker-compose).
14. ~~Observability stack.~~ — DONE (Prometheus + Grafana in docker-compose, `/metrics` on engine-api and ML, pre-built dashboard).
15. ~~ML legacy removal.~~ — DONE (inspect.signature() shim removed, bootstrap workflow interface standardized, all tests updated).
16. ~~Applications page redesign.~~ — DONE (table+panel layout with inline status change, quick-panel slide-in, search/filter header).
17. ~~Internal ID cleanup.~~ — DONE (`application_id` removed from nested DTOs, `source_job_id` removed from job variant responses, no UUIDs rendered to users).

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
- Do not render raw UUIDs to users — IDs may appear in URL paths and `key` props but never in visible text.
- Observability: all new services must emit structured JSON logs and accept/propagate `X-Request-ID`.

## Preferred working style
- Make one vertical slice at a time.
- Update docs when the slice changes architecture or contracts.
- Add tests for all new matching/ranking/merge logic.
- Keep comments short and useful.
- Prefer maintainability over cleverness.

## Good next slices
- CV tailoring endpoint in ML sidecar + web modal
- Settings expansion (dedicated notification prefs + more profile preference controls)
- Rust-side validation of LLM enrichment output before storing
- LightGBM backend (`lgbm_model.py`) when ≥50 labeled examples
- Market Intelligence snapshot aggregation (complete the partially-live market page)
- `BootstrapContext` dataclass threading through bootstrap_workflow for full request-id correlation in ML logs
