# Block A — Stability & Small Fixes (18 tasks)

Each task below is a self-contained Codex prompt. Copy the section for the task you want to implement.

---

## A1 — ML provider default consistency

```
You are working in the Job Copilot monorepo.
First read: CLAUDE.md, apps/ml/app/settings.py, infra/docker-compose.yml

## Goal
Ensure ML_LLM_PROVIDER defaults to "template" everywhere — both in Python settings
and Docker Compose. Currently settings.py defaults to "template" but docker-compose.yml
sets ML_LLM_PROVIDER=ollama. Align docker-compose to use "template" as default,
with a comment explaining how to switch to ollama.

## Scope
Only config/env changes. No logic changes.

## Inspect first
- apps/ml/app/settings.py — current default value
- infra/docker-compose.yml — ML service env section
- apps/ml/app/llm_provider_factory.py — how provider is selected

## Likely files to modify
- infra/docker-compose.yml (ML_LLM_PROVIDER value + comment)

## Files not allowed to modify
- apps/ml/app/settings.py (default is already correct)
- Any route or service file

## Acceptance criteria
- [ ] docker-compose.yml ML service has ML_LLM_PROVIDER=template
- [ ] Comment above it explains: "change to 'ollama' if Ollama is running locally"
- [ ] No logic changes

## Verification commands
grep -n "ML_LLM_PROVIDER" infra/docker-compose.yml apps/ml/app/settings.py

## Final response format
## What changed
## Why
## Files changed
## Verification
## Notes / follow-ups
```

---

## A2 — Fix LLM model name

```
You are working in the Job Copilot monorepo.
First read: CLAUDE.md, apps/ml/app/settings.py, apps/ml/app/llm_provider_factory.py

## Goal
Find any reference to the non-existent model name "gpt-5.4-mini" (or similarly invalid names)
and replace with "gpt-4o-mini". Also add a runtime warning if ML_LLM_PROVIDER=openai
but OPENAI_API_KEY is empty.

## Inspect first
- apps/ml/app/settings.py — model name field
- apps/ml/app/llm_provider_factory.py — where model name is used
- apps/ml/app/providers/ (if exists) — OpenAI provider implementation

## Likely files to modify
- apps/ml/app/settings.py
- apps/ml/app/llm_provider_factory.py or OpenAI provider file

## Rules
- Do not add paid API calls to any default path.
- Warning log only — do not raise at startup if provider != openai.

## Acceptance criteria
- [ ] No reference to "gpt-5.4-mini" anywhere in codebase
- [ ] Default OpenAI model is "gpt-4o-mini"
- [ ] If ML_LLM_PROVIDER=openai and OPENAI_API_KEY is empty → log warning at startup
- [ ] Template and Ollama paths unaffected

## Verification commands
grep -rn "gpt-5" apps/ml/
cd apps/ml && python -m pytest

## Final response format
## What changed
## Why
## Files changed
## Verification
## Notes / follow-ups
```

---

## A3 — ML rerank cache invalidation

```
You are working in the Job Copilot monorepo.
First read: CLAUDE.md, apps/ml/app/scoring_routes.py, apps/web/src/api/jobs.ts,
apps/web/src/api/feedback.ts

## Goal
The ML /api/v1/rerank endpoint caches results for 5 minutes. After the user
saves/hides/marks bad-fit a job, the next feed refresh should not return stale rankings.
Two fixes needed:
1. ML side: expose a cache-bust mechanism (e.g. accept a cache_bust boolean in rerank request,
   or add a POST /api/v1/rerank/invalidate endpoint for a given profile_id).
2. Web side: after any feedback mutation (save/hide/bad-fit), include cache_bust=true
   in the next rerank call OR call the invalidate endpoint.

## Inspect first
- apps/ml/app/scoring_routes.py — rerank endpoint and cache logic
- apps/web/src/api/jobs.ts — how rerank is called from web
- apps/web/src/api/feedback.ts — save/hide/bad-fit mutations

## Likely files to modify
- apps/ml/app/scoring_routes.py
- apps/web/src/api/jobs.ts (or wherever rerank is triggered)
- apps/web/src/api/feedback.ts (add cache bust after mutation)

## Rules
- Do not change scoring logic.
- Cache invalidation should be profile-scoped.
- Keep changes minimal.

## Acceptance criteria
- [ ] After save/hide/bad-fit, next feed call bypasses 5min cache
- [ ] Cache still works for normal browsing (no bust)
- [ ] ML tests pass
- [ ] Web typecheck passes

## Verification commands
cd apps/ml && python -m pytest
pnpm --dir apps/web typecheck

## Final response format
## What changed
## Why
## Files changed
## Verification
## Notes / follow-ups
```

