# Current Focus — 2026-04-22

## ✅ Already Done

- **Active web shell** — runtime goes through `apps/web/src/App.tsx` -> `apps/web/src/AppShell.tsx`; `AppShellNew.tsx` is only a re-export alias
- **Sidebar profile** — real name/email is wired from profile query in the active app shell
- **Query invalidation** — TanStack invalidation is in place after feedback/profile changes
- **Canonical role catalog** — `RoleId` enum + aliases are live in Rust matching and related flows
- **Bootstrap training data** — bootstrap retraining flow exists in `ml/app/bootstrap_training.py`
- **Freshness decay** — deterministic search scoring decays older jobs
- **Notifications** — DB table + API endpoints + web inbox + unread badge + ingestion trigger
- **Global search** — Cmd/Ctrl+K overlay is implemented
- **Profile compensation + languages** — schema, API, persistence, and web UI are live
- **Market base page** — overview, company activity, salary trends, and role demand are live
- **Search profile persistence** — `search_preferences` now persist on profiles; the structured search profile still rebuilds on demand
- **Lifecycle presentation semantics** — jobs now expose explicit lifecycle labels for posted/seen, last confirmed active, inactive since, and reactivated states
- **Dashboard rerank throttling** — ranked mode now reranks on demand and only over a bounded first window instead of eager-ranking the full feed

## ⚠️ Partially Done

- **Market Intelligence overall** — snapshot writer now refreshes `market_snapshots` after successful ingestion upserts, but current UI/API still read directly from `jobs`
- **ML provider defaulting** — runtime code defaults to `template`; Docker Compose defaults `ML_LLM_PROVIDER` to `ollama`

## ❌ Missing

- CV Tailoring endpoint + web entrypoint
- Analytics freshness widget for ingestion recency
- Additional market sections from the broader spec:
  - freeze signals
  - tech skills demand
  - remote adoption trends

## Current Runtime Notes

- Active shell source of truth: `apps/web/src/AppShell.tsx`
- Legacy alias only: `apps/web/src/AppShellNew.tsx`
- Notifications, market base sections, profile compensation/languages, and global search should no longer be tracked as open feature work
- `market_snapshots` is now refreshed by ingestion after successful upserts
- Settings now has a minimal route/page, but dedicated notification preferences are still not implemented
- Profile completion indicator now exists in the profile/settings surfaces
- Lifecycle-heavy UI surfaces should read engine presentation labels instead of inferring state from `postedAt` alone
- PostgreSQL extension guidance for self-hosted PG16 lives in `docs/04-development/postgres-extensions.md`
- Verification matrix lives in `docs/04-development/verification-matrix.md`
- ADR template lives in `docs/02-architecture/adr-template.md`

## Recommended Next Slices

| # | Задача | Складність | Пріоритет |
|---|--------|------------|-----------|
| A | **CV Tailoring** — ML endpoint + web modal (відмінно від cover letter — фокус на адаптації CV під JD) | M | Висока |
| B | **Settings expansion** — dedicated notification prefs + profile preference controls beyond the current read-only route | M | Висока |
| C | **Analytics freshness widget** — ingestion recency and feed freshness in analytics | XS | Низьке |
| D | **Market snapshot readers** — якщо потрібно, поступово переводити market sections з live `jobs` queries на snapshots | M | Низьке |

## Not Now

- Semantic embeddings / sentence-transformers (Phase 4.4) — потрібні дані спочатку
- Auth / multi-user (Phase 5) — потрібна монетизація спочатку
- Stripe (Phase 5) — після auth
- Email delivery — після notifications прогресу
