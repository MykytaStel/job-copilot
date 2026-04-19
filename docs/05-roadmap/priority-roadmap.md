# Priority Roadmap — 2026-04-18

## Принципи
- Тільки реальні дані, ніяких моків там де є реальне джерело
- LLM — тільки після того як Ollama налаштований або платна підписка активна
- Фіксити видимі баги до нових фічей
- Market Intelligence будується з даних що вже є (без нових джерел)

---

## ФАЗА 1 — Основи та реальні дані (поточна)

Мета: зробити все що є — стабільним і без hardcoded/mock значень.

| # | Задача | Складність | Пріоритет |
|---|--------|------------|-----------|
| 1.1 | Sidebar: profile name/email замість hardcoded | XS | Критично |
| 1.2 | Query invalidation: rerank/analytics після feedback | S | Критично |
| 1.3 | Canonical role catalog (`RoleId` enum + aliases) | M | Критично |
| 1.4 | Виправити LLM provider (model name + Anthropic/Ollama path) | S | Важливо |
| 1.5 | Bootstrap training data з user_events | S | Важливо |
| 1.6 | Freshness decay у scoring (jobs > 14 днів) | S | Важливо |
| 1.7 | Salary fit у scoring (якщо profile має salary range) | S | Середнє |
| 1.8 | Company reputation у scoring (whitelist +5, blacklist -20) | XS | Середнє |

---

## ФАЗА 2 — Market Intelligence (новий диференціатор)

Мета: перший в UA job copilot з ринковою аналітикою.

| # | Задача | Складність | Пріоритет |
|---|--------|------------|-----------|
| 2.1 | Company hiring velocity (jobs_count per company over time) | M | Висока |
| 2.2 | Salary trends по role/seniority | M | Висока |
| 2.3 | Role demand index (які ролі зростають/падають) | M | Висока |
| 2.4 | Market freeze signals (компанії що зупинились) | M | Середнє |
| 2.5 | Tech skills demand chart (top N skills по тижнях) | S | Середнє |
| 2.6 | Remote work adoption trends | S | Низьке |
| 2.7 | `market_snapshots` таблиця (daily aggregations) | M | Критично для 2.x |
| 2.8 | Web: Market Intelligence сторінка | L | Висока |

Детально: `docs/03-domain/market-intelligence.md`

---

## ФАЗА 3 — User engagement фічі

Мета: дати юзеру причину повертатися щодня.

| # | Задача | Складність | Пріоритет |
|---|--------|------------|-----------|
| 3.1 | Notifications: таблиця + API + UI (bell icon) | L | Висока |
| 3.2 | Global search: PostgreSQL FTS + Cmd+K overlay | M | Середнє |
| 3.3 | CV Tailoring: endpoint + modal (шаблон → Ollama) | M | Висока |
| 3.4 | Settings сторінка: профіль + notifications prefs | M | Середнє |
| 3.5 | Profile: years_of_experience + salary_range + languages | S | Середнє |
| 3.6 | Profile completion indicator (%) | XS | Низьке |
| 3.7 | Ingestion stats widget в Analytics ("Last updated X min ago") | S | Низьке |

---

## ФАЗА 4 — ML та Ollama

Мета: власний AI без платного API.

| # | Задача | Складність | Пріоритет |
|---|--------|------------|-----------|
| 4.1 | Ollama provider в `llm_provider.py` | M | Висока |
| 4.2 | Retrain reranker після 30+ нових labeled examples | S | Висока |
| 4.3 | Покращені шаблони enrichment (без LLM, краще ніж зараз) | M | Середнє |
| 4.4 | Sentence-transformers для semantic matching | XL | Низьке |

---

## ФАЗА 5 — Платна підписка

Мета: монетизація.

| # | Задача | Складність | Пріоритет |
|---|--------|------------|-----------|
| 5.1 | Auth (registration, login, JWT) | L | Критично |
| 5.2 | Multi-profile support | L | Критично |
| 5.3 | Stripe integration | L | Критично |
| 5.4 | Tier switching (free → paid LLM provider) | M | Висока |
| 5.5 | Claude Haiku / GPT-4o-mini для paid CV tailoring | M | Висока |
| 5.6 | Rate limiting по tier | M | Висока |
| 5.7 | Job alerts + email delivery (Resend) | M | Середнє |
