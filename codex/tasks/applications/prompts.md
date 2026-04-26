# Block G — Applications Board Improvements (8 tasks)

---

## G1 — Interview prep integration in Application Detail

```
You are working in the Job Copilot monorepo.
First read: CLAUDE.md, apps/web/src/pages/ (ApplicationDetail or similar),
apps/web/src/api/enrichment/ (interview-prep client),
apps/ml/app/enrichment_routes.py (interview_prep endpoint)

## Goal
In the Application Detail page, add a "Prepare for Interview" section that calls
POST /ml/api/v1/enrichment/interview-prep via the engine proxy.
Shows: common interview questions for the role, suggested talking points,
skills to demonstrate.

Button "Generate Interview Prep" triggers the call. Results are shown below in a
collapsible section. Results are not persisted — re-generate on each open.

## Inspect first
- apps/web/src/pages/ — find ApplicationDetail page/component
- apps/web/src/api/enrichment/interview-prep.ts — existing client (if any)
- apps/engine-api/src/api/routes/ — check if engine proxies interview-prep
- apps/ml/app/enrichment_routes.py — interview_prep response shape

## Likely files to modify
- apps/web/src/pages/ApplicationDetail.tsx (add Interview Prep section)
- apps/web/src/api/enrichment/interview-prep.ts (ensure client exists)

## Rules
- Lazy load — only trigger on button click.
- Loading state with spinner.
- Error state with retry button.
- Do not persist to DB — enrichment is ephemeral.

## Acceptance criteria
- [ ] "Prepare for Interview" button in Application Detail
- [ ] Click calls interview-prep endpoint
- [ ] Shows questions, talking points, skills
- [ ] Loading/error states handled
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

## G2 — Application follow-up reminders

```
You are working in the Job Copilot monorepo.
First read: CLAUDE.md, apps/engine-api/src/api/routes/applications.rs,
apps/engine-api/migrations/ (applications table),
apps/web/src/pages/ (ApplicationDetail)

## Goal
Add a follow_up_date field to applications. In the Application Detail page, show
a date picker "Set follow-up reminder" (date only). When the date passes, a notification
should appear in the Notifications panel.

Backend: add follow_up_date column to applications table via migration.
Add it to the PATCH /api/v1/applications/:id handler.
Notification: create a simple job in engine-api that runs on startup, queries
applications with follow_up_date = today and creates notification entries.
(Or skip the auto-notification for now and just display the date in the UI.)

## Inspect first
- apps/engine-api/src/api/routes/applications.rs — PATCH handler
- apps/engine-api/migrations/ — applications table
- apps/web/src/pages/ — ApplicationDetail component

## Likely files to modify
- apps/engine-api/migrations/ (add follow_up_date column)
- apps/engine-api/src/api/routes/applications.rs (add to PATCH)
- apps/web/src/pages/ApplicationDetail.tsx (date picker section)
- apps/web/src/api/applications.ts (update PATCH to include follow_up_date)

## Rules
- follow_up_date is optional (nullable).
- Date picker shows in the "Tasks" or "Notes" section of Application Detail.
- For MVP: just store and display the date. Auto-notification is a stretch goal.

## Acceptance criteria
- [ ] follow_up_date column in applications
- [ ] PATCH accepts and saves follow_up_date
- [ ] Application Detail shows date picker
- [ ] Saved date persists on page refresh
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

## G3 — Application status change timeline

```
You are working in the Job Copilot monorepo.
First read: CLAUDE.md, apps/engine-api/src/api/routes/applications.rs,
apps/engine-api/migrations/ (application_activities or similar table),
apps/web/src/pages/ (ApplicationDetail)

## Goal
Show a timeline of status changes in Application Detail:
"Saved → Apr 1", "Applied → Apr 5", "Interview scheduled → Apr 10"
This requires logging status changes as activities.

Check if application_activities or activities table already logs status changes.
If not, add a trigger or explicit logging in the PATCH status handler.

## Inspect first
- apps/engine-api/src/api/routes/applications.rs — PATCH status handler
- apps/engine-api/migrations/ — activities table schema
- apps/web/src/pages/ — ApplicationDetail, activities section

## Likely files to modify
- apps/engine-api/src/api/routes/applications.rs (log status change to activities)
- apps/web/src/pages/ApplicationDetail.tsx (timeline component in activities section)

## Rules
- Log only status changes, not every field update.
- Timestamp from server, not client.
- Timeline is read-only (no edit/delete of history entries).

## Acceptance criteria
- [ ] Each status change logged as an activity with timestamp
- [ ] Application Detail shows status change timeline
- [ ] Timeline is chronological ascending
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

## G4 — Offer comparison side-by-side

```
You are working in the Job Copilot monorepo.
First read: CLAUDE.md, apps/web/src/pages/ (Applications board),
apps/engine-api/src/api/routes/applications.rs (offer fields),
apps/engine-api/migrations/ (applications/offers table)

## Goal
When the user has 2+ applications in "offer" status, show a "Compare Offers" button
in the Applications board. Clicking opens a side-by-side comparison modal showing:
- Job title, company
- Salary offered
- Start date
- Notes

Data comes from the existing offer fields in the applications table.

## Inspect first
- apps/engine-api/src/api/routes/applications.rs — offer-related fields
- apps/web/src/pages/Applications.tsx — Kanban board, offer column
- apps/web/src/api/applications.ts — applications list/detail fetching

## Likely files to modify
- apps/web/src/pages/Applications.tsx (add Compare Offers button when >= 2 offers)
- apps/web/src/components/OfferComparison.tsx (create comparison modal)

## Rules
- Button only shows when >= 2 offers exist.
- Comparison is read-only (no editing from comparison view).
- Match dark theme, 2-column grid layout.

