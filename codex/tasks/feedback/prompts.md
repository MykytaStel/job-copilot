# Block F — Feedback Center Improvements (8 tasks)

---

## F1 — Bulk feedback: hide all from company

```
You are working in the Job Copilot monorepo.
First read: CLAUDE.md, apps/engine-api/src/api/routes/feedback.rs,
apps/web/src/pages/Feedback.tsx (or wherever feedback list is)

## Goal
In the job feed and feedback center, add a "Hide all from [Company]" action.
When triggered from a job card's context menu or the feedback center's company row:
1. Backend: POST /api/v1/feedback/jobs/bulk-hide with body { company_name: String }
   → hides all unhidden jobs from that company for the current profile.
2. Web: "Hide all from X" option in job card menu + button in Feedback Center → Companies tab.

## Inspect first
- apps/engine-api/src/api/routes/feedback.rs — existing bulk and per-job routes
- apps/web/src/pages/Feedback.tsx — companies tab
- apps/web/src/pages/Dashboard.tsx — job card action menu

## Likely files to modify
- apps/engine-api/src/api/routes/feedback.rs (add bulk-hide endpoint)
- apps/web/src/pages/Dashboard.tsx (context menu option)
- apps/web/src/pages/Feedback.tsx (company row button)
- apps/web/src/api/feedback.ts (add bulkHideByCompany function)

## Rules
- Bulk operation is profile-scoped.
- Returns count of affected jobs in response body.
- Confirm if count > 5 ("Hide 8 jobs from Acme Corp?").

## Acceptance criteria
- [ ] POST /api/v1/feedback/jobs/bulk-hide hides all jobs from company
- [ ] Confirmation dialog for > 5 jobs
- [ ] Response includes affected_count
- [ ] Feed refreshes after bulk hide
- [ ] Cargo + web typecheck pass

## Verification commands
cargo test --manifest-path apps/engine-api/Cargo.toml
pnpm --dir apps/web typecheck

## Final response format
## What changed
## Why
## Files changed
## Verification
## Notes / follow-ups
```

---

## F2 — Feedback undo (30-second window)

```
You are working in the Job Copilot monorepo.
First read: CLAUDE.md, apps/web/src/api/feedback.ts,
apps/web/src/context/ToastContext.tsx (from A17),
apps/engine-api/src/api/routes/feedback.rs

## Goal
After a hide or bad-fit action, show an "Undo" button in the toast notification for
30 seconds. If clicked, call DELETE /api/v1/feedback/jobs/:job_id/hide (or equivalent)
to remove the feedback entry.

Backend: add DELETE /api/v1/feedback/jobs/:job_id/hide and
DELETE /api/v1/feedback/jobs/:job_id/bad-fit endpoints.
Web: toast shows "Job hidden [Undo]" with a 30s countdown or just 30s window.

## Inspect first
- apps/engine-api/src/api/routes/feedback.rs — existing routes, any delete endpoints
- apps/web/src/context/ToastContext.tsx — toast component
- apps/web/src/api/feedback.ts — hide/bad-fit mutations

## Likely files to modify
- apps/engine-api/src/api/routes/feedback.rs (add DELETE hide/bad-fit endpoints)
- apps/web/src/context/ToastContext.tsx or Toast.tsx (add action button)
- apps/web/src/api/feedback.ts (add undo functions)

## Rules
- Undo window is client-side only (30s timer in toast).
- After undo, invalidate job feed (job reappears).
- DELETE endpoints are idempotent (no error if entry already gone).

## Acceptance criteria
- [ ] DELETE /api/v1/feedback/jobs/:id/hide removes hide entry
- [ ] DELETE /api/v1/feedback/jobs/:id/bad-fit removes bad-fit entry
- [ ] Toast after hide/bad-fit has "Undo" button
- [ ] Undo reverses the action and job reappears in feed
- [ ] Undo button disappears after 30s

## Verification commands
cargo test --manifest-path apps/engine-api/Cargo.toml
pnpm --dir apps/web typecheck

## Final response format
## What changed
## Why
## Files changed
## Verification
## Notes / follow-ups
```

---

## F3 — Company notes

```
You are working in the Job Copilot monorepo.
First read: CLAUDE.md, apps/engine-api/src/api/routes/feedback.rs,
apps/engine-api/migrations/ (company_feedback table),
apps/web/src/pages/Feedback.tsx (companies tab)

## Goal
Add a notes field to company feedback (whitelist/blacklist rows). The user can write
a short note about why they blacklisted/whitelisted a company (e.g. "Interviewed here, bad culture").
Notes are stored in the company_feedback table and displayed in the Companies tab.

Backend: PATCH /api/v1/feedback/companies/:company_slug with { notes: String }.
Web: inline textarea in the company row (click to edit, auto-save on blur).

## Inspect first
- apps/engine-api/src/api/routes/feedback.rs — company feedback endpoints
- apps/engine-api/migrations/ — company_feedback table schema
- apps/web/src/pages/Feedback.tsx — companies tab component

## Likely files to modify
- apps/engine-api/migrations/ (add notes column if missing)
- apps/engine-api/src/api/routes/feedback.rs (add/update PATCH handler)
- apps/web/src/pages/Feedback.tsx (inline note editor per company row)
- apps/web/src/api/feedback.ts (update company function)

## Rules
- Notes max 500 characters (validate in engine-api).
- Auto-save on blur (no explicit save button).
- Empty note is allowed (clears existing note).

## Acceptance criteria
- [ ] notes column exists in company_feedback
- [ ] PATCH saves note
- [ ] Company row shows existing note in editable text area
- [ ] Auto-saves on blur
- [ ] Cargo + web typecheck pass

## Verification commands
cargo test --manifest-path apps/engine-api/Cargo.toml
pnpm --dir apps/web typecheck

## Final response format
## What changed
## Why
## Files changed
## Verification
## Notes / follow-ups
```

