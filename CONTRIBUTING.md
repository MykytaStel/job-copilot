# Contributing

## Branch strategy
- `main` is always the current stable line
- do not commit directly to `main`
- create short-lived branches from `main`

Branch names:
- `feat/<scope>`
- `fix/<scope>`
- `refactor/<scope>`
- `chore/<scope>`

Examples:
- `feat/search-profile-filters`
- `refactor/web-direct-api-imports`
- `fix/ml-timeout-error`
- `chore/ci-guardrails`

## Commit style
Use simple conventional-style commits:
- `feat(web): add role filter`
- `fix(engine-api): handle missing profile id`
- `refactor(ml): split enrichment routes`
- `chore(repo): add ci workflow`

## Pull requests
Every change goes through a PR, even if you work alone.

PR checklist:
- code builds
- typecheck passes
- relevant tests pass
- scope is small and reviewable
- PR title describes one slice only

## Merge strategy
- use **Squash and merge**
- keep `main` linear and readable

## Engineering rules
- prefer small slices
- do not mix refactor + feature + infra in one PR
- keep Rust as source of truth
- treat LLM as enrichment, not canonical state
- keep `apps/web/src/api.ts` as compatibility facade only