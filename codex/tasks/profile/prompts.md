# Block E — Profile Improvements (10 tasks)

---

## E1 — Profile completion indicator

```
You are working in the Job Copilot monorepo.
First read: CLAUDE.md, apps/web/src/pages/Profile.tsx,
apps/engine-api/src/domain/ (profile struct fields),
apps/engine-api/src/api/routes/profile.rs

## Goal
Show a profile completion percentage and a checklist of what's missing:
- Has name: +10%
- Has email: +10%
- Has skills (>= 3): +20%
- Has CV uploaded: +20%
- Has salary expectation: +10%
- Has work mode preference: +10%
- Has location preference: +10%
- Has language preference: +10%
Total: 100%

Show a progress bar + "X% complete" label + checklist of incomplete items in the Profile page.
Compute this on the frontend from profile data (no new backend endpoint needed).

## Inspect first
- apps/web/src/pages/Profile.tsx — current profile page layout
- apps/web/src/api/ — profile data shape returned from API
- packages/contracts/src/profiles.ts — profile type definition

## Likely files to modify
- apps/web/src/pages/Profile.tsx or a new ProfileCompletion.tsx component
- apps/web/src/components/ProfileCompletion.tsx (create)

## Rules
- Frontend computation only — no API call.
- Progress bar matches dark theme (use accent color for fill).
- Checklist items are clickable → scroll to the relevant section.

## Acceptance criteria
- [ ] Progress bar shows correct percentage
- [ ] Checklist shows incomplete items only
- [ ] Clicking checklist item scrolls to the section
- [ ] 100% shows a "Profile complete" badge

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

## E2 — Skills auto-suggest from CV analysis

```
You are working in the Job Copilot monorepo.
First read: CLAUDE.md, apps/engine-api/src/api/routes/profile.rs (analyze endpoint),
apps/web/src/pages/Profile.tsx (skills section)

## Goal
After the user uploads and analyzes a CV (POST /api/v1/profiles/:id/analyze),
the response contains extracted skills. Currently these may or may not be
auto-populated into the profile's skills list.

If they're not auto-saved: after analyze, show the extracted skills as "suggested skills"
chips in the Profile page with a "Add all" button and individual "+" buttons per skill.
Clicking "Add" calls PATCH /api/v1/profiles/:id to merge the skill into the skills array.

## Inspect first
- apps/engine-api/src/api/routes/profile.rs — analyze response shape
- apps/web/src/pages/Profile.tsx — skills editing section
- apps/web/src/api/profiles/ — analyze and update profile functions

## Likely files to modify
- apps/web/src/pages/Profile.tsx (add suggestion row after analyze)
- apps/web/src/api/profiles/ (ensure patch/update function exists)

## Rules
- Suggestions are separate from confirmed skills until user clicks "Add".
- Duplicate skills must not be added twice.
- Clear suggestion list after "Add all".

## Acceptance criteria
- [ ] After analyze, extracted skills shown as suggestions
- [ ] "Add all" adds all non-duplicate suggestions
- [ ] Individual "+" adds one skill
- [ ] Suggestions clear after adding
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

## E3 — Profile: preferred work modes field

```
You are working in the Job Copilot monorepo.
First read: CLAUDE.md, apps/engine-api/src/domain/ (profile struct),
apps/engine-api/migrations/, apps/web/src/pages/Profile.tsx

## Goal
Add work_mode_preference to the candidate profile: values are
remote_only / hybrid / onsite / any (enum). Store in the profiles table.
Show as a radio button group in the Profile page's preferences section.

## Inspect first
- apps/engine-api/src/domain/ — CandidateProfile struct
- apps/engine-api/migrations/ — profiles table columns
- apps/web/src/pages/Profile.tsx — preference fields section
- packages/contracts/src/profiles.ts — shared TS type

## Likely files to modify
- apps/engine-api/src/domain/ (add work_mode_preference field)
- apps/engine-api/migrations/ (new migration)
- apps/engine-api/src/api/routes/profile.rs (include in PATCH handler)
- packages/contracts/src/profiles.ts (add field to type)
- apps/web/src/pages/Profile.tsx (radio group UI)

## Rules
- Migration must be non-breaking (nullable or default 'any').
- PATCH must allow partial update (omitting field = no change).

## Acceptance criteria
- [ ] work_mode_preference persists to DB
- [ ] Profile page radio group reflects saved value
- [ ] Changing and saving works
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

## E4 — Profile: language proficiency levels

```
You are working in the Job Copilot monorepo.
First read: CLAUDE.md, apps/engine-api/src/domain/ (profile struct),
apps/engine-api/migrations/, apps/web/src/pages/Profile.tsx