---

## A4 — Sidebar shows real profile name/email

```
You are working in the Job Copilot monorepo.
First read: CLAUDE.md, apps/web/src/AppShell.tsx, apps/web/src/api/profiles/

## Goal
The sidebar currently shows hardcoded placeholder text instead of the real profile
name and email. Wire it to the actual profile data from the API.

## Inspect first
- apps/web/src/AppShell.tsx — find the sidebar user section (search for hardcoded name/email)
- apps/web/src/api/profiles/ — existing profile fetch hooks/queries
- apps/web/src/AppShellNew.tsx — check if it's a re-export or has its own logic

## Likely files to modify
- apps/web/src/AppShell.tsx

## Rules
- Use existing React Query hooks for profile data — do not add new API calls.
- Show loading skeleton while profile loads, not empty string.
- Do not render raw UUIDs in visible text.

## Acceptance criteria
- [ ] Sidebar shows real profile name (or "Your Profile" if name empty)
- [ ] Sidebar shows real email (or blank if not set)
- [ ] Shows skeleton or placeholder during loading
- [ ] No hardcoded strings remain

## Verification commands
pnpm --dir apps/web typecheck
pnpm --dir apps/web lint

## Final response format
## What changed
## Why
## Files changed
## Verification
## Notes / follow-ups
```

---

## A5 — React Query invalidation cascade after feedback

```
You are working in the Job Copilot monorepo.
First read: CLAUDE.md, apps/web/src/api/feedback.ts, apps/web/src/pages/Dashboard.tsx (or
wherever job feed is rendered), apps/web/src/api/jobs.ts

## Goal
After the user saves/hides/marks bad-fit a job, the job feed and any ranked list should
refresh. Currently the feed may show stale state (job still appears as unhidden, etc.).
Use React Query's queryClient.invalidateQueries to invalidate the relevant feed/search
query keys after each feedback mutation success.

## Inspect first
- apps/web/src/api/feedback.ts — mutation hooks (save, hide, bad-fit)
- apps/web/src/api/jobs.ts — query keys used for job feed
- apps/web/src/pages/Dashboard.tsx — how feed is rendered and keyed

## Likely files to modify
- apps/web/src/api/feedback.ts (add onSuccess invalidation)

## Rules
- Only invalidate queries that are actually affected.
- Do not refetch unrelated queries.
- Match existing mutation pattern in the file.

## Acceptance criteria
- [ ] After save → job feed re-fetches
- [ ] After hide → job disappears from feed on next render
- [ ] After bad-fit → same as hide
- [ ] No unnecessary re-fetches for unrelated queries

## Verification commands
pnpm --dir apps/web typecheck
pnpm --dir apps/web test

## Final response format
## What changed
## Why
## Files changed
## Verification
## Notes / follow-ups
```

---

## A6 — Notifications page wired to real data

```
You are working in the Job Copilot monorepo.
First read: CLAUDE.md, apps/web/src/pages/Notifications.tsx (if exists),
apps/web/src/api/notifications.ts, apps/engine-api/src/api/routes/notifications.rs

## Goal
The /notifications route exists in the web app but the page may be empty or unimplemented.
Wire it to GET /api/v1/notifications (list) and PATCH /api/v1/notifications/:id/read
(mark read). Show a list of notification cards with timestamp, title, body.
Add "Mark all read" button.

## Inspect first
- apps/web/src/pages/Notifications.tsx — current state
- apps/web/src/api/notifications.ts — existing API client (if any)
- apps/engine-api/src/api/routes/notifications.rs — response shape
- apps/web/src/App.tsx — confirm /notifications route exists

## Likely files to modify
- apps/web/src/pages/Notifications.tsx
- apps/web/src/api/notifications.ts (create or extend)

## Rules
- Match existing page/card visual style (dark base, operator-focused).
- Use React Query for data fetching.
- Do not add a new backend endpoint — use what exists.

## Acceptance criteria
- [ ] /notifications shows list of notifications from API
- [ ] Each notification shows title, body, timestamp, read/unread state
- [ ] "Mark all read" button calls the batch-read endpoint (or loops individual reads)
- [ ] Empty state when no notifications
- [ ] Unread count badge in sidebar updates after mark-read

## Verification commands
pnpm --dir apps/web typecheck
pnpm --dir apps/web lint

## Final response format
## What changed
## Why
## Files changed
## Verification
## Notes / follow-ups
```

