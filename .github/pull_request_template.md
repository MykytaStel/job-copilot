## What changed

-

## Why

-

## How I verified it

- [ ] `cd apps/web && pnpm test -- --run`
- [ ] `cd apps/web && pnpm run typecheck`
- [ ] `cd apps/web && pnpm run eslint`
- [ ] `cd apps/engine-api && cargo test -q`
- [ ] `cd apps/ingestion && cargo test -q`
- [ ] `cd apps/ml && pytest -q`

## Risk level

- [ ] Low
- [ ] Medium
- [ ] High

## Notes

- If runtime truth changed, update `docs/05-roadmap/current-focus.md`, `docs/02-architecture/current-state.md`, and the relevant roadmap/status doc in the same PR.
- For non-trivial slices, include or reference an ADR using `docs/02-architecture/adr-template.md`.
