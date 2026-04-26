# Block D — Settings Expansion (8 tasks)

Each task below is a self-contained Codex prompt.

---

## D1 — Settings: notification preferences

```
You are working in the Job Copilot monorepo.
First read: CLAUDE.md, apps/web/src/pages/Settings.tsx,
apps/engine-api/src/api/routes/notifications.rs,
apps/engine-api/migrations/ (notification_preferences table if exists)

## Goal
Add a "Notifications" section in Settings where the user can toggle:
- New jobs matching search profile (on/off)
- Application status change reminders (on/off)
- Weekly digest (on/off)
- Market intelligence updates (on/off)

These preferences should persist in the engine-api (not localStorage). If no
notification_preferences table exists, add a migration.

Backend: add PATCH /api/v1/notifications/preferences and GET /api/v1/notifications/preferences.
Web: connect Settings Notifications section to these endpoints.

## Inspect first
- apps/engine-api/src/api/routes/notifications.rs — existing notification routes
- apps/engine-api/migrations/ — check for notification_preferences table
- apps/web/src/pages/Settings.tsx — Notifications section placeholder

## Likely files to modify
- apps/engine-api/src/api/routes/notifications.rs (add preferences endpoints)
- apps/engine-api/migrations/ (new migration if table missing)
- apps/web/src/pages/Settings.tsx (wire Notifications section)
- apps/web/src/api/notifications.ts (add preferences client)

## Rules
- Preferences are profile-scoped.
- Default: all notifications ON.
- PATCH is partial update (only send changed fields).

## Acceptance criteria
- [ ] GET /api/v1/notifications/preferences returns current preferences
- [ ] PATCH updates preferences
- [ ] Settings page shows toggles wired to API
- [ ] Default is all ON for new profiles
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

## D2 — Settings: ingestion frequency preference

```
You are working in the Job Copilot monorepo.
First read: CLAUDE.md, apps/web/src/pages/Settings.tsx,
apps/engine-api/src/api/routes/ (profile or settings endpoint),
apps/ingestion/src/ (daemon interval config)

## Goal
Add an "Ingestion Frequency" setting: 30 / 60 / 120 minutes (radio buttons).
Store in the profile or a settings table. The ingestion daemon reads this preference
to set its scraping interval.

Note: if per-user ingestion frequency is too complex (ingestion is a global daemon),
store the preference as a display setting only — show "Scrapes every X minutes" based
on the saved value, but the actual daemon interval comes from env/config.

## Inspect first
- apps/ingestion/src/main.rs or daemon.rs — where interval is configured
- apps/engine-api/src/api/routes/ — if a general settings endpoint exists
- apps/web/src/pages/Settings.tsx — Search Preferences section

## Likely files to modify
- apps/web/src/pages/Settings.tsx (add frequency radio group in Search Preferences)
- localStorage for now (if per-user daemon is out of scope, this is display-only)

## Rules
- If per-user daemon interval is not feasible: store in localStorage, show informational
  text "Scraping runs every 60 min (system default)".
- Do not change the actual ingestion daemon unless it naturally supports per-user intervals.

## Acceptance criteria
- [ ] Frequency preference UI exists in Settings
- [ ] Preference persists (localStorage or API)
- [ ] Clear label showing what the setting actually controls

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

## D3 — Settings: scoring weight sliders