---

## A7 — Settings page structure

```
You are working in the Job Copilot monorepo.
First read: CLAUDE.md, apps/web/src/pages/Settings.tsx (if exists),
apps/web/src/App.tsx (route registration)

## Goal
Create a proper Settings page with a sectioned layout. Sections needed (content can be
placeholder for now):
1. Profile & Account
2. Search Preferences
3. Notifications
4. Display
5. Data & Privacy

Use a sidebar-nav + content-panel layout pattern (like most settings UIs).

## Inspect first
- apps/web/src/pages/Settings.tsx — current state
- apps/web/src/App.tsx — route definition
- apps/web/src/pages/Profile.tsx — style reference for form layout
- apps/web/src/AppShell.tsx — how other pages are structured

## Likely files to modify
- apps/web/src/pages/Settings.tsx

## Rules
- Match dark operator-focused visual style.
- Each section is a distinct component (SettingsSection or inline).
- No backend calls yet for placeholder sections.
- Do not mix with profile editing logic.

## Acceptance criteria
- [ ] /settings loads without error
- [ ] 5 sections visible with headings
- [ ] Active section highlighted in nav
- [ ] Responsive layout (stacks on mobile)

## Verification commands
pnpm --dir apps/web typecheck
pnpm --dir apps/web lint

## Final response format
## What changed
## Why
## Files changed
## Verification
## Notes / follow-ups
```

---

## A8 — Settings: display preferences

```
You are working in the Job Copilot monorepo.
First read: CLAUDE.md, apps/web/src/pages/Settings.tsx, apps/web/src/AppShell.tsx

## Goal
Add a "Display" section in Settings with:
- Job card density toggle: Compact / Normal / Comfortable
- Default job sort: Relevance / Date / Salary
These preferences should persist in localStorage. Apply density class to the job feed.

## Inspect first
- apps/web/src/pages/Settings.tsx — Settings structure
- apps/web/src/pages/Dashboard.tsx — where job cards are rendered (apply density)
- apps/web/src/api/jobs.ts — where sort param is sent

## Likely files to modify
- apps/web/src/pages/Settings.tsx (Display section)
- apps/web/src/pages/Dashboard.tsx (read density from localStorage)
- apps/web/src/api/jobs.ts (read sort from localStorage as default)

## Rules
- localStorage only — no backend call needed.
- Use a simple context or direct localStorage reads.
- Do not change the visual style of cards themselves.

## Acceptance criteria
- [ ] Density toggle saves to localStorage and applies immediately
- [ ] Sort preference saves to localStorage and is used on next feed load
- [ ] Preference survives page refresh

## Verification commands
pnpm --dir apps/web typecheck
pnpm --dir apps/web test

## Final response format
## What changed
## Why
## Files changed
## Verification
## Notes / follow-ups
```

---

## A9 — Settings: persist last search filters

```
You are working in the Job Copilot monorepo.
First read: CLAUDE.md, apps/web/src/pages/Dashboard.tsx,
apps/web/src/api/jobs.ts, apps/web/src/api/search.ts (if exists)

## Goal
When the user applies filters on the Dashboard (role, remote, salary, source), save them
to localStorage. On next visit, restore those filters as the starting state.

## Inspect first
- apps/web/src/pages/Dashboard.tsx — filter state management
- Filter component files (wherever filter state lives)
- apps/web/src/api/jobs.ts — filter params shape

## Likely files to modify
- apps/web/src/pages/Dashboard.tsx or filter component
- A new small hook: usePersistedFilters (optional)

## Rules
- localStorage only. No backend call.
- Do not override URL query params if they exist (URL params take precedence).
- Keep filter state shape stable.

## Acceptance criteria
- [ ] Filters survive page refresh
- [ ] Clearing filters also clears localStorage
- [ ] No interference with URL-based filter sharing

## Verification commands
pnpm --dir apps/web typecheck

## Final response format
## What changed
## Why
## Files changed
## Verification
## Notes / follow-ups
```

---

## A10 — Analytics freshness widget

