# Priority Roadmap — 2026-04-22

## Принципи
- Тільки реальні дані, ніяких моків там де є реальне джерело
- LLM — тільки після того як Ollama налаштований або платна підписка активна
- Фіксити видимі баги до нових фічей
- Market Intelligence будується з даних що вже є (без нових джерел)

---

## ФАЗА 1 — Основи та реальні дані ✅ ЗАВЕРШЕНО

| # | Задача | Статус | Доказ |
|---|--------|--------|-------|
| 1.1 | Sidebar: profile name/email замість hardcoded | ✅ | Active runtime path is `web/src/App.tsx` -> `web/src/AppShell.tsx`; `AppShellNew.tsx` is only a re-export alias |
| 1.2 | Query invalidation: rerank/analytics після feedback | ✅ | `JobDetails.tsx:270-276`; ML reranker stateless |
| 1.3 | Canonical role catalog (`RoleId` enum + aliases) | ✅ | `domain/role/catalog.rs`, `role_id.rs` |
| 1.4 | Виправити LLM provider (model name + Ollama path) | ✅ | `llm_provider_factory.py:31`, `llm_provider_remote.py:178` |
| 1.5 | Bootstrap training data з user_events | ✅ | `ml/app/bootstrap_training.py` |
| 1.6 | Freshness decay у scoring (jobs > 14 днів) | ✅ | `services/matching/scoring.rs` — `compute_freshness_decay()` |
| 1.7 | Salary fit у scoring (якщо profile має salary range) | ✅ | `services/salary.rs` — `score_search_salary()` ±8 pts |
| 1.8 | Company reputation у scoring | ✅ (partial) | Whitelist **+10** (не +5); blacklist = **виключення з результатів** (не -20); `api/routes/search/reranking.rs:19` |
| 1.9 | Lifecycle presentation semantics in UI/read-model | ✅ | Engine presentation now returns explicit lifecycle labels; web surfaces use them instead of inferring from `postedAt` alone |
| 1.10 | Deferred + bounded dashboard rerank | ✅ | Ranked mode reranks on demand and caps the first window to keep feed browsing responsive |

---

## ФАЗА 2 — Market Intelligence ✅ ЗДЕБІЛЬШОГО ЗАВЕРШЕНО

| # | Задача | Статус | Доказ |
|---|--------|--------|-------|
| 2.1 | Company hiring velocity | ✅ | `api/routes/market.rs` — `get_market_companies()` |
| 2.2 | Salary trends по role/seniority | ✅ | `api/routes/market.rs` — `get_market_salary_trend()` |
| 2.3 | Role demand index | ✅ | `api/routes/market.rs` — `get_market_role_demand()` |
| 2.4 | Market freeze signals | ⏳ | Не реалізовано |
| 2.5 | Tech skills demand chart | ⏳ | Не реалізовано |
| 2.6 | Remote work adoption trends | ⏳ | Не реалізовано |
| 2.7 | `market_snapshots` таблиця | ✅ | `migrations/20260416000000_add_market_snapshots.sql` |
| 2.8 | Web: Market Intelligence сторінка | ✅ | `web/src/pages/Market.tsx` |
| 2.9 | **Aggregation job** — заповнення `market_snapshots` | ✅ | `ingestion` refreshes snapshots after successful upserts; live readers still use `jobs` directly |

Детально: `docs/03-domain/market-intelligence.md`

---

## ФАЗА 3 — User engagement фічі

| # | Задача | Статус | Доказ / Примітка |
|---|--------|--------|------------------|
| 3.1 | Notifications: таблиця + API + UI (bell icon) | ✅ | `migrations/20260419000000_add_notifications.sql`, `api/routes/notifications.rs`, `web/src/pages/Notifications.tsx`, active shell badge wiring in `app-shell/useAppShell.ts`; ingestion trigger в `ingestion/src/db.rs` |
| 3.2 | Global search: PostgreSQL FTS + Cmd+K overlay | ✅ | `web/src/components/GlobalSearch.tsx` |
| 3.3 | CV Tailoring: endpoint + modal (адаптація CV під JD) | ❌ MISSING | Немає в `ml/app/enrichment_routes.py`; `cover_letter_draft` — окрема фіча |
| 3.4 | Settings сторінка: профіль + notifications prefs | ⚠️ PARTIAL | `/settings` route/page exists, but dedicated notification prefs are not implemented yet |
| 3.5 | Profile: years_of_experience + salary_range + languages | ✅ | Міграція + DTO + persistence + UI already live |
| 3.6 | Profile completion indicator (%) | ✅ | Completion status now exists in profile/settings surfaces |
| 3.7 | Ingestion stats widget в Analytics ("Last updated X min ago") | ❌ MISSING | Немає в `pages/Analytics.tsx` |

---

## ФАЗА 4 — ML та Ollama ✅ ЗДЕБІЛЬШОГО ЗАВЕРШЕНО

| # | Задача | Статус | Доказ |
|---|--------|--------|-------|
| 4.1 | Ollama provider в `llm_provider.py` | ✅ | `llm_provider_remote.py:172-282` — `OllamaEnrichmentProvider`, `/api/chat` + `format: json` |
| 4.2 | Retrain reranker після 30+ нових labeled examples | ✅ | `ml/app/bootstrap_training.py`; `scoring_routes.py:84` — bootstrap endpoint |
| 4.3 | Покращені шаблони enrichment (без LLM) | ✅ | `ml/app/llm_provider_template.py:11-739` — всі 6 методів |
| 4.4 | Sentence-transformers для semantic matching | ⏳ XL | Не реалізовано; low priority |

**Також реалізовано (не було в roadmap):**
- Всі 6 enrichment endpoints (`ml/app/enrichment_routes.py`)
- AI Analysis tab в JobDetails: fit explanation + cover letter + interview prep (`web/src/pages/JobDetails.tsx`)
- FeedbackCenter з undo controls (`web/src/pages/FeedbackCenter.tsx`)
- ApplicationBoard Kanban 5 колонок (`web/src/pages/ApplicationBoard.tsx`)
- Analytics сторінка: funnel, behavior, weekly guidance (`web/src/pages/Analytics.tsx`)
- Reranker endpoint `/api/v1/rerank` + bootstrap (`ml/app/scoring_routes.py`)
- Notifications inbox unread badge in the active app shell
- Global search wired in the active header shell

---

## ФАЗА 5 — Платна підписка

| # | Задача | Складність | Пріоритет |
|---|--------|------------|-----------|
| 5.1 | Auth (registration, login, JWT) | L | Критично |
| 5.2 | Multi-profile support | L | Критично |
| 5.3 | Stripe integration | L | Критично |
| 5.4 | Tier switching (free → paid LLM provider) | M | Висока |
| 5.5 | Claude Haiku / GPT-4o-mini для paid CV tailoring | M | Висока |
| 5.6 | Rate limiting по tier | M | Висока |
| 5.7 | Job alerts + email delivery (Resend) | M | Середнє |
