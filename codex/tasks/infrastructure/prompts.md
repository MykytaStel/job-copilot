# Block J — Infrastructure & Observability (6 tasks)

---

## J1 — Database indexes audit

```
You are working in the Job Copilot monorepo.
First read: CLAUDE.md, apps/engine-api/migrations/ (all migration files),
apps/engine-api/src/db/ or src/api/routes/ (WHERE clause patterns in queries)

## Goal
Audit all migration files and find tables/columns that are frequently used in WHERE
clauses, JOINs, and ORDER BY but may lack indexes. Add missing indexes via a new
migration file.

Priority columns to check:
- jobs: profile_id (if exists), source, status, first_seen_at, last_confirmed_active_at, company
- job_feedback: profile_id, job_id
- applications: profile_id, status
- user_events: profile_id, event_type, created_at
- notifications: profile_id, is_read, created_at
- market_snapshots: snapshot_date

## Inspect first
- apps/engine-api/migrations/ — all migration files, existing index definitions
- apps/engine-api/src/ — most frequent WHERE patterns in sqlx queries

## Likely files to modify
- New migration: apps/engine-api/migrations/XXXX_add_missing_indexes.sql

## Rules
- Do not add indexes that already exist.
- Only add B-tree indexes (no GiST/GIN unless for FTS columns).
- Partial indexes where appropriate (e.g. WHERE status = 'active').
- Migration must be safe to run on existing data.

## Acceptance criteria
- [ ] New migration adds indexes for all identified missing cases
- [ ] Migration runs without error
- [ ] Existing cargo tests still pass
- [ ] List of added indexes documented in migration comment

## Verification commands
cargo test --manifest-path apps/engine-api/Cargo.toml
cargo check --manifest-path apps/engine-api/Cargo.toml

## Final response format
## What changed
## Why
## Files changed
## Verification
## Notes / follow-ups
```

---

## J2 — API response time tracking per endpoint

```
You are working in the Job Copilot monorepo.
First read: CLAUDE.md, apps/engine-api/src/main.rs (middleware stack),
apps/engine-api/Cargo.toml (existing dependencies — axum-prometheus)

## Goal
Add per-endpoint response time histograms to Prometheus metrics.
The axum-prometheus crate is already a dependency. Ensure it's properly configured
to track latency per route (not just a global metric).

If axum-prometheus is already configured, verify that the metric labels include
the route path and HTTP method. If not, reconfigure to add those labels.

## Inspect first
- apps/engine-api/src/main.rs — middleware configuration
- apps/engine-api/Cargo.toml — axum-prometheus version
- infra/prometheus/prometheus.yml — scrape config

## Likely files to modify
- apps/engine-api/src/main.rs (configure prometheus middleware with route labels)

## Rules
- Do not add a new metrics dependency — use axum-prometheus.
- Metrics must include route path label for per-endpoint breakdown.
- Prometheus endpoint on :9090 or wherever it's configured.

## Acceptance criteria
- [ ] http_request_duration_seconds histogram exists with route/method labels
- [ ] Cargo check passes
- [ ] Metrics endpoint returns labeled histogram

## Verification commands
cargo check --manifest-path apps/engine-api/Cargo.toml

## Final response format
## What changed
## Why
## Files changed
## Verification
## Notes / follow-ups
```

---

## J3 — Grafana dashboard: API latency

```
You are working in the Job Copilot monorepo.
First read: CLAUDE.md, infra/grafana/ (existing dashboard JSON files),
infra/prometheus/prometheus.yml

## Goal
Add a Grafana dashboard panel showing:
- P50/P95/P99 response times per endpoint
- Top 5 slowest endpoints (table panel)
- Request rate per endpoint (req/s)
- Error rate (4xx/5xx) per endpoint

Create as a new JSON dashboard file in infra/grafana/dashboards/ or extend
the existing dashboard.

## Inspect first
- infra/grafana/ — existing dashboard JSON structure
- infra/prometheus/prometheus.yml — metric names available
- infra/docker-compose.yml — Grafana provisioning config

## Likely files to modify
- infra/grafana/dashboards/ (new or extended dashboard JSON)

## Rules
- Use Prometheus queries based on http_request_duration_seconds metric.
- Dashboard should auto-refresh every 30s.
- No hardcoded data — queries must be dynamic.

## Acceptance criteria
- [ ] New/updated dashboard JSON is valid Grafana schema
- [ ] Panel for P50/P95/P99 latency by route exists
- [ ] Panel for top 5 slowest endpoints exists
- [ ] Error rate panel exists

## Verification commands
# Validate JSON:
python3 -c "import json; json.load(open('infra/grafana/dashboards/<filename>.json'))"

## Final response format
## What changed
## Why
## Files changed
## Verification
## Notes / follow-ups
```

---

## J4 — Structured JSON logging