```
You are working in the Job Copilot monorepo.
First read: CLAUDE.md, apps/web/src/pages/Analytics.tsx,
apps/engine-api/src/api/routes/health.rs (or similar),
apps/engine-api/src/api/routes/sources.rs

## Goal
Show a "Last ingested X minutes ago" widget in the Analytics page header.
The widget reads the most recent job ingest timestamp. If no ingestion stats endpoint
exists, add a lightweight one: GET /api/v1/ingestion/stats returning
{ last_ingested_at: DateTime, total_jobs: u32, active_jobs: u32 }.

## Inspect first
- apps/engine-api/src/api/routes/ — check if ingestion stats endpoint exists
- apps/engine-api/src/db/ or services/ — query for max(last_seen_at) or similar
- apps/web/src/pages/Analytics.tsx — where to place the widget

## Likely files to modify
- apps/engine-api/src/api/routes/ (new route file or add to health.rs)
- apps/engine-api/src/api/routes/mod.rs (register new route if added)
- apps/web/src/api/ (new ingestion stats client function)
- apps/web/src/pages/Analytics.tsx (add widget)

## Rules
- The endpoint must be auth-protected.
- Query must be fast (single MAX aggregate, no full scan).
- Widget is read-only display — no actions.

## Acceptance criteria
- [ ] GET /api/v1/ingestion/stats returns last_ingested_at, total_jobs, active_jobs
- [ ] Analytics header shows "Last updated X min ago" (relative time)
- [ ] Shows "Never" if no jobs ingested yet
- [ ] Cargo tests pass, web typecheck passes

## Verification commands
cargo test --manifest-path apps/engine-api/Cargo.toml
cargo check --manifest-path apps/engine-api/Cargo.toml
pnpm --dir apps/web typecheck

## Final response format
## What changed
## Why
## Files changed
## Verification
## Notes / follow-ups
```

---

## A11 — Ingestion stats API endpoint

```
(Covered by A10 — see A10 for the backend endpoint. This task is the standalone version
if A10 was split: implement only the backend endpoint without the web widget.)

You are working in the Job Copilot monorepo.
First read: CLAUDE.md, apps/engine-api/src/api/routes/,
apps/engine-api/src/db/ (jobs table schema)

## Goal
Add GET /api/v1/ingestion/stats endpoint that returns:
{ last_ingested_at: Option<DateTime>, total_jobs: u32, active_jobs: u32,
  inactive_jobs: u32, sources: [{ source: String, count: u32, last_seen: DateTime }] }
This is a single-query aggregation over the jobs table.

## Inspect first
- apps/engine-api/src/api/routes/health.rs — pattern for simple endpoints
- apps/engine-api/migrations/ — jobs table schema
- apps/engine-api/src/db/ — existing job queries

## Likely files to modify
- apps/engine-api/src/api/routes/ (new file: ingestion.rs or stats.rs)
- apps/engine-api/src/api/routes/mod.rs (register route)

## Rules
- Auth-protected (require valid JWT).
- Single SQL query with GROUP BY source.
- No new DB tables or migrations.

## Acceptance criteria
- [ ] Endpoint returns correct aggregates
- [ ] Unit test with mock data
- [ ] Cargo check passes

## Verification commands
cargo test --manifest-path apps/engine-api/Cargo.toml
cargo check --manifest-path apps/engine-api/Cargo.toml

## Final response format
## What changed
## Why
## Files changed
## Verification
## Notes / follow-ups
```

---

## A12 — Reranker bootstrap trigger in UI

```
You are working in the Job Copilot monorepo.
First read: CLAUDE.md, apps/web/src/pages/Analytics.tsx (or Profile.tsx),
apps/web/src/api/ (bootstrap endpoint client if exists),
apps/ml/app/scoring_routes.py (bootstrap endpoint)

## Goal
Add a "Retrain Model" button in the Analytics or Profile page that calls
POST /ml/api/v1/reranker/bootstrap. Show progress by polling
GET /ml/api/v1/reranker/bootstrap/{task_id}. Display status: idle / running / done / error.

## Inspect first
- apps/ml/app/scoring_routes.py — bootstrap endpoint request/response shape
- apps/web/src/api/ — existing mlRequest() client
- apps/web/src/pages/Analytics.tsx — where to place the button

## Likely files to modify
- apps/web/src/api/enrichment/ or a new apps/web/src/api/reranker.ts
- apps/web/src/pages/Analytics.tsx (add Retrain button + status)

## Rules
- Poll every 3 seconds while status is "running".
- Show clear feedback: spinner during run, checkmark on done, error message on failure.
- Do not block the page — button triggers background task.

## Acceptance criteria
- [ ] "Retrain Model" button visible in Analytics
- [ ] POST /ml/api/v1/reranker/bootstrap called on click
- [ ] Status polled and displayed
- [ ] Button disabled while task is running
- [ ] Success/error state shown after completion

## Verification commands
pnpm --dir apps/web typecheck

## Final response format
## What changed
## Why
## Files changed
## Verification
## Notes / follow-ups
```

