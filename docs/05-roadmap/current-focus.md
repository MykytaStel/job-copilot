# Current Focus — 2026-04-21

## ✅ Завершено (Фаза 1 — всі пункти)

1. **Sidebar profile** — реальне ім'я/email з API (`AppShellNew.tsx:288-313`)
2. **Query invalidation** — web-side TanStack invalidation після feedback (`JobDetails.tsx:270-276`); ML reranker stateless, server-side не потрібен
3. **Canonical role catalog** — `RoleId` enum, display_name, aliases, підключено до scoring (`domain/role/catalog.rs`, `role_id.rs`)
4. **Bootstrap training data** — скрипт генерує labeled examples з user_events (`ml/app/bootstrap_training.py`)
5. **LLM provider** — правильні model names (`gpt-4o-mini`, `llama3.1:8b`); Ollama є default провайдером (`docker-compose.yml`)
6. **Freshness decay** — jobs > 14 днів зменшують score; `compute_freshness_decay()` (`services/matching/scoring.rs`)

## ✅ Завершено (Фаза 2 — Market Intelligence)

- `market_snapshots` таблиця (`migrations/20260416000000_add_market_snapshots.sql`)
- API endpoints: overview, companies, salary trends, role demand (`api/routes/market.rs`)
- Web сторінка `/market` з усіма секціями (`pages/Market.tsx`)
- **⚠️ Відкрито:** job що пише агрегати в `market_snapshots` — відсутній (таблиця є, даних нема)

## ✅ Завершено (Фаза 3 — часткова)

- **Notifications (3.1)** — таблиця + 3 API endpoints + UI + bell icon + ingestion trigger (`notifications.rs`, `Notifications.tsx`, `ingestion/src/db.rs`)
- **Global search (3.2)** — Cmd+K overlay з debounced search (`GlobalSearch.tsx`)
- **Salary/languages (3.5)** — міграція є (`20260419153000_add_profile_compensation_and_languages.sql`); UI частково

## ✅ Завершено (Фаза 4 — Ollama та ML)

- **Ollama provider (4.1)** — `OllamaEnrichmentProvider` використовує `/api/chat` + `format: json` (`llm_provider_remote.py:172`)
- **Reranker retrain (4.2)** — bootstrap скрипт + `/api/v1/reranker/bootstrap` endpoint (`scoring_routes.py:84`)
- **Template provider (4.3)** — повна реалізація всіх 6 методів (`llm_provider_template.py`)
- **ML enrichment endpoints** — всі 6 routes (`enrichment_routes.py`)

## Зараз робимо (Sprint 3)

| # | Задача | Складність | Пріоритет |
|---|--------|------------|-----------|
| A | **Market snapshot aggregation job** — заповнити `market_snapshots` реальними даними (агрегація з `jobs` таблиці) | M | Критично |
| B | **CV Tailoring** — ML endpoint + web modal (відмінно від cover letter — фокус на адаптації CV під JD) | M | Висока |
| C | **Settings сторінка** — `/settings` route + profile prefs + notification prefs UI | M | Висока |
| D | **Search profile persistence** — DB таблиця для збереження `SearchPreferences`; зараз будується on-demand | M | Середнє |
| E | **Profile completion indicator** — % показник заповненості профілю | XS | Низьке |

## Не робимо зараз

- Semantic embeddings / sentence-transformers (Phase 4.4) — потрібні дані спочатку
- Auth / multi-user (Phase 5) — потрібна монетизація спочатку
- Stripe (Phase 5) — після auth
- Email delivery — після notifications прогресу