```
You are working in the Job Copilot monorepo.
First read: CLAUDE.md, apps/engine-api/src/main.rs (logging setup),
apps/ml/app/api.py (FastAPI logging setup)

## Goal
Ensure all services output structured JSON logs (not plain text) for easier parsing
and aggregation.

Engine-API: verify tracing subscriber outputs JSON format in production.
If not, configure tracing-subscriber with JSON format when LOG_FORMAT=json env var is set.

ML Sidecar: configure uvicorn/FastAPI to output JSON access logs when LOG_FORMAT=json.
Add LOG_FORMAT=json to docker-compose.yml for both services.

## Inspect first
- apps/engine-api/src/main.rs — tracing/logging setup
- apps/ml/app/api.py — logging configuration
- infra/docker-compose.yml — env vars for both services

## Likely files to modify
- apps/engine-api/src/main.rs (conditional JSON formatter)
- apps/engine-api/Cargo.toml (add tracing-subscriber json feature if missing)
- apps/ml/app/api.py (add JSON log formatter)
- infra/docker-compose.yml (add LOG_FORMAT=json)

## Rules
- JSON logging only when LOG_FORMAT=json env var set.
- Default (no env var) stays as human-readable text for local dev.
- Do not log token values, secrets, or PII.

## Acceptance criteria
- [ ] Engine-API outputs JSON logs when LOG_FORMAT=json
- [ ] ML sidecar outputs JSON access logs when LOG_FORMAT=json
- [ ] docker-compose.yml sets LOG_FORMAT=json
- [ ] Cargo check + ML pytest pass

## Verification commands
cargo check --manifest-path apps/engine-api/Cargo.toml
cd apps/ml && python -m pytest

## Final response format
## What changed
## Why
## Files changed
## Verification
## Notes / follow-ups
```

---

## J5 — Improved health/readiness check

```
You are working in the Job Copilot monorepo.
First read: CLAUDE.md, apps/engine-api/src/api/routes/health.rs,
apps/ml/app/api.py (/ready endpoint),
infra/docker-compose.yml (healthcheck configs)

## Goal
Improve the /ready endpoint in both engine-api and ML sidecar to return detailed
component status:
{ status: "ready"|"degraded"|"not_ready",
  components: {
    database: { status: "ok"|"error", latency_ms: u32 },
    ml_sidecar: { status: "ok"|"error"|"unreachable" },
    ingestion: { status: "ok"|"stale", last_run_at: Option<DateTime> }
  }
}

"Degraded" = some components unavailable but core is functional.
"Not ready" = database unavailable.

## Inspect first
- apps/engine-api/src/api/routes/health.rs — current /ready implementation
- apps/ml/app/api.py — /ready endpoint
- infra/docker-compose.yml — healthcheck intervals

## Likely files to modify
- apps/engine-api/src/api/routes/health.rs (expand /ready response)
- apps/engine-api/src/api/routes/health.rs (add DB ping + ML sidecar check)
- infra/docker-compose.yml (update healthcheck commands)

## Rules
- /ready must respond within 1 second.
- DB check: simple SELECT 1 query.
- ML sidecar check: GET /health with 500ms timeout.
- Ingestion check: query max(first_seen_at) from jobs (cached 60s).

## Acceptance criteria
- [ ] /ready returns detailed component status JSON
- [ ] "degraded" when ML unreachable but DB ok
- [ ] "not_ready" when DB unreachable
- [ ] Responds in < 1s
- [ ] Cargo tests pass

## Verification commands
cargo test --manifest-path apps/engine-api/Cargo.toml
cargo check --manifest-path apps/engine-api/Cargo.toml

## Final response format
## What changed
## Why
## Files changed
## Verification
## Notes / follow-ups
```

---

## J6 — Docker Compose production config

```
You are working in the Job Copilot monorepo.
First read: CLAUDE.md, infra/docker-compose.yml,
apps/engine-api/src/main.rs (env var usage),
apps/ml/app/settings.py (env var usage)

## Goal
Create infra/docker-compose.prod.yml (override file) for production deployment:
- Remove volume mounts of source code (use built images only)
- Set LOG_FORMAT=json for all services
- Set ML_LLM_PROVIDER=template (or ollama if configured)
- Add restart: unless-stopped to all services
- Add explicit memory limits (engine-api: 512m, ml: 1g, postgres: 1g)
- Remove Grafana and Prometheus from prod config (optional monitoring)
- Add POSTGRES_PASSWORD from environment variable (not hardcoded)

Document usage:
docker compose -f infra/docker-compose.yml -f infra/docker-compose.prod.yml up -d

## Inspect first
- infra/docker-compose.yml — current services and config
- apps/engine-api/src/main.rs — required env vars
- apps/ml/app/settings.py — required env vars

## Likely files to modify
- New file: infra/docker-compose.prod.yml

## Rules
- Do not change docker-compose.yml (it's the dev config).
- prod.yml is an override — only specify what changes from dev.
- Add a comment explaining each override.

## Acceptance criteria
- [ ] docker-compose.prod.yml exists and is valid YAML
- [ ] Override adds restart policies
- [ ] Override adds memory limits
- [ ] Override uses env var for DB password
- [ ] README or comments explain usage

## Verification commands
docker compose -f infra/docker-compose.yml -f infra/docker-compose.prod.yml config

## Final response format
## What changed
## Why
## Files changed
## Verification
## Notes / follow-ups
```
