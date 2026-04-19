# Current Focus — 2026-04-18

## Зараз робимо (Фаза 1)

### Критичні фікси
1. **Sidebar profile** — показати реальне ім'я/email з API замість hardcoded тексту
   - Файл: `apps/web/src/AppShellNew.tsx:253`

2. **Query invalidation** — після save/hide/badFit інвалідувати ML rerank ключі; після profile save — downstream
   - Файли: `apps/web/src/pages/Dashboard.tsx`, `useProfilePage.ts`

3. **Canonical role catalog** — `RoleId` enum, display_name, aliases, підключити до scoring
   - Файл: `apps/engine-api/src/domain/role/catalog.rs` (новий)

### ML без платного API
4. **Bootstrap training data** — скрипт що генерує labeled examples з `user_events` (save=positive, hide=negative, badfit=strong negative)
   - Файл: `apps/ml/app/bootstrap_training.py` (новий)

5. **Виправити LLM provider** — `gpt-5.4-mini` → правильний model name, підготувати Ollama path
   - Файл: `apps/ml/app/llm_provider.py`

### Покращення scoring
6. **Freshness decay** — jobs > 14 днів зменшувати score поступово
   - Файл: `apps/engine-api/src/services/matching.rs:152`

## Наступне після фази 1

- Market Intelligence (Фаза 2) — агрегатна аналітика ринку з наявних даних
- Notifications (Фаза 3.1)
- CV Tailoring (Фаза 3.3)

## Не робимо зараз

- Semantic embeddings (потрібні дані спочатку)
- Auth / multi-user (потрібна монетизація спочатку)
- Email delivery (потрібні notifications спочатку)
- Будь-який paid LLM API (Ollama спочатку)
