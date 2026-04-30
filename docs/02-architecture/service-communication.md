# Service Communication — 2026-04-26

This document describes how the current services communicate with each other,
what the internal token flow looks like, and what does not yet exist.

---

## Service Topology

```
Browser
  │
  └─► nginx (port 3000)
        ├─ /api/*  ─────────────────► engine-api (port 8080)
        │                                   │
        ├─ /ml/*   ─────────────────► ml sidecar (port 8000)
        │                                   │
        └─ /*      ──────────────── web SPA (nginx serves built assets)
                                            │
PostgreSQL ◄────────────────────── engine-api
PostgreSQL ◄────────────────────── ingestion (direct, no API layer)
Prometheus/Grafana ◄────────────── engine-api metrics endpoint
```

---

## Communication Paths

### Browser to engine-api

The browser sends all API calls to `/api/*`. Nginx strips the `/api` prefix and
proxies to `engine-api:8080`. The full stack is exposed on a single origin
(`localhost:3000` in Docker Compose), so no CORS applies for the bundled web app.

### Browser to ML sidecar

The browser can reach the ML sidecar directly via `/ml/*` through nginx. Nginx
strips the `/ml` prefix and proxies to `ml:8000`. Long-running enrichment calls
have a proxy read timeout of 120 seconds.

In practice, most ML calls originate from engine-api on the server side, not
directly from the browser.

### Engine-api to ML sidecar

Engine-api calls the ML sidecar over HTTP (configured via `ML_SIDECAR_BASE_URL`).
These are server-to-server calls that bypass nginx. Engine-api sends an internal
token (`ML_INTERNAL_TOKEN`) in these requests.

### Ingestion to PostgreSQL

The ingestion service connects directly to PostgreSQL. It does not go through
engine-api. Ingestion owns the write path for job supply: fetch, scrape, normalize,
dedupe, and upsert. It also triggers `market_snapshots` refresh after successful
upserts.

### Engine-api to PostgreSQL

Engine-api connects directly to PostgreSQL for all canonical domain reads and writes:
profiles, jobs, applications, feedback, events, notifications, and market data.

### Observability

Engine-api exposes a metrics endpoint consumed by Prometheus. Grafana reads from
Prometheus. These are read-only pull paths with no reverse dependency on engine-api.

---

## Internal ML Token Flow

1. Engine-api reads `ML_INTERNAL_TOKEN` from its environment at startup.
2. Engine-api sends this token in HTTP requests to ML internal endpoints.
3. ML validates the token to allow access to internal-only routes.
4. The token value is never logged or included in responses.

Production startup fails fast when `ML_INTERNAL_TOKEN` is absent. Local development
can still run without the token.

---

## ML Provider Selection

The ML sidecar selects an LLM provider via the `ML_LLM_PROVIDER` environment variable.

| Provider | Description |
|----------|-------------|
| `template` | Deterministic template-based enrichment. No external API calls. Safe default. |
| `ollama` | Local LLM via Ollama. Requires a running Ollama instance. |
| `openai` | OpenAI API. Requires `OPENAI_API_KEY`. Not the default path. |
| `anthropic` | Anthropic API. Optional. Not the default path. |

**Runtime default:** `template` (set in `apps/ml/app/settings.py`)

**Docker Compose default:** `template` (set via `ML_LLM_PROVIDER: "${ML_LLM_PROVIDER:-template}"`)

Runtime code and Docker Compose intentionally share the same safe default. Use
`ML_LLM_PROVIDER=ollama` only when Ollama is running and intended.

Paid providers (OpenAI, Anthropic) are never the default path. They require explicit
configuration.

---

## Failure Behavior

- **ML unavailable:** Engine-api must fall back to deterministic ranking/search when the
  ML sidecar is unreachable. Core job feed, feedback, and application flows must not
  depend on ML availability.
- **Ingestion stopped:** Engine-api continues serving existing job data. No real-time
  dependency on ingestion being live.
- **PostgreSQL unavailable:** Engine-api and ingestion fail; this is a hard dependency.

---

## What Does Not Exist Yet

The following patterns are not implemented and should not be assumed:

- **No message queue** — there is no Kafka, RabbitMQ, or similar async messaging layer
- **No service mesh** — no Istio, Linkerd, or mTLS between services
- **No separate ingestion API** — ingestion writes directly to PostgreSQL, not through engine-api
- **No production deployment contract** — the current infra is Docker Compose for local/dev;
  a production deployment target has not been defined
- **No health-based circuit breaker** — ML fallback is handled in application code, not at
  the infrastructure level