---

## A13 — Market page: read from snapshots instead of live jobs

```
You are working in the Job Copilot monorepo.
First read: CLAUDE.md, apps/engine-api/src/api/routes/market.rs,
apps/engine-api/migrations/ (market_snapshots table),
docs/03-domain/market-intelligence.md (if exists)

## Goal
The market.rs routes currently read directly from the jobs table. Switch the read path
to use the market_snapshots table (which is already populated by ingestion).
If a snapshot is older than 24h, fall back to live jobs with a warning header.

## Inspect first
- apps/engine-api/src/api/routes/market.rs — current queries
- apps/engine-api/migrations/ — market_snapshots schema
- apps/engine-api/src/db/ — any existing snapshot read functions

## Likely files to modify
- apps/engine-api/src/api/routes/market.rs
- apps/engine-api/src/db/ (new or extended market read functions)

## Rules
- Do not change the API response shape (no contract break).
- Fallback to live jobs is acceptable but must log a warning.
- No new migrations.

## Acceptance criteria
- [ ] GET /api/v1/market/* reads from market_snapshots when available
- [ ] Falls back to live jobs if snapshot is stale (>24h)
- [ ] Response shape unchanged
- [ ] Cargo tests pass

## Verification commands
cargo test --manifest-path apps/engine-api/Cargo.toml
cargo check --manifest-path apps/engine-api/Cargo.toml

## Final response format
## What changed
## Why
## Files changed
## Verification
## Notes / follow-ups
```

---

## A14 — React error boundaries per page

```
You are working in the Job Copilot monorepo.
First read: CLAUDE.md, apps/web/src/App.tsx, apps/web/src/AppShell.tsx

## Goal
Wrap each route-level page component in a React error boundary so a crash in one page
does not destroy the whole app. Show a minimal "Something went wrong. Reload page."
fallback UI.

## Inspect first
- apps/web/src/App.tsx — route definitions
- apps/web/src/components/ — check if ErrorBoundary component exists

## Likely files to modify
- apps/web/src/components/ErrorBoundary.tsx (create if missing)
- apps/web/src/App.tsx (wrap each lazy/route component)

## Rules
- Use React class component for ErrorBoundary (required for componentDidCatch).
- Fallback UI must match dark theme.
- Do not suppress errors in development.

## Acceptance criteria
- [ ] ErrorBoundary component exists
- [ ] Each page route is wrapped
- [ ] A thrown error in one page shows fallback, not white screen
- [ ] Typecheck passes

## Verification commands
pnpm --dir apps/web typecheck

## Final response format
## What changed
## Why
## Files changed
## Verification
## Notes / follow-ups
```

---

## A15 — Empty states for feed / applications / feedback

```
You are working in the Job Copilot monorepo.
First read: CLAUDE.md, apps/web/src/pages/Dashboard.tsx,
apps/web/src/pages/Applications.tsx, apps/web/src/pages/Feedback.tsx

## Goal
When a list is empty (no jobs, no applications, no feedback), show a meaningful empty
state instead of a blank area. Each empty state should have:
- An icon (use existing icon library)
- A short message ("No jobs found. Try adjusting your filters.")
- Optional action button ("Add filters" / "Find jobs" link)

## Inspect first
- apps/web/src/pages/Dashboard.tsx — where job list is rendered
- apps/web/src/pages/Applications.tsx — application board empty state
- apps/web/src/pages/Feedback.tsx — feedback list empty state
- apps/web/src/components/ — check for existing EmptyState component

## Likely files to modify
- apps/web/src/components/EmptyState.tsx (create if missing)
- apps/web/src/pages/Dashboard.tsx
- apps/web/src/pages/Applications.tsx
- apps/web/src/pages/Feedback.tsx

## Acceptance criteria
- [ ] EmptyState component accepts icon, message, optional action props
- [ ] All 3 pages show empty state when list is empty
- [ ] Empty state does not show during loading

## Verification commands
pnpm --dir apps/web typecheck
pnpm --dir apps/web lint

## Final response format
## What changed
## Why
## Files changed
## Verification
## Notes / follow-ups
```

---