```
You are working in the Job Copilot monorepo.
First read: CLAUDE.md, apps/web/src/pages/Settings.tsx,
apps/engine-api/src/services/ (scoring — where weight constants are),
apps/engine-api/src/api/routes/ (profile or settings endpoint)

## Goal
Add a "Scoring Weights" section in Settings with sliders:
- Skill match importance (1-10, default 8)
- Salary fit importance (1-10, default 6)
- Job freshness importance (1-10, default 5)
- Remote work importance (1-10, default 5)

These weights modify the scoring engine per profile. Store in the profile or a
search_preferences table. Pass weights to scoring service when computing job scores.

## Inspect first
- apps/engine-api/src/services/ — scoring weight constants
- apps/engine-api/src/domain/ — search profile struct (may already have weight fields)
- apps/engine-api/migrations/ — check for weight columns

## Likely files to modify
- apps/engine-api/src/domain/ (add weight fields to search profile if missing)
- apps/engine-api/migrations/ (migration if columns added)
- apps/engine-api/src/services/ (read weights from profile instead of constants)
- apps/web/src/pages/Settings.tsx (add sliders)
- apps/web/src/api/ (PATCH search profile or settings)

## Rules
- Weights are 1-10 integers, stored as such.
- Default values must be sensible (not 0).
- Changing weights triggers feed re-score on next page load.

## Acceptance criteria
- [ ] Sliders exist in Settings Scoring section
- [ ] Weights persist to engine-api
- [ ] Scoring service uses profile weights instead of global constants
- [ ] Changing weights and reloading dashboard shows different order
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

## D4 — Settings: clear all hidden jobs

```
You are working in the Job Copilot monorepo.
First read: CLAUDE.md, apps/engine-api/src/api/routes/feedback.rs,
apps/web/src/pages/Settings.tsx, apps/web/src/pages/Feedback.tsx

## Goal
Add a "Clear all hidden jobs" button in Settings → Data & Privacy section.
This calls a new endpoint DELETE /api/v1/feedback/hidden/all which removes all
hidden feedback entries for the current profile. Show confirmation dialog before proceeding.

## Inspect first
- apps/engine-api/src/api/routes/feedback.rs — existing feedback routes
- apps/engine-api/src/db/ — feedback table queries
- apps/web/src/pages/Settings.tsx — Data & Privacy section

## Likely files to modify
- apps/engine-api/src/api/routes/feedback.rs (add bulk delete endpoint)
- apps/web/src/pages/Settings.tsx (add button with confirm dialog)
- apps/web/src/api/feedback.ts (add clearAllHidden function)