---

## F4 — Feedback stats panel

```
You are working in the Job Copilot monorepo.
First read: CLAUDE.md, apps/web/src/pages/Feedback.tsx,
apps/engine-api/src/api/routes/analytics.rs (behavior signals endpoint)

## Goal
Add a stats summary panel at the top of the Feedback Center:
- "Saved this week: X"
- "Hidden this week: X"
- "Bad fit this week: X"
- "Whitelisted companies: X"
- "Blacklisted companies: X"

Read from the behavior signals endpoint if it returns this data, or add a new
lightweight endpoint GET /api/v1/feedback/stats.

## Inspect first
- apps/engine-api/src/api/routes/analytics.rs — behavior_summary endpoint
- apps/engine-api/src/api/routes/feedback.rs — any existing count endpoints
- apps/web/src/pages/Feedback.tsx — top of page

## Likely files to modify
- apps/engine-api/src/api/routes/feedback.rs (add /stats endpoint if needed)
- apps/web/src/pages/Feedback.tsx (add stats bar)
- apps/web/src/api/feedback.ts (add stats client function)

## Rules
- "This week" = last 7 days from current timestamp.
- Stats panel is non-critical — if endpoint fails, show "--" not an error.
- No new DB tables.

## Acceptance criteria
- [ ] Stats panel visible at top of Feedback Center
- [ ] Counts are accurate for the last 7 days
- [ ] Shows "--" gracefully if data unavailable
- [ ] Cargo + web typecheck pass

## Verification commands
cargo test --manifest-path apps/engine-api/Cargo.toml
pnpm --dir apps/web typecheck

## Final response format
## What changed
## Why
## Files changed
## Verification
## Notes / follow-ups
```

---

## F5 — Feedback export to CSV

```
You are working in the Job Copilot monorepo.
First read: CLAUDE.md, apps/engine-api/src/api/routes/feedback.rs,
apps/web/src/pages/Feedback.tsx

## Goal
Add an "Export" button in the Feedback Center that downloads a CSV of the current
tab's data:
- Saved tab → CSV with: job_title, company, source, saved_at, url
- Hidden tab → CSV with: job_title, company, source, hidden_at
- Bad Fit tab → CSV with: job_title, company, source, marked_at, reason
- Companies tab → CSV with: company, status (whitelist/blacklist), notes, date

Backend: GET /api/v1/feedback/export?type=saved|hidden|bad_fit|companies
Returns CSV with Content-Disposition: attachment; filename="feedback-<type>-YYYY-MM-DD.csv"

## Inspect first
- apps/engine-api/src/api/routes/feedback.rs — list endpoints for each feedback type
- apps/web/src/pages/Feedback.tsx — tab structure, active tab state
- apps/web/src/api/feedback.ts — existing list functions

## Likely files to modify
- apps/engine-api/src/api/routes/feedback.rs (add /export endpoint with CSV response)
- apps/web/src/pages/Feedback.tsx (add Export button)
- apps/web/src/api/feedback.ts (add exportFeedback function)

## Rules
- CSV must be properly escaped (commas in fields quoted).
- Auth-required, profile-scoped.
- Use Content-Type: text/csv and Content-Disposition: attachment.

## Acceptance criteria
- [ ] Export endpoint returns valid CSV
- [ ] Web button triggers download for active tab type
- [ ] Filename includes date
- [ ] Special characters in company names correctly escaped
- [ ] Cargo + web typecheck pass

## Verification commands
cargo test --manifest-path apps/engine-api/Cargo.toml
pnpm --dir apps/web typecheck

## Final response format
## What changed
## Why
## Files changed
## Verification
## Notes / follow-ups
```

---

## F6 — Bad fit reason tagging

