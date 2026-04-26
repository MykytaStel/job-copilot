# Data Flow — 2026-04-26

This document describes the end-to-end data flow through Job Copilot as it stands today,
including what is implemented, what is partially implemented, and what is planned.

See [service-communication.md](service-communication.md) for the service topology
and internal token flow. See [current-state.md](current-state.md) for an itemised
build status.

---

## High-level flow

```
External sources
  │
  ▼
ingestion (Rust)
  │  fetch → scrape → normalize → dedupe → lifecycle → upsert
  ▼
PostgreSQL
  │
  ├──► engine-api (Rust)  ◄── Browser (via nginx)
  │        │
  │        ├── deterministic ranking
  │        ├── behavior scoring
  │        ├── ML rerank call (optional)
  │        └── presentation layer (JobPresentationResponse)
  │
  ▼
ML sidecar (Python)
  │  fit analysis, rerank, enrichment
  ▼
engine-api  ──►  Browser (Web UI)
                   │
                   └── user actions (save, hide, apply, bad fit, feedback)
                            │
                            ▼
                       engine-api
                         ├── user_events log
                         ├── feedback table
                         └── analytics endpoints
                                  │
                                  ▼
                            ML training pipeline (offline)
```

---

## Ingestion flow

### Implemented

1. **Source fetch** — four scrapers run on a 60-minute daemon interval:
   - Djinni (HTML scraper)
   - Work.ua (HTML scraper)
   - Dou.ua (RSS scraper)
   - Robota.ua (JSON API scraper)

2. **Detail page enrichment** — Djinni, Work.ua, and Robota.ua fetch full job detail
   pages to supplement listing-level data.

3. **Normalization** — each scraper produces a canonical job shape. Source-specific
   quirks stay inside source adapters and do not leak into canonical fields.
   Description quality scoring selects the best available description text.

4. **Dedupe** — keyed on `(source, source_job_id)` → `dedupe_key`. Multi-source merge
   with conflict resolution ensures one canonical record per logical job.

5. **Lifecycle tracking** — each job carries `first_seen_at`, `inactivated_at`, and
   `reactivated_at`. Active/inactive/reactivated states are tracked across ingestion runs.

6. **Canonical upsert** — normalized jobs are written to PostgreSQL, then
   `market_snapshots` are refreshed after successful upserts.

### Partial / known gaps

- **Market read-side** — `market_snapshots` are refreshed on ingest, but current market
  route handlers still query the live `jobs` table directly. The snapshot write path
  exists; read-side decoupling is not complete.

---

## Engine-api: reads and ranking

### Implemented

1. **Job feed** — serves paginated jobs with lifecycle filter, source filter, and
   search profile matching.

2. **Deterministic ranking** — scores every job against the candidate profile using
   role IDs, skills, keywords, work mode, region, seniority, and source signals.
   This layer has no external dependencies and is always active.

3. **Behavior scoring** — aggregates user event signals (saves, hides, bad fits,
   applies) into per-source and per-role-family adjustments applied on top of the
   deterministic score.

4. **Optional ML rerank** — when enabled, engine-api calls the ML sidecar to apply
   a further scoring layer. Failures fall back to the deterministic+behavior score
   without breaking the feed.

5. **Presentation layer** — `JobPresentationResponse` adds UI-ready labels, lifecycle
   primary/secondary labels, and fit reasons before returning to the browser.

6. **Market data** — overview, company stats, salary trends, role demand endpoints
   currently served from live `jobs` queries.

### Partial / known gaps

- Market readers bypass `market_snapshots` as noted above.
- Analytics freshness widget is not yet exposed in the web UI.

---

## ML sidecar: enrichment

### Implemented

- `/api/v1/fit/analyze` — deterministic fit scoring against a profile.
- `/api/v1/rerank` — rerank a bounded job list.
- Six enrichment endpoints using `TemplateEnrichmentProvider` (template-based,
  no external API calls) as the runtime default.
- PII filtering, term normalization, compound term handling.
- Logistic regression trained reranker architecture (`trained_reranker_v3`) — the
  code and artifact format exist, but the model is not production-ready due to
  limited training data (currently ~4 labeled examples).

### Partial / known gaps

- ML output is enrichment input only. It is not canonical domain state.
- The runtime provider default (`template`) and Docker Compose default (`ollama`)
  differ. See [service-communication.md](service-communication.md).
- The trained reranker is disabled by default (`TRAINED_RERANKER_ENABLED=false`).
  It must not be enabled in production until a sufficient labeled dataset exists.

---

## User feedback and events

### Implemented

Feedback actions recorded in engine-api:

| Action | Effect |
|--------|--------|
| Save job | Writes feedback row; boosts future ranking for similar signals |
| Hide job | Writes feedback row; demotes job and similar patterns |
| Bad fit | Writes feedback row; negative signal for reranker training |
| Whitelist company | Positive company-level signal |
| Blacklist company | Negative company-level signal |
| Apply to job | Creates application record; strongest positive training label |

All user actions also log a `user_events` record for analytics and reranker training.

---

## Analytics and learning loop

```
User actions
  │  (save, hide, apply, view, bad fit)
  ▼
user_events + feedback table  (engine-api / PostgreSQL)
  │
  ▼
Analytics endpoints  (behavior signals, funnel, source quality, LLM enrichment)
  │
  ▼
GET /api/v1/profiles/:id/reranker-dataset  (engine-api, offline export)
  │
  ▼
ML training script  (apps/ml/app/trained_reranker/)
  │
  ├── temporal holdout evaluation
  └── promote artifact  (apps/ml/models/trained-reranker-v3.json)
            │
            ▼
       engine-api optional live layer  (TRAINED_RERANKER_ENABLED=true)
```

### Implemented

- User event logging and aggregation.
- Analytics endpoints: funnel, behavior signals, source quality, LLM enrichment context.
- Reranker dataset export endpoint with `outcome_label_v3` labeling policy.
- Offline ML training script and temporal holdout evaluation.
- Per-profile reranker metrics (`ml_examples_since_retrain`, `ml_last_retrained_at`, etc.).
- Background retrain poller for profiles above the retrain threshold.

### Partial / known gaps

- Training dataset is very small (~4 examples per profile in early use).
- The retrain threshold is set at ≥ 30 labelable outcomes. In early use, this is
  not yet reached.
- Promotion of a trained artifact to production use is a manual step and requires
  explicit operator decision.
- The analytics freshness widget (showing ingestion recency in the UI) is not yet
  implemented in the web dashboard.

---

## What is not implemented (planned)

- Message queue or event bus between services (no Kafka or RabbitMQ).
- Ingestion writing through engine-api instead of directly to PostgreSQL.
- Read-side market data exclusively from `market_snapshots` (decoupling planned).
- Semantic embeddings (deferred until sufficient labeled data exists).
- Paid LLM API as the default enrichment path.
- Production deployment target beyond Docker Compose for local/dev use.