## Goal
Add languages array to the candidate profile. Each entry:
{ language: String, level: A1|A2|B1|B2|C1|C2|Native }
Store as JSONB in the profiles table.
Show as a tag list in the Profile page where the user can add/remove languages
and set proficiency level per language.

## Inspect first
- apps/engine-api/src/domain/ — profile struct (check if languages already exists)
- apps/engine-api/migrations/ — profiles table
- apps/web/src/pages/Profile.tsx — existing tag/multi-select patterns
- packages/contracts/src/profiles.ts

## Likely files to modify
- apps/engine-api/src/domain/ (add languages field)
- apps/engine-api/migrations/ (add languages JSONB column)
- apps/engine-api/src/api/routes/profile.rs (include in PATCH)
- packages/contracts/src/profiles.ts
- apps/web/src/pages/Profile.tsx (language list with level picker)

## Rules
- Migration nullable (existing profiles get null → treated as empty array).
- Use CEFR levels (A1-C2 + Native).
- UI: add language by name + select level from dropdown.

## Acceptance criteria
- [ ] Languages JSONB column in DB
- [ ] PATCH saves/updates languages array
- [ ] Profile page shows language tags with level badge
- [ ] Add/remove language works
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

## E5 — Profile: salary expectations

```
You are working in the Job Copilot monorepo.
First read: CLAUDE.md, apps/engine-api/src/domain/ (profile struct),
apps/engine-api/migrations/, apps/web/src/pages/Profile.tsx

## Goal
Add salary_min, salary_max, salary_currency (USD/EUR/UAH) fields to the candidate profile.
Show as a range input in the Profile page (two number inputs + currency selector).
These values are used by the B2 scoring task.

## Inspect first
- apps/engine-api/src/domain/ — profile struct (check if salary fields already exist)
- apps/engine-api/migrations/ — profiles table
- apps/web/src/pages/Profile.tsx — preferences section

## Likely files to modify
- apps/engine-api/src/domain/ (add salary fields if missing)
- apps/engine-api/migrations/ (add columns if missing)
- apps/engine-api/src/api/routes/profile.rs (include in PATCH)
- packages/contracts/src/profiles.ts
- apps/web/src/pages/Profile.tsx (salary range inputs)

## Rules
- All three fields are optional (no penalty for not filling).
- salary_min must be <= salary_max (validate in engine-api).
- Currency must be one of: USD, EUR, UAH.

## Acceptance criteria
- [ ] salary_min/max/currency persist to DB
- [ ] Profile page has salary range inputs with currency dropdown
- [ ] Validation: min <= max, valid currency
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

## E6 — Profile: location preferences

```
You are working in the Job Copilot monorepo.
First read: CLAUDE.md, apps/engine-api/src/domain/ (profile struct),
apps/engine-api/migrations/, apps/web/src/pages/Profile.tsx

## Goal
Add preferred_locations: Vec<String> to the candidate profile (e.g. ["Kyiv", "Remote", "Lviv"]).
Store as JSONB text array. Show as a tag input in the Profile page where the user
can type a city/country and add it to the list.

## Inspect first
- apps/engine-api/src/domain/ — profile struct
- apps/engine-api/migrations/ — profiles table
- apps/web/src/pages/Profile.tsx — tag input patterns (check if skills input can be reused)

## Likely files to modify
- apps/engine-api/src/domain/ (add preferred_locations)
- apps/engine-api/migrations/ (new column)
- apps/engine-api/src/api/routes/profile.rs
- packages/contracts/src/profiles.ts
- apps/web/src/pages/Profile.tsx (tag input for locations)

## Rules
- Nullable in DB — existing profiles have empty array.
- No geolocation validation — free text input.
- Show "Remote" as a predefined option.

## Acceptance criteria
- [ ] preferred_locations persists to DB
- [ ] Tag input lets user add/remove locations
- [ ] "Remote" available as a quick-add option
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

## E7 — Profile: portfolio and GitHub links