```
You are working in the Job Copilot monorepo.
First read: CLAUDE.md, apps/engine-api/src/api/routes/feedback.rs,
apps/engine-api/migrations/ (job_feedback table),
apps/web/src/pages/Dashboard.tsx (bad-fit action)

## Goal
When marking a job as "bad fit", show a quick reason selector:
reasons = ["Salary too low", "Wrong location", "Wrong tech stack", "Too senior/junior",
           "Bad company reputation", "Other"]
The selected reason (or "other" text) is stored alongside the bad-fit feedback.

Backend: extend POST /api/v1/feedback/jobs/:id/bad-fit to accept { reason: String }.
Web: after clicking "Bad fit", show a small popover with reason chips. Confirm closes popover.

## Inspect first
- apps/engine-api/src/api/routes/feedback.rs — bad-fit endpoint
- apps/engine-api/migrations/ — job_feedback table (check for reason column)
- apps/web/src/pages/Dashboard.tsx — bad-fit action handler

## Likely files to modify
- apps/engine-api/migrations/ (add reason column to job_feedback)
- apps/engine-api/src/api/routes/feedback.rs (accept reason in request body)
- apps/web/src/pages/Dashboard.tsx or JobCard component (reason popover)

## Rules
- reason is optional — existing bad-fit calls without reason still work.
- Reason values are free-form strings (no enum constraint in DB, but suggest from list in UI).
- Migration: nullable TEXT column.

## Acceptance criteria
- [ ] reason column exists in job_feedback
- [ ] POST bad-fit accepts optional reason
- [ ] Web shows reason popover after "Bad fit" click
- [ ] Selected reason sent with request
- [ ] Cargo + web typecheck pass

## Verification commands
cargo test --manifest-path apps/engine-api/Cargo.toml
pnpm --dir apps/web typecheck

## Final response format
## What changed
## Why
## Files changed
## Verification
## Notes / follow-ups
```

---

## F7 — Interest rating (1-5 stars)

```
You are working in the Job Copilot monorepo.
First read: CLAUDE.md, apps/engine-api/src/api/routes/feedback.rs,
apps/engine-api/migrations/ (job_feedback table — check for interest_rating column),
apps/web/src/components/ (job cards)

## Goal
The backend may already have interest_rating in job_feedback. If so:
Wire it to the UI as a 1-5 star rating in the job card (compact star row) or in
Job Detail. Rating saves via PATCH /api/v1/feedback/jobs/:id with { interest_rating: 1-5 }.
If the backend field doesn't exist, add it with a migration first.

## Inspect first
- apps/engine-api/migrations/ — job_feedback table
- apps/engine-api/src/api/routes/feedback.rs — existing interest_rating handling
- apps/web/src/components/ — existing star/rating component (may already exist)
- apps/web/src/pages/JobDetail.tsx — match/overview tab

## Likely files to modify
- apps/engine-api/migrations/ (if column missing)
- apps/engine-api/src/api/routes/feedback.rs (ensure PATCH handles interest_rating)
- apps/web/src/components/StarRating.tsx (create if missing)
- apps/web/src/pages/JobDetail.tsx (add star rating in Match tab)

## Rules
- Rating is 1-5 integer, nullable (not rated = null).
- Hover state shows preview (3 filled stars on hover over 3rd star).
- Saving rating does not require a separate save button — saves on click.

## Acceptance criteria
- [ ] interest_rating column exists
- [ ] PATCH saves rating
- [ ] StarRating component renders 5 stars with hover/active states
- [ ] Job Detail shows current rating
- [ ] Click changes rating and saves

## Verification commands
cargo test --manifest-path apps/engine-api/Cargo.toml
pnpm --dir apps/web typecheck

## Final response format
## What changed
## Why
## Files changed
## Verification
## Notes / follow-ups
```

---

## F8 — Feedback timeline/history

```
You are working in the Job Copilot monorepo.
First read: CLAUDE.md, apps/engine-api/src/api/routes/events.rs,
apps/engine-api/src/api/routes/feedback.rs,
apps/web/src/pages/Feedback.tsx

## Goal
Add a "Timeline" tab in the Feedback Center that shows a chronological history of
all feedback actions: "Saved [Job Title] at [Company] — Apr 22", "Hidden [Job Title]...",
"Marked bad fit [Job Title]... (reason: Salary too low)".

Read from the user_events table or a derived view. Show most recent first, paginated (20/page).

## Inspect first
- apps/engine-api/src/api/routes/events.rs — events list endpoint
- apps/engine-api/src/api/routes/feedback.rs — if there's already a history endpoint
- apps/web/src/pages/Feedback.tsx — tab structure

## Likely files to modify
- apps/engine-api/src/api/routes/feedback.rs or events.rs (add history endpoint if missing)
- apps/web/src/pages/Feedback.tsx (add Timeline tab)
- apps/web/src/api/feedback.ts (add history client)

## Rules
- Read-only display (no undo from timeline — that's F2).
- Show job title, company, action type, and timestamp.
- Paginated: load more button or infinite scroll.

## Acceptance criteria
- [ ] Timeline tab exists in Feedback Center
- [ ] Shows all feedback actions in reverse chronological order
- [ ] Pagination/load more works
- [ ] Empty state when no history

## Verification commands
cargo test --manifest-path apps/engine-api/Cargo.toml
pnpm --dir apps/web typecheck

## Final response format
## What changed
## Why
## Files changed
## Verification
## Notes / follow-ups
```
