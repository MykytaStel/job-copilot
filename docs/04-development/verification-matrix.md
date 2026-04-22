# Verification Matrix

Updated: 2026-04-22

Run these commands from the repo root or from the app directories shown below.

## apps/web

- `cd apps/web && pnpm test -- --run`
- `cd apps/web && pnpm run typecheck`
- `cd apps/web && pnpm run eslint`

## apps/engine-api

- `cd apps/engine-api && cargo test -q`

## apps/ingestion

- `cd apps/ingestion && cargo test -q`

## apps/ml

- `cd apps/ml && pytest -q`

## Notes

- Keep checks app-local instead of forcing one repo-wide test layout.
- Prefer green tests before formatting-only cleanup.
- If a slice changes contracts, rerun the touching app plus its closest caller.
