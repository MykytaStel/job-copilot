# Engineering Checklist

## Before push
- Run `pnpm typecheck`
- Run `pnpm build`
- Run `pnpm --dir apps/web test`
- Run `cargo check --manifest-path apps/engine-api/Cargo.toml`
- Run `cargo check --manifest-path apps/ingestion/Cargo.toml`
- Run the app-local verification matrix for every touched app in `docs/04-development/verification-matrix.md`

## For architecture changes
- Prefer adding or moving code inside an existing bounded context first
- Do not add new transport types to a god file if a domain module already exists
- Keep `apps/web/src/api.ts` as a compatibility facade, not a logic file
- Keep ML route wiring separate from enrichment/scoring implementations
- Keep Rust as source of truth for domain state
- Treat LLM outputs as enrichment, not canonical state
- Add or update an ADR for every non-trivial slice using `docs/02-architecture/adr-template.md`
- If runtime truth changes, update `docs/05-roadmap/current-focus.md`, `docs/02-architecture/current-state.md`, and the relevant roadmap/status doc in the same slice

## For API work
- Preserve public function signatures unless intentionally changing contracts
- Prefer type-only imports across `engine-types/*`
- Avoid reintroducing cross-domain helpers in facade files

## For ML work
- Route files should register routes, not own reusable business logic
- Provider/factory/dependency wiring should stay separate
- Add new enrichment features as bounded modules, not inline in `api.py`
