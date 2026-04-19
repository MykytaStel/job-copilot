# Current State — 2026-04-18

## Що побудовано і працює

### Ingestion
- ✅ 4 scrapers: Djinni (HTML), Work.ua (HTML), Dou.ua (RSS), Robota.ua (JSON API)
- ✅ Detail page enrichment (Djinni, Work.ua, Robota.ua)
- ✅ Dedupe: `(source, source_job_id)` → dedupe_key → merge
- ✅ Lifecycle: first_seen_at, inactivated_at, reactivated_at
- ✅ Multi-source merge з conflict resolution
- ✅ Daemon mode: 60 хв інтервал, всі 4 sources
- ✅ Description quality scoring (вибирає кращий текст)

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
- ✅ Presentation layer: JobPresentationResponse (UI labels)
- ✅ PostgreSQL 16, 10 міграцій

### ML Sidecar (Python)
- ✅ `/api/v1/fit/analyze` — deterministic fit scoring
- ✅ `/api/v1/rerank` — rerank job list
- ✅ Enrichment endpoints (6 шт.) — шаблонний fallback (TemplateEnrichmentProvider)
- ✅ PII filtering, term normalization, compound term handling
- ✅ Logistic regression trained reranker (v2) — архітектура є, 4 приклади
- ✅ Seniority normalization у engine API client
- ⚠️ LLM: template fallback (OpenAI API key відсутній, model name `gpt-5.4-mini` — не існує)
- ❌ Ollama provider — не реалізований
- ❌ Bootstrap training data pipeline

### Web (React 19)
- ✅ Dashboard: job feed, lifecycle filter, source filter, ML ranking toggle
- ✅ Job Detail: fit analysis, match tab, lifecycle tab, feedback actions
- ✅ Profile: edit, PDF upload, analyze, search profile builder
- ✅ Application Board: 5-column Kanban, CSV export
- ✅ Application Detail: повний запис (offer, contacts, notes, tasks, activities)
- ✅ Feedback Center: saved/hidden/badfit/companies tabs
- ✅ Analytics: funnel, behavior signals, source quality, LLM enrichment
- ❌ Sidebar показує hardcoded текст замість profile name/email
- ❌ Notifications — іконка є, сторінки немає
- ❌ Settings — іконка є, сторінки немає
- ❌ Global search — placeholder, не функціональний
- ❌ ML rerank cache не інвалідується після profile/feedback змін

### Infrastructure
- ✅ Docker Compose: postgres + engine-api + web + scraper + ml
- ✅ PostgreSQL health checks, restart policies
- ✅ Auto-migrations при старті engine-api

## Відомі проблеми

| Проблема | Файл | Вплив |
|----------|------|-------|
| Sidebar hardcoded | `AppShellNew.tsx:253` | Cosmetic, confusing |
| LLM model name = `gpt-5.4-mini` | `llm_provider.py` | LLM не працює |
| Trained reranker: 4 examples | `models/trained-reranker-v2.json` | Модель нефункціональна |
| ML rerank staleTime: 5 min | `Dashboard.tsx` | Стара аналітика після feedback |
| Analytics context string-cache | `Analytics.tsx` | Insights не оновлюються |
| Notifications: no route/page | `AppShellNew.tsx` | Фіча недоступна |
| Global search: placeholder | Header component | Фіча недоступна |

## Що не потребує змін (правильно)
- Архітектура (Rust + Python + React)
- PostgreSQL schema (добра)
- Scraper архітектура (daemon mode, 4 sources)
- React Query pattern (баги в invalidation, але не архітектурні)
- Dedupe logic
- Presentation layer (JobPresentationResponse)