```
You are working in the Job Copilot monorepo.
First read: CLAUDE.md, apps/engine-api/src/domain/ (profile struct),
apps/engine-api/migrations/, apps/web/src/pages/Profile.tsx

## Goal
Add portfolio_url, github_url, linkedin_url fields (all optional String) to the candidate
profile. Show as URL inputs in the Profile page's contact/links section.
Validate that values are valid URLs on the backend.

## Inspect first
- apps/engine-api/src/domain/ — profile struct
- apps/engine-api/migrations/ — profiles table
- apps/web/src/pages/Profile.tsx — contact fields section

## Likely files to modify
- apps/engine-api/src/domain/ (add URL fields)
- apps/engine-api/migrations/ (add columns)
- apps/engine-api/src/api/routes/profile.rs (URL validation in PATCH handler)
- packages/contracts/src/profiles.ts
- apps/web/src/pages/Profile.tsx (URL inputs with icons)

## Rules
- URL validation: must start with http:// or https://.
- All fields nullable.
- Show icons (GitHub, LinkedIn, Portfolio) next to inputs.

## Acceptance criteria
- [ ] URL fields persist to DB
- [ ] Invalid URL returns 422 with field-level error
- [ ] Profile page shows URL inputs with correct icons
- [ ] Links are clickable (open in new tab)
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

## E8 — Profile: experience timeline entries

```
You are working in the Job Copilot monorepo.
First read: CLAUDE.md, apps/engine-api/src/domain/ (profile struct),
apps/engine-api/migrations/, apps/web/src/pages/Profile.tsx

## Goal
Add experience: Vec<ExperienceEntry> to the profile where:
ExperienceEntry = { company: String, role: String, from: Date, to: Option<Date>, description: Option<String> }

Store as JSONB. Show in Profile as a timeline list with add/edit/remove per entry.
"to" being null means "current position".

## Inspect first
- apps/engine-api/src/domain/ — profile struct and existing JSON field patterns
- apps/engine-api/migrations/ — profiles table
- apps/web/src/pages/Profile.tsx — experience section (may be partially implemented)

## Likely files to modify
- apps/engine-api/src/domain/ (add experience field)
- apps/engine-api/migrations/ (add JSONB column)
- apps/engine-api/src/api/routes/profile.rs
- packages/contracts/src/profiles.ts (add ExperienceEntry type)
- apps/web/src/pages/Profile.tsx (timeline editor UI)

## Rules
- JSONB column, nullable, default empty array.
- from/to are ISO 8601 date strings (year-month level precision is sufficient).
- Entries sorted by from date descending.

## Acceptance criteria
- [ ] experience persists to DB
- [ ] Timeline shows entries sorted by date
- [ ] Add/edit/remove works
- [ ] "Current" badge for entries with no to date
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

## E9 — Profile strength indicator

```
You are working in the Job Copilot monorepo.
First read: CLAUDE.md, apps/web/src/pages/Profile.tsx,
apps/engine-api/src/api/routes/profile.rs (analyze endpoint response),
apps/engine-api/src/api/routes/roles.rs (role catalog)

## Goal
Show a "Profile Strength" indicator on the Profile page that says:
"Your profile is a strong match for X roles in the database."
This calls GET /api/v1/roles to get the role catalog and computes locally
how many roles are a good match based on profile skills + role requirements.

Also show: "You match Y active jobs" — read from the job feed response count.

## Inspect first
- apps/engine-api/src/api/routes/roles.rs — role list response
- apps/engine-api/src/api/routes/jobs.rs — how to get match count
- apps/web/src/pages/Profile.tsx — where to place the indicator

## Likely files to modify
- apps/web/src/pages/Profile.tsx (add strength indicator component)
- apps/web/src/components/ProfileStrength.tsx (create)

## Rules
- Computed on frontend from existing data — no new endpoint.
- Must update after profile save.
- Keep computation simple (skill overlap, not deep ML scoring).

## Acceptance criteria
- [ ] Profile strength indicator visible in Profile page
- [ ] Role count is accurate based on current skills
- [ ] Updates after profile changes

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

## E10 — Profile: CV version history

```
You are working in the Job Copilot monorepo.
First read: CLAUDE.md, apps/engine-api/src/api/routes/resumes.rs,
apps/web/src/pages/Profile.tsx

## Goal
Show the history of uploaded CVs in the Profile page's resume section:
- List of CVs with: filename, upload date, whether it's active
- Each row has: "Activate" button, "Download" link (if download URL exists)
- Timestamps formatted as "Uploaded Apr 22, 2026"

This is display-only — the upload/activate/delete actions already exist,
just improve the history presentation.

## Inspect first
- apps/engine-api/src/api/routes/resumes.rs — list endpoint response shape
- apps/web/src/pages/Profile.tsx — current resume section
- apps/web/src/api/ — resume list client

## Likely files to modify
- apps/web/src/pages/Profile.tsx (improve resume history section)
- apps/web/src/components/ResumeHistory.tsx (create if needed)

## Rules
- Read-only display (actions reuse existing API).
- Active resume has visual distinction (highlighted row or badge).
- Empty state: "No CVs uploaded yet. Upload one above."

## Acceptance criteria
- [ ] CV history shows all uploaded CVs with dates
- [ ] Active CV highlighted
- [ ] "Activate" changes active CV
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
