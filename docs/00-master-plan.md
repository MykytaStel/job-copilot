# Job Copilot — Master Plan

> Last updated: 2026-04-21

## 1. Що таке Job Copilot

Job Copilot — це не job board. Це **candidate intelligence system**:
- розуміє кандидата (профіль, навички, цілі)
- розуміє ринок (тренди, зарплати, компанії, попит)
- знаходить і ранжує вакансії під конкретну людину
- пояснює fit і gaps
- підтримує дії: зберегти, сховати, відправити, підготуватись
- навчається від результатів

## 2. Продуктові стовпи

### A. Розуміння кандидата
- CV/raw text → canonical profile
- Цілі та обмеження → search profile
- Дії користувача → поведінкові сигнали

### B. Розуміння ринку (Market Intelligence)
- Агрегація статистики з ingested jobs (без додаткових scrape)
- Яка компанія активно набирає / призупинила набір
- Тренди зарплат по ролях і senior/junior
- Попит на технології в реальному часі
- Географія та remote-тренди
- Детально: `docs/03-domain/market-intelligence.md`

### C. Matching та ранжування
- Детермінований scoring (Rust) — базис
- Поведінкові сигнали (save/hide/badfit) — коригування
- Навчений reranker — після накопичення даних
- Пояснення: matched terms, missing signals, reasons

### D. Action layer
- save / hide / bad fit / whitelist company / blacklist company
- applied / interviewing / offer / rejected
- CV tailoring / cover letter / interview prep

### E. Learning loop
- User events → training data → retrain reranker
- Bootstrap від перших saved/hidden
- Semantic matching — після накопичення достатньо даних

## 3. Архітектура

```
ingestion (Rust) ─→ PostgreSQL ←─ engine-api (Rust) ←─ web (React)
                                         ↕
                                   ml sidecar (Python)
```

- **engine-api**: canonical domain, API, validation, persistence
- **ingestion**: scrape → normalize → dedupe → lifecycle → DB
- **ml**: scoring, reranking, enrichment, market analytics, (Ollama LLM)
- **web**: dashboard, profile, ranked jobs, application board, market insights

## 4. AI стратегія (без платного API)

### Зараз (активно)
- Детермінований scoring в Rust — головний двигун
- Template enrichment provider в Python — структуровані пояснення без LLM (`ml/app/llm_provider_template.py`)
- **Ollama provider активний** — `OllamaEnrichmentProvider` є default (`ML_LLM_PROVIDER=ollama`); використовує `llama3.1:8b` через `/api/chat` з `format: json` (`ml/app/llm_provider_remote.py`)
- Bootstrap ML з реальних feedbacks юзера (`ml/app/bootstrap_training.py`)
- Всі 6 enrichment endpoints доступні: fit explanation, cover letter, interview prep, profile insights, coaching, weekly guidance

### Наступний крок
- Market snapshot aggregation job — наповнення `market_snapshots` реальними агрегатами
- CV Tailoring endpoint (адаптація CV під конкретну JD)
- Перемикання провайдера через `ML_LLM_PROVIDER` env var (template / ollama / openai)

### Платна підписка (майбутнє)
- Claude Haiku / GPT-4o-mini для CV tailoring, cover letter, interview prep
- Tier free → Ollama self-hosted, Tier paid → real API

Детально: `docs/04-development/ml-strategy.md`

## 5. Монетизація (план)

### Free tier
- Ingestion + matching + ranking (все детерміністичне)
- Market Intelligence (аналітика ринку)
- Application board (до 20 активних)
- Базові шаблонні пояснення

### Paid tier (~$10-15/міс)
- CV tailoring з реальним LLM
- Cover letter generation
- Interview preparation
- Необмежені applications
- Пріоритетний scraping (частіше оновлення)
- Export (PDF, CSV)

### Team/Enterprise (майбутнє)
- Multi-user
- HR-аналітика
- Власні джерела вакансій

## 6. Правила

- Domain truth — тільки в Rust, не в LLM і не у фронтенді
- LLM — шар збагачення, не джерело канонічної правди
- Canonical role catalog (`RoleId`) — не free-form strings
- Search filters — структуровані поля, не флаттений текст
- Prefer explicit DTOs, small services, testable helpers
- Не додавати broad abstractions без реального другого use-case

## 7. Sources (поточні)

| Source | Метод | Статус |
|--------|-------|--------|
| Djinni | HTML + detail page | ✅ Active |
| Work.ua | HTML + detail page | ✅ Active |
| Dou.ua | RSS feed | ✅ Active |
| Robota.ua | JSON API + detail page | ✅ Active |

Оновлення: кожні 60 хв (daemon), 3 сторінки на source.

## 8. Що заборонено

- Нові role IDs поза canonical catalog
- Обхід DTO validation
- Domain truth у фронтенді
- LLM output без Rust-side validation
- Broad abstractions без реального другого use-case
- Мок-дані там де можна взяти реальні
