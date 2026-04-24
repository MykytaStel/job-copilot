# Verification Matrix

Updated: 2026-04-24

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

## Cross-app runtime

- `VERIFY_REQUESTS=100 VERIFY_CONCURRENCY=10 pnpm verify:first-100`
- `PROFILE_IDS_CSV=<profile-1>,<profile-2> VERIFY_BOOTSTRAP_CONCURRENCY=4 pnpm verify:bootstrap-pressure`
- `PROFILE_ID=<profile-id> JOB_IDS_CSV=<job-1>,<job-2> PROFILE_IDS_CSV=<profile-1>,<profile-2> pnpm verify:mixed-load`

Optional env vars for profile-scoped checks:

- `PROFILE_ID=<profile-id>`
- `PROFILE_IDS_CSV=<profile-id-1>,<profile-id-2>` for bootstrap pressure verification
- `JOB_IDS_CSV=<job-id-1>,<job-id-2>`
- `ENGINE_API_BASE_URL=http://127.0.0.1:8080`
- `ML_BASE_URL=http://127.0.0.1:8000`
- `ML_INTERNAL_TOKEN=<token>` when ML internal auth is enabled

Bootstrap pressure verifier env vars:

- `VERIFY_BOOTSTRAP_REQUESTS=<n>` defaults to the number of supplied profile ids
- `VERIFY_BOOTSTRAP_CONCURRENCY=<n>` default `4`
- `VERIFY_BOOTSTRAP_MIN_EXAMPLES=<n>` default `15`
- `VERIFY_BOOTSTRAP_TIMEOUT_SECONDS=<n>` default `60`
- `VERIFY_BOOTSTRAP_POLL_INTERVAL_SECONDS=<n>` default `0.5`

Mixed-load verifier env vars:

- `VERIFY_MIXED_BASELINE_REQUESTS=<n>` default `20`
- `VERIFY_MIXED_RERANK_REQUESTS=<n>` default `60`
- `VERIFY_MIXED_RERANK_CONCURRENCY=<n>` default `10`
- `VERIFY_MIXED_BOOTSTRAP_REQUESTS=<n>` default number of supplied profile ids
- `VERIFY_MIXED_BOOTSTRAP_CONCURRENCY=<n>` default `4`
- `VERIFY_MIXED_BOOTSTRAP_MIN_EXAMPLES=<n>` default `15`
- `VERIFY_MIXED_TIMEOUT_SECONDS=<n>` default `60`
- `VERIFY_MIXED_POLL_INTERVAL_SECONDS=<n>` default `0.25`
- `VERIFY_MIXED_MAX_RERANK_P95_SLOWDOWN=<ratio>` default `3.0`
- `VERIFY_MIXED_MAX_RERANK_ERROR_RATE=<ratio>` default `0.0`
- `VERIFY_MIXED_CLEANUP_RUNTIME=<true|false>` default `true`
- `VERIFY_MIXED_CLEANUP_DOCKER_CONTAINER=<container>` default `job-copilot-ml`
- `VERIFY_MIXED_RESOURCE_CONTAINERS_CSV=<container-1>,<container-2>` default `job-copilot-ml,job-copilot-engine-api`

Notes for pressure runs:

- Use distinct profile ids to exercise the global bootstrap semaphore.
- Repeating the same profile id is valid for lock-conflict verification, but those tasks may fail with `bootstrap already running`.
- To pressure-test real training time rather than the control path, prepare each profile with enough labeled examples before running the verifier.
- If bootstrap tasks fail with `engine-api error: 400`, check that each profile is valid for the reranker dataset flow first, for example by running profile analysis and preparing feedback history.
- Use `verify:mixed-load` after `verify:bootstrap-pressure` to confirm rerank latency stays within an acceptable slowdown envelope while bootstrap jobs are queued or training.
- `verify:mixed-load` also samples `docker stats` for the configured containers so you can see whether latency regressions line up with CPU or memory saturation.
- `verify:mixed-load` now cleans up the runtime artifacts and task files it created by default, so `ready_status_after_cleanup` may be `degraded` even when the verification itself passed.

## Observability checks

When `docker compose up` is running:

- Structured JSON logs from engine-api: `docker compose logs engine-api | jq .` — every request should have a `request_id` field
- ML sidecar logs: `docker compose logs ml | jq .` — requests should show `request_id` propagated from engine-api
- Prometheus: open `http://localhost:9090` → Status → Targets, confirm `engine-api` and `ml` endpoints show as UP
- Grafana: open `http://localhost:3001` (admin/admin) → Engine API dashboard shows request rate and latency panels

Log correlation check:
1. Make any API call that triggers the ML sidecar (e.g. a rerank request)
2. Note the `x-request-id` header in the response
3. `docker compose logs engine-api | jq 'select(.request_id == "<id>")'` — should show the outbound ML call
4. `docker compose logs ml | jq 'select(.request_id == "<id>")'` — should show the inbound ML request

## ID-clean checks

After any DTO or contract change, verify:

```bash
# No source_job_id in web JSX/TSX
grep -rn "sourceJobId\|source_job_id" apps/web/src --include="*.tsx" | grep -v "//\|test"

# No application_id in rendered output
grep -rn "applicationId\b" apps/web/src/pages --include="*.tsx"
```

Expected: zero hits in rendered JSX. `applicationId` may appear only as a function parameter or in API call URLs.

## Applications page checks

- Open `/applications` → table view renders, columns visible: Status, Role, Company, Updated, Change
- Click a row → right-side quick panel slides in with application summary
- Change status via inline dropdown → row updates immediately, panel stays open
- Filter by status pill in header → table filters to matching rows
- Search by role or company name → results filter in real time
- Click "Open full detail" in panel → navigates to `/applications/:id`

## Notes

- Keep checks app-local instead of forcing one repo-wide test layout.
- Prefer green tests before formatting-only cleanup.
- If a slice changes contracts, rerun the touching app plus its closest caller.
- After a Rust DTO change: `cargo check` in `apps/engine-api`; after a TypeScript contract change: `pnpm tsc --noEmit` in `apps/web`.