## Rules
- Confirmation dialog must say exactly how many items will be deleted.
- Operation is profile-scoped (cannot delete other profile's data).
- Return 204 No Content on success.

## Acceptance criteria
- [ ] DELETE /api/v1/feedback/hidden/all deletes all hidden for profile
- [ ] 403 if profile mismatch
- [ ] Confirmation dialog shows count
- [ ] After confirmation, hidden jobs list in Feedback Center is empty
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

## D5 — Settings: blacklist management

```
You are working in the Job Copilot monorepo.
First read: CLAUDE.md, apps/engine-api/src/api/routes/feedback.rs,
apps/web/src/pages/Feedback.tsx (companies tab),
apps/web/src/pages/Settings.tsx

## Goal
In Settings → Data & Privacy, add a "Blocked Companies" section that shows the
current blacklist with the ability to remove individual companies.
This reuses the existing company blacklist data — just a different UI entry point
(the primary UI is in Feedback Center → Companies tab, but Settings provides a
management view).

Also add DELETE /api/v1/feedback/companies/:company_slug/blacklist endpoint
(if it doesn't exist already) to remove a company from blacklist.

## Inspect first
- apps/engine-api/src/api/routes/feedback.rs — company whitelist/blacklist endpoints
- apps/web/src/pages/Feedback.tsx — companies tab (companies component)
- apps/web/src/api/feedback.ts — company feedback functions

## Likely files to modify
- apps/engine-api/src/api/routes/feedback.rs (add remove-from-blacklist if missing)
- apps/web/src/pages/Settings.tsx (add Blocked Companies section)
- apps/web/src/api/feedback.ts (add removeFromBlacklist if missing)

## Acceptance criteria
- [ ] Settings shows list of blacklisted companies
- [ ] Each has a "Remove" button
- [ ] Removal confirmed with "Are you sure?" prompt
- [ ] After removal, company no longer in list
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

## D6 — Settings: export profile + feedback as JSON

```
You are working in the Job Copilot monorepo.
First read: CLAUDE.md, apps/engine-api/src/api/routes/profile.rs,
apps/engine-api/src/api/routes/feedback.rs,
apps/web/src/pages/Settings.tsx

## Goal
Add GET /api/v1/export which returns a JSON export of all user data:
{ profile: {...}, feedback: { saved: [...], hidden: [...], bad_fit: [...] },
  companies: { whitelist: [...], blacklist: [...] },
  applications: [...] }

Web: add "Export my data" button in Settings → Data & Privacy.
On click, fetches the endpoint and triggers a browser download of the JSON file
named "job-copilot-export-YYYY-MM-DD.json".

## Inspect first
- apps/engine-api/src/api/routes/ — existing export or bulk read endpoints
- apps/engine-api/src/db/ — queries needed for each section
- apps/web/src/pages/Settings.tsx — Data & Privacy section

## Likely files to modify
- New file: apps/engine-api/src/api/routes/export.rs
- apps/engine-api/src/api/routes/mod.rs (register)
- apps/web/src/pages/Settings.tsx (add button)
- apps/web/src/api/ (add export client function)

## Rules
- Auth-required. Profile-scoped only.
- Do not include internal IDs that mean nothing to the user.
- Response Content-Type: application/json.
- Web triggers download, not opens in tab.

## Acceptance criteria
- [ ] GET /api/v1/export returns full user data JSON
- [ ] "Export my data" button in Settings
- [ ] Click triggers file download with correct filename
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

## D7 — Settings: CV management

```
You are working in the Job Copilot monorepo.
First read: CLAUDE.md, apps/engine-api/src/api/routes/resumes.rs,
apps/web/src/pages/Profile.tsx (resume upload section),
apps/web/src/pages/Settings.tsx

## Goal
Add a "CV Management" section in Settings → Profile & Account that shows:
- List of uploaded CVs with filename, upload date, and active badge
- "Activate" button to make a CV the active one
- "Delete" button to remove a CV (with confirmation)
- "Upload new CV" link → navigates to Profile page upload section

This reuses the existing /api/v1/resumes endpoints — just a dedicated management view
in Settings.

## Inspect first
- apps/engine-api/src/api/routes/resumes.rs — list, activate, delete endpoints
- apps/web/src/pages/Profile.tsx — existing resume UI
- apps/web/src/api/ — resume client functions

## Likely files to modify
- apps/web/src/pages/Settings.tsx (add CV Management section)
- apps/web/src/api/ (reuse or extend resume client)

## Rules
- Do not duplicate resume upload logic — link to Profile page for uploads.
- Delete requires confirmation dialog.
- Active resume has a visual indicator (badge/checkmark).

## Acceptance criteria
- [ ] CV list visible in Settings
- [ ] Activate changes active resume
- [ ] Delete removes resume after confirmation
- [ ] "Upload new CV" navigates to /profile
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

## D8 — Settings: delete all data (placeholder)

```
You are working in the Job Copilot monorepo.
First read: CLAUDE.md, apps/web/src/pages/Settings.tsx,
apps/engine-api/src/api/routes/ (check if any bulk-delete endpoints exist)

## Goal
Add a "Danger Zone" section at the bottom of Settings with a "Clear all data" button.
For now: delete all feedback (saved/hidden/bad-fit), all applications, and reset the
search profile to blank. Do NOT delete the candidate profile itself.

Backend: POST /api/v1/data/reset — deletes feedback + applications for the profile.
Web: button with double-confirmation (type "RESET" to confirm).

## Inspect first
- apps/engine-api/src/api/routes/ — check for any reset/clear endpoints
- apps/engine-api/src/db/ — tables: feedback, applications
- apps/web/src/pages/Settings.tsx — Danger Zone section

## Likely files to modify
- New file: apps/engine-api/src/api/routes/data_management.rs
- apps/engine-api/src/api/routes/mod.rs (register)
- apps/web/src/pages/Settings.tsx (Danger Zone section)

## Rules
- Double confirmation: first confirm dialog, then type "RESET" in text input.
- Profile (skills, CV, name) is NOT deleted — only feedback + applications.
- Return 204 on success.
- This is a destructive operation — add extra care to ownership check.

## Acceptance criteria
- [ ] POST /api/v1/data/reset deletes feedback + applications for the profile
- [ ] Profile itself is preserved
- [ ] Web shows double-confirmation flow
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
