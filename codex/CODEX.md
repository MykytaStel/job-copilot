# CODEX.md — Job Copilot Implementation Rules

You are implementing bounded slices in the Job Copilot monorepo.

Your job is to make small, correct, verifiable changes. Do not redesign the project unless the prompt explicitly asks for architecture work.

## First steps for every task

1. Read `CLAUDE.md`.
2. Read `AGENTS.md`.
3. Read `docs/00-master-plan.md`.
4. Read `docs/05-roadmap/current-focus.md`.
5. Read `docs/engineering-checklist.md`.
6. Inspect existing code and tests before editing.

Do not assume fields, endpoints, env vars, migrations, or DTO names. Search first.

## Implementation discipline

- Make one vertical slice at a time.
- Touch only files required for the slice.
- Match existing style.
- Keep route handlers thin.
- Keep domain rules in Rust `engine-api`.
- Keep ML as enrichment, not canonical authority.
- Keep frontend API transport typed and domain-specific.
- Prefer tests first for bug fixes and validation logic.
- Do not add broad abstractions for a single use case.
- Do not delete unrelated dead code.

## When changing `apps/engine-api`

Check for relevant routes, services, DTOs, migrations, and tests.

Rules:

- Domain truth belongs here.
- New DB state requires migrations.
- New ranking/matching/feedback behavior requires tests.
- New public/debug response fields must be safe and stable.
- Do not leak raw loader errors, paths, secrets, or internal traces.

Common commands:

```bash
cargo test --manifest-path apps/engine-api/Cargo.toml
cargo check --manifest-path apps/engine-api/Cargo.toml
pnpm verify:phase8:db
```

## When changing `apps/ingestion`

Check adapters, fixtures, lifecycle behavior, canonical job shape, and DB upserts.

Rules:

- Source-specific data must not pollute canonical fields.
- Preserve dedupe and lifecycle semantics.
- Keep source adapters isolated.
- Use fixtures for scraper/parser behavior where possible.

Common commands:

```bash
cargo test --manifest-path apps/ingestion/Cargo.toml
cargo check --manifest-path apps/ingestion/Cargo.toml
pnpm local:ingestion:demo
```

## When changing `apps/ml`

Check route wiring, DTOs, provider selection, scoring, reranker, and tests.

Rules:

- Keep FastAPI routes thin.
- Keep provider implementations separated from provider factory/wiring.
- Do not make paid APIs the default path.
- Do not train from ambiguous raw events when normalized engine exports exist.
- Keep artifacts inspectable and deterministic enough for review.

Common commands:

```bash
cd apps/ml
python -m pytest
```

## When changing `apps/web`

Check API modules, hooks, query keys, router, components, tests, and contract imports.

Rules:

- Do not move business/domain logic into components.
- Keep `src/api.ts` as compatibility facade only.
- Use domain-specific API modules under `src/api/`.
- Invalidate React Query keys after mutations.
- Do not hardcode profile/user data that should come from API.
- Do not combine visual redesign and contract changes in the same slice.

Common commands:

```bash
pnpm --dir apps/web typecheck
pnpm --dir apps/web lint
pnpm --dir apps/web test
pnpm guard:web-api-imports
pnpm build
```

## Required final response

At the end of every implementation, respond with:

```md
## What changed

## Why

## Files changed

## Verification

## Notes / follow-ups
```

Verification must say which commands passed. If a command was not run, say why.

## Stop conditions

Stop and ask or report clearly when:

- the requested behavior conflicts with existing contracts
- the task requires secrets or production credentials
- a migration is required but schema intent is unclear
- tests fail for reasons unrelated to the slice
- the prompt asks for a broad rewrite when a smaller slice is safer