## A16 — Loading skeletons instead of spinners

```
You are working in the Job Copilot monorepo.
First read: CLAUDE.md, apps/web/src/pages/Dashboard.tsx,
apps/web/src/components/ (existing skeleton components if any)

## Goal
Replace full-page spinners with skeleton placeholder cards during initial data load.
At minimum implement skeletons for: job feed cards, application board columns,
and job detail page.

## Inspect first
- apps/web/src/pages/Dashboard.tsx — loading state rendering
- apps/web/src/pages/JobDetail.tsx — loading state
- apps/web/src/pages/Applications.tsx — loading state
- apps/web/src/components/ — existing Skeleton or Shimmer component

## Likely files to modify
- apps/web/src/components/Skeleton.tsx (create if missing — simple shimmer CSS)
- apps/web/src/components/JobCardSkeleton.tsx
- apps/web/src/pages/Dashboard.tsx
- apps/web/src/pages/JobDetail.tsx

## Rules
- Use CSS animation (pulse or shimmer) — no external library.
- Show correct number of skeleton cards (e.g. 5 job card placeholders).
- Match card dimensions roughly.

## Acceptance criteria
- [ ] Job feed shows 5 skeleton cards during load
- [ ] Job detail shows skeleton layout during load
- [ ] No jarring flash from skeleton to content

## Verification commands
pnpm --dir apps/web typecheck

## Final response format
## What changed
## Why
## Files changed
## Verification
## Notes / follow-ups
```

---

## A17 — Global toast notification system

```
You are working in the Job Copilot monorepo.
First read: CLAUDE.md, apps/web/src/App.tsx, apps/web/src/api/feedback.ts

## Goal
Add a lightweight toast notification system for user-facing action feedback:
- "Job saved" after save
- "Job hidden" after hide
- "Marked as bad fit" after bad-fit
- "Error: could not save" on API failure
Use a context + portal approach. No external library (keep bundle small).

## Inspect first
- apps/web/src/App.tsx — where to mount toast provider
- apps/web/src/api/feedback.ts — mutation success/error handlers
- apps/web/src/components/ — any existing toast/notification component

## Likely files to modify
- apps/web/src/components/Toast.tsx (create)
- apps/web/src/context/ToastContext.tsx (create)
- apps/web/src/App.tsx (wrap with ToastProvider)
- apps/web/src/api/feedback.ts (call showToast in onSuccess/onError)

## Rules
- Toasts auto-dismiss after 3 seconds.
- Max 3 toasts visible at once (queue overflow).
- Match dark theme.
- Do not use react-hot-toast or similar libraries.

## Acceptance criteria
- [ ] Toast appears on save/hide/bad-fit
- [ ] Toast shows error message on API failure
- [ ] Auto-dismisses after 3s
- [ ] Multiple toasts stack correctly

## Verification commands
pnpm --dir apps/web typecheck
pnpm --dir apps/web test

## Final response format
## What changed
## Why
## Files changed
## Verification
## Notes / follow-ups
```

---

## A18 — Mobile responsive layout fixes

```
You are working in the Job Copilot monorepo.
First read: CLAUDE.md, apps/web/src/AppShell.tsx, apps/web/src/pages/Dashboard.tsx

## Goal
Audit and fix the most broken mobile layout issues:
1. Sidebar collapses/hides on mobile (hamburger menu or bottom nav)
2. Job cards stack single-column on mobile
3. Kanban board scrolls horizontally on mobile
4. Job detail modal/page is readable on small screens

## Inspect first
- apps/web/src/AppShell.tsx — sidebar layout
- apps/web/src/pages/Dashboard.tsx — grid layout
- apps/web/src/pages/Applications.tsx — Kanban layout
- apps/web/src/pages/JobDetail.tsx — detail layout

## Likely files to modify
- apps/web/src/AppShell.tsx
- apps/web/src/pages/Dashboard.tsx
- apps/web/src/pages/Applications.tsx

## Rules
- Use CSS media queries or existing CSS utility classes (no new framework).
- Do not change desktop layout.
- Minimum target: 375px viewport width.

## Acceptance criteria
- [ ] Sidebar hidden on mobile, accessible via toggle
- [ ] Job cards single-column on mobile
- [ ] No horizontal overflow on pages
- [ ] Typecheck passes

## Verification commands
pnpm --dir apps/web typecheck
pnpm --dir apps/web lint

## Final response format
## What changed
## Why
## Files changed
## Verification
## Notes / follow-ups
```
