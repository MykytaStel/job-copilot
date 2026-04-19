# Milestones

## M1 — Real Data Everywhere (поточний)
- Sidebar показує реальний профіль
- Role catalog стабілізований
- Query invalidation виправлений
- LLM provider виправлений (навіть якщо template)
- Bootstrap ML training data pipeline

## M2 — Market Intelligence
- `market_snapshots` таблиця з daily aggregations
- API endpoints: company stats, salary trends, role demand, market freeze signals
- Web: Market Intelligence сторінка з charts
- Перший диференціатор від звичайних job boards

## M3 — Scoring v2
- Freshness decay
- Salary fit в scoring
- Company reputation в scoring
- Reranker retrained на реальних даних (30+ labeled examples)

## M4 — User Engagement
- Notifications: in-app bell + список
- Global search (FTS, Cmd+K)
- CV Tailoring (Ollama або шаблон)
- Settings сторінка
- Profile: years_of_experience, salary_range, languages

## M5 — Ollama Self-Hosted AI
- OllamaEnrichmentProvider
- CV tailoring, cover letter, interview prep через локальну модель
- Конфігурація через `ML_LLM_PROVIDER=ollama` + `OLLAMA_BASE_URL`

## M6 — Paid Subscription
- Auth (JWT)
- Multi-profile
- Stripe
- Tier switching (free Ollama → paid Claude/GPT)
- Rate limiting
- Job alerts + email (Resend)
