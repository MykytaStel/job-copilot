# Current Focus — 2026-04-26

## Done

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
- **AI-agent workflow docs (PR #30)** — stabilized and merged: `CLAUDE.md`, `AGENTS.md`, `codex/CODEX.md`, Codex templates (`_template-implementation-slice.md`, `_template-review-diff.md`), security task prompt (`codex/tasks/security/profile-ownership-and-ml-token-slice.md`), Claude skill (`/.claude/skills/job-copilot-quality/SKILL.md`), and `docs/06-agents/ai-agent-operating-guide.md`

## Partially Done

- **Market Intelligence overall** — snapshot writer now refreshes `market_snapshots` after successful ingestion upserts, but current UI/API still read directly from `jobs`
- **ML provider defaulting** — runtime code defaults to `template`; Docker Compose defaults `ML_LLM_PROVIDER` to `ollama`

## Missing

- Profile ownership guards in `engine-api` (see next slices)
- ML internal token production validation (see next slices)
- CV Tailoring endpoint + web entrypoint
- Analytics freshness widget for ingestion recency
- Dedicated notification preferences (settings expansion)
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

| # | Task | Complexity | Priority |
|---|------|-----------|----------|
| 1 | **Security: profile ownership guards** — profile-scoped engine routes must reject mismatched owner with `403`; missing profile stays `404`; see `codex/tasks/security/profile-ownership-and-ml-token-slice.md` | S | High |
| 2 | **Security: ML internal token production validation** — production ML startup must fail fast without internal token; no token logging; see same task prompt | XS | High |
| 3 | **CV Tailoring** — ML endpoint + web modal (distinct from cover letter — focus on adapting CV to JD) | M | High |
| 4 | **Settings expansion** — dedicated notification prefs + profile preference controls beyond the current minimal route | M | Medium |
| 5 | **Analytics freshness widget** — ingestion recency and feed freshness in analytics | XS | Low |
| 6 | **Market snapshot readers** — gradually migrate market sections from live `jobs` queries to snapshots where beneficial | M | Low |

## Not Now

- Semantic embeddings / sentence-transformers — labeled data needed first
- Auth / multi-user (Phase 5) — product flow must stabilize first
- Stripe (Phase 5) — after auth
- Email delivery — after notifications progress
