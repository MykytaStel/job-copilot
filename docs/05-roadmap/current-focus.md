# Current Focus — 2026-07-17

## Done

- **Active web shell** — runtime goes through `apps/web/src/App.tsx` -> `apps/web/src/AppShell.tsx`; `AppShellNew.tsx` is only a re-export alias
- **Sidebar profile** — real name/email is wired from profile query in the active app shell
- **Query invalidation** — TanStack invalidation is in place after feedback/profile changes
- **Canonical role catalog** — `RoleId` enum + aliases are live in Rust matching and related flows
- **Bootstrap training data** — bootstrap retraining flow exists in `ml/app/bootstrap_training.py`
- **Freshness decay** — deterministic search scoring decays older jobs
- **Notifications** — DB table + API endpoints + web inbox + unread badge + ingestion trigger + task due reminders
- **Global search** — Cmd/Ctrl+K overlay is implemented
- **Profile compensation + languages** — schema, API, persistence, and web UI are live
- **Market base page** — overview, company activity, salary trends, and role demand are live
- **Search profile persistence** — `search_preferences` now persist on profiles; the structured search profile still rebuilds on demand
- **Lifecycle presentation semantics** — jobs expose explicit lifecycle labels for posted/seen,
  last confirmed active, inactive since, and reactivated states
- **Dashboard rerank throttling** — ranked mode reranks on demand over a bounded first window
  instead of eager-ranking the full feed
- **AI-agent workflow docs (PR #30)** — stabilized and merged: `CLAUDE.md`, `AGENTS.md`,
  `codex/CODEX.md`, Codex templates, security task prompt, Claude skill,
  and `docs/06-agents/ai-agent-operating-guide.md`
- **CV Tailoring web entrypoint** — job details exposes the engine + ML tailoring flow
  in an interactive panel
- **Notification preferences** — dedicated profile-scoped controls persist through engine-api
- **Analytics freshness widget** — analytics shows per-source run status, recency, counts,
  failures, and the next scheduled refresh
- **Market snapshot readers** — overview, companies, salary trends, and role demand prefer
  fresh snapshots and fall back to live jobs when a usable snapshot is unavailable
- **Additional market sections** — freeze signals, region demand, and technology demand are live
- **Active scraper set** — Djinni, DOU, and Work.ua are verified in the daemon; Robota.ua
  is explicit opt-in while its machine endpoints require a Cloudflare browser challenge
- **Remote adoption trends** — ingestion writes an eight-week snapshot by source and normalized
  work mode; engine-api validates it with a bounded live fallback and Market UI keeps unknown
  values visible
- **UI stabilization pass** — dashboard renders 20 jobs at a time with visible-only impression
  tracking, Analytics enrichment sends valid JSON through the Rust gateway, Market cards no longer
  stretch to sibling height, and mobile Settings uses an explicit section selector
- **UI foundation pass** — shell/page gutters no longer stack, typography and muted-text contrast
  are clearer, shared cards use distinct elevated surfaces with normalized radii/shadows, and the
  Dashboard hero is materially shorter on mobile

## Partially Done

- **Market Intelligence overall** — snapshot writer refreshes `market_snapshots` after successful
  ingestion upserts; core readers and remote adoption are snapshot-backed, while newer sections
  listed below still read directly from `jobs`

## Missing

- Snapshot-backed readers for the newer freeze, region, and technology sections

## Current Runtime Notes

- Active shell source of truth: `apps/web/src/AppShell.tsx`
- Legacy alias only: `apps/web/src/AppShellNew.tsx`
- Notifications, task due reminders, market base sections, profile compensation/languages, and global search should no longer be tracked as open feature work
- `market_snapshots` is now refreshed by ingestion after successful upserts
- Legacy profile role deserialization is scheduled for removal by 2026-07-01; tracking details live in `docs/05-roadmap/backlog.md` under "Remove legacy profile role deserialization by 2026-07-01"
- Settings has profile, search, notification, display, and privacy sections
- Profile completion indicator now exists in the profile/settings surfaces
- Lifecycle-heavy UI surfaces should read engine presentation labels instead of inferring state from `postedAt` alone
- PostgreSQL extension guidance for self-hosted PG16 lives in `docs/04-development/postgres-extensions.md`
- Verification matrix lives in `docs/04-development/verification-matrix.md`
- ADR template lives in `docs/02-architecture/adr-template.md`

## Recommended Next Slices

1. **Complete market snapshot coverage**
   - Move freeze, region, and technology readers to their existing snapshot payloads with
     the same bounded live fallback used by the core market readers.
   - Complexity: S, Priority: Low.

2. **Scraper source observability**
   - Surface selector failures and repeated partial runs as an operator-visible warning.
   - Complexity: S, Priority: Medium.

3. **Dense-page information architecture and language consistency**
   - After the shared UI foundation pass, collapse or progressively disclose secondary sections on
     Dashboard, Analytics, Market, and Profile, then normalize mixed Ukrainian/English labels without
     changing domain meaning.
   - Complexity: M, Priority: High.

4. **Robota.ua authorized access**
   - Replace the challenged legacy REST call with an authorized machine contract when
     Robota.ua provides credentials or a supported feed.
   - Complexity: M, Priority: Medium.

## Not Now

- Semantic embeddings / sentence-transformers — labeled data needed first
- Auth / multi-user (Phase 5) — product flow must stabilize first
- Stripe (Phase 5) — after auth
- Email delivery — after notifications progress
