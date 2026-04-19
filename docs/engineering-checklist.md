# Engineering Checklist

## Before push
- Run `pnpm typecheck`
- Run `pnpm build`
- Run `pnpm --dir apps/web test`
- Run `cargo check --manifest-path apps/engine-api/Cargo.toml`
- Run `cargo check --manifest-path apps/ingestion/Cargo.toml`

## For architecture changes
- Prefer adding or moving code inside an existing bounded context first
- Do not add new transport types to a god file if a domain module already exists
- Keep `apps/web/src/api.ts` as a compatibility facade, not a logic file
- Keep ML route wiring separate from enrichment/scoring implementations
- Keep Rust as source of truth for domain state
- Treat LLM outputs as enrichment, not canonical state

## For API work
- Preserve public function signatures unless intentionally changing contracts
- Prefer type-only imports across `engine-types/*`
- Avoid reintroducing cross-domain helpers in facade files

## For ML work
- Route files should register routes, not own reusable business logic
- Provider/factory/dependency wiring should stay separate
- Add new enrichment features as bounded modules, not inline in `api.py`