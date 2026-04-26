# Current State — 2026-04-26

## Що побудовано і працює

### Ingestion
- ✅ 4 scrapers: Djinni (HTML), Work.ua (HTML), Dou.ua (RSS), Robota.ua (JSON API)
- ✅ Detail page enrichment (Djinni, Work.ua, Robota.ua)
- ✅ Dedupe: `(source, source_job_id)` → dedupe_key → merge
- ✅ Lifecycle: first_seen_at, inactivated_at, reactivated_at
- ✅ Multi-source merge з conflict resolution
- ✅ Daemon mode: 60 хв інтервал, всі 4 sources
- ✅ Description quality scoring (вибирає кращий текст)
- ✅ Standalone migrations via `db_runtime::run_migrations()`

### Engine-API (Rust)
- ✅ Canonical domain models: Job, JobVariant, Profile, Application, Feedback
- ✅ Jobs: feed, lifecycle filter, source filter, detail, variants
- ✅ Profile: create, analyze (skills/summary), search profile builder
- ✅ Applications: Kanban (saved/applied/interview/offer/rejected)
- ✅ Application detail: notes, contacts, tasks, activities, offers
- ✅ Feedback: save/hide/bad fit per job; whitelist/blacklist per company
- ✅ User events logging
- ✅ Analytics: summary, funnel, behavior signals, LLM context
- ✅ Reranker: deterministic + behavior-based + trained (logistic regression)
- ✅ Trained reranker export dataset endpoint
- ✅ Presentation layer: JobPresentationResponse (UI labels + explicit lifecycle primary/secondary labels)
- ✅ PostgreSQL 16, 10 міграцій
- ✅ Notifications endpoints + profile-scoped unread count
- ✅ Market endpoints: overview, companies, salary trends, role demand
- ✅ `market_snapshots` refresh after successful ingestion upserts
- ✅ Search profile preferences persist on profiles and hydrate back into the web builder
- ⚠️ Current market readers still query live `jobs` directly

### ML Sidecar (Python)
- ✅ `/api/v1/fit/analyze` — deterministic fit scoring
- ✅ `/api/v1/rerank` — rerank job list
- ✅ Enrichment endpoints (6 шт.) — шаблонний fallback (TemplateEnrichmentProvider)
- ✅ PII filtering, term normalization, compound term handling
- ✅ Logistic regression trained reranker (v2) — архітектура є, 4 приклади
- ✅ Seniority normalization у engine API client
- ✅ OpenAI provider with current model defaults
- ✅ Ollama provider is implemented
- ✅ Bootstrap training data pipeline
- ⚠️ Runtime provider default is `template`; Docker Compose overrides default env to `ollama`

### Web (React 19)
- ✅ Dashboard: job feed, lifecycle filter, source filter, ML ranking toggle
- ✅ Dashboard ranked mode is deferred and capped to a bounded rerank window
- ✅ Job Detail: fit analysis, match tab, lifecycle tab, feedback actions
- ✅ Profile: edit, PDF upload, analyze, search profile builder
- ✅ Application Board: 5-column Kanban, CSV export
- ✅ Application Detail: повний запис (offer, contacts, notes, tasks, activities)
- ✅ Feedback Center: saved/hidden/badfit/companies tabs
- ✅ Analytics: funnel, behavior signals, source quality, LLM enrichment
- ✅ Active shell reads real profile data in the sidebar/header
- ✅ Notifications page + unread badge
- ✅ Global search overlay
- ✅ Query invalidation for profile/feedback-driven rerank refresh
- ✅ Market Intelligence page
- ✅ Minimal settings route/page
- ✅ Profile completion indicator

### Infrastructure
- ✅ Docker Compose: postgres + engine-api + web + scraper + ml
- ✅ PostgreSQL health checks, restart policies
- ✅ Auto-migrations при старті engine-api
- ✅ Docker default `ML_LLM_PROVIDER=ollama`

## Відомі проблеми

| Проблема | Файл | Вплив |
|----------|------|-------|
| Market readers still bypass snapshots | `market` routes query `jobs` directly | Snapshot refresh exists, but read-side decoupling is still incomplete |
| Settings preferences are still partial | `apps/web/src/pages/Settings.tsx` | Dedicated notification controls and richer profile preferences are not implemented yet |
| Analytics freshness widget відсутній | `apps/web/src/pages/Analytics.tsx` | Ingestion recency is not visible in the analytics flow |
| Provider defaults are inconsistent | `apps/ml/app/settings.py`, `infra/docker-compose.yml` | Runtime code and Docker Compose still disagree on the default ML provider |

## Що не потребує змін (правильно)
- Архітектура (Rust + Python + React)
- PostgreSQL schema (добра)
- Scraper архітектура (daemon mode, 4 sources)
- React Query pattern (баги в invalidation, але не архітектурні)
- Dedupe logic
- Presentation layer (JobPresentationResponse)
