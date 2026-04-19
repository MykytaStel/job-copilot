# Backlog (відкладено, але не забуто)

## Технічний борг
- PostgreSQL FTS5 index для global search (`GIN(to_tsvector(...))`)
- Ingestion stats endpoint (`GET /api/v1/admin/ingestion-stats`)
- `SCRAPE_INTERVAL_PEAK_MINUTES` — частіший scraping в пікові години
- Збільшити `SCRAPE_PAGES` до 5 для Djinni
- Profile completion % indicator
- Ingestion stats widget у Analytics ("Last updated X min ago")

## Домен
- Company resolution v2 (нормалізація назв компаній)
- Compensation normalization (USD/EUR/UAH уніфікація)
- Richer geographic model (city-level, не тільки country)
- Source trust score (якість source впливає на ranking)
- User feedback weighting (чим більше сигналів — тим сильніший вплив)
- Company notes (коментарі до компанії від юзера)

## Фічі
- Alerting / watchlists (email коли нова вакансія за критеріями)
- Timeline analytics (активність по тижнях)
- Export: PDF резюме + CSV application list
- Semantic embeddings + pgvector (після достатнього training set)
- Telegram bot integration (повернення фічі зі старого проекту)
- Job sharing / referral links

## ML
- Sentence-transformers (`all-MiniLM-L6-v2`) для semantic match
- A/B testing scoring variants
- Outcome tracking (чи отримали оффер після apply)
- Reranker v3 з більш складною моделлю (після 200+ labeled examples)