## Acceptance criteria
- [ ] "Compare Offers" button appears when >= 2 applications are in offer status
- [ ] Modal shows 2 columns side-by-side with offer details
- [ ] Handles case of 3+ offers (show first 2, allow selecting which to compare)
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

## G5 — Application success stats

```
You are working in the Job Copilot monorepo.
First read: CLAUDE.md, apps/engine-api/src/api/routes/analytics.rs,
apps/web/src/pages/Analytics.tsx,
apps/engine-api/migrations/ (applications table)

## Goal
Add application funnel stats to the Analytics page:
- Total applications: X
- Response rate: X% (replied/applied)
- Interview rate: X% (interview/applied)
- Offer rate: X% (offer/applied)
- Source breakdown: "X from Djinni, Y from Work.ua"
- Average time-to-response: X days

Read from the applications table. Add a dedicated query in analytics service or
reuse the existing funnel endpoint.

## Inspect first
- apps/engine-api/src/api/routes/analytics.rs — funnel endpoint response shape
- apps/engine-api/src/db/ or services/ — application analytics queries
- apps/web/src/pages/Analytics.tsx — funnel section

## Likely files to modify
- apps/engine-api/src/api/routes/analytics.rs (extend funnel response or add new endpoint)
- apps/web/src/pages/Analytics.tsx (add success stats cards)

## Rules
- Rates shown as percentages (round to 1 decimal).
- "Applied" status is the denominator for all rates.
- Zero-division guard: show "0%" not NaN.

## Acceptance criteria
- [ ] Response/interview/offer rates visible in Analytics
- [ ] Source breakdown shows per-source application count
- [ ] Zero-division handled gracefully
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

## G6 — Application CSV export improvement

```
You are working in the Job Copilot monorepo.
First read: CLAUDE.md, apps/engine-api/src/api/routes/applications.rs,
apps/web/src/pages/Applications.tsx (export button if exists)

## Goal
Improve the applications CSV export to include all relevant fields:
job_title, company, source, source_url, status, applied_at, last_status_change_at,
salary_offered, contact_name, contact_email, notes (first 100 chars), follow_up_date.

Add/update GET /api/v1/applications/export endpoint returning CSV.
Web: "Export CSV" button in Applications board header.

## Inspect first
- apps/engine-api/src/api/routes/applications.rs — check if /export exists
- apps/web/src/pages/Applications.tsx — export button location
- apps/engine-api/migrations/ — applications table + related tables

## Likely files to modify
- apps/engine-api/src/api/routes/applications.rs (add/update export endpoint)
- apps/web/src/pages/Applications.tsx (add export button or improve existing)
- apps/web/src/api/applications.ts (add exportApplications function)

## Rules
- Long text fields (notes, descriptions) truncated to 100 chars in CSV.
- CSV properly escaped.
- Filename: "applications-YYYY-MM-DD.csv".

## Acceptance criteria
- [ ] Export CSV contains all listed fields
- [ ] Empty fields shown as empty string, not null
- [ ] Download triggers correctly from UI
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

## G7 — Quick apply notes in Job Detail

```
You are working in the Job Copilot monorepo.
First read: CLAUDE.md, apps/web/src/pages/JobDetail.tsx,
apps/engine-api/src/api/routes/applications.rs (create application)

## Goal
When the user clicks "Save" or "Apply" on a Job Detail page, show a small textarea
"Add a note (optional)" before confirming the action. The note is saved as the
initial note in the application record.

This is a pre-action note — appears as a dropdown/inline section when the action
button is clicked, not as a separate page.

## Inspect first
- apps/web/src/pages/JobDetail.tsx — "Save"/"Apply" action buttons
- apps/engine-api/src/api/routes/applications.rs — create application request body
- apps/web/src/api/applications.ts — create application function

## Likely files to modify
- apps/web/src/pages/JobDetail.tsx (add inline note input before action)
- apps/web/src/api/applications.ts (pass note in create request if provided)

## Rules
- Note is optional — existing quick save/apply still works without note.
- Textarea appears inline below the button, not in a modal.
- Max 500 characters.
- Auto-focuses textarea when it appears.

## Acceptance criteria
- [ ] Clicking "Apply" reveals textarea
- [ ] Submitting with note saves note to application
- [ ] Submitting without note works (empty string or null)
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

## G8 — Kanban keyboard shortcuts

```
You are working in the Job Copilot monorepo.
First read: CLAUDE.md, apps/web/src/pages/Applications.tsx,
apps/web/src/components/ (Kanban or application board components)

## Goal
Add keyboard shortcuts in the Applications Kanban board:
- Arrow keys: navigate between cards
- S: move selected card to Saved
- A: move selected card to Applied
- I: move selected card to Interview
- O: move selected card to Offer
- R: move selected card to Rejected
- Enter or Space: open Application Detail for selected card
- ?: show keyboard shortcut help overlay

## Inspect first
- apps/web/src/pages/Applications.tsx — Kanban board, how cards are rendered
- apps/web/src/components/ — card components, any existing keyboard handling

## Likely files to modify
- apps/web/src/pages/Applications.tsx (add keyboard event listener)
- apps/web/src/components/ (add selected/focused state to cards)
- apps/web/src/components/KeyboardShortcutsHelp.tsx (create help overlay)

## Rules
- Shortcuts only active when Kanban tab is focused (not in input fields).
- Show selected card with visible focus ring.
- Help overlay on "?" key, dismiss with Escape.

## Acceptance criteria
- [ ] Arrow navigation moves focus between cards
- [ ] S/A/I/O/R shortcuts change status of focused card
- [ ] Enter opens Application Detail
- [ ] "?" shows shortcut overlay
- [ ] Shortcuts inactive when typing in inputs
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
