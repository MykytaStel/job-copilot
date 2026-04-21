# Folder Responsibilities

This document defines the intended module boundaries for the current monorepo.

The goal is to keep runtime code predictable:
- pages compose features instead of owning business logic
- shared contracts stay stable and explicit
- transport types do not leak outside API layers
- test/demo assets stay isolated from runtime code

## apps/web

- `src/pages/*`
  Route-level composition only.
  Pages may wire layout, route params, and feature entrypoints, but should not become query/mutation god files.
- `src/features/*`
  Feature-level UI state, query orchestration, selectors, and bounded presentation.
  If a page needs multiple queries, derived view models, or multi-step mutations, the logic belongs here.
- `src/api/*`
  Transport boundary only.
  This layer talks to engine-api / ml, maps wire payloads, and returns app-facing shapes.
  Raw `engine-types/*` stay internal to this layer.
- `src/api.ts`
  Compatibility facade only.
  Do not add business logic here.
- `src/components/ui/*`
  Presentational building blocks with no domain logic.
- `src/lib/*`
  Cross-feature utilities.
  Browser storage access for profile scope must stay centralized here, not spread across pages.

## apps/engine-api

- `src/domain/*`
  Canonical domain rules and source of truth.
- `src/services/*`
  Application orchestration and reusable use-case logic.
- `src/api/routes/*`
  Endpoint wiring only.
  Route files should parse input, call services, and serialize DTOs.
- `src/api/dto/*`
  Transport mapping only.
  DTO modules should not own domain rules or persistence logic.
- `src/db/repositories/*`
  SQL, persistence reads/writes, and row mapping only.

## apps/ingestion

- `src/scrapers/*` and `src/adapters/*`
  Source parsing and normalization only.
- `src/db*`
  Persistence, lifecycle merge, and database bootstrap.
  Keep connection/migration setup separate from lifecycle reconciliation when possible.
- `tests/`, `examples/`
  Dev/test-only assets.
  Runtime ingestion code must not depend on them.

## apps/ml

- `app/api.py`
  Composition root only.
- `app/*_routes.py`
  Route registration only.
- `app/service_dependencies.py`
  Dependency wiring only.
- `app/llm_provider*`
  Provider protocols/factories and provider-specific integration.
- Prompt/template generation, response normalization, and capability-specific heuristics
  should stay split by bounded concern rather than accumulated in one file.

## packages/contracts

- Stable app-facing contracts only.
- Do not place raw engine transport payloads here.
- If a shape is shared across features and is part of the app contract surface, prefer promoting it here instead of duplicating it inside web runtime modules.
