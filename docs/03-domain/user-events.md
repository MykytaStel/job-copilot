# User Events

Immutable profile-scoped event log for learning signals and future personalization.

## Purpose

The event layer captures user behavior without changing canonical ranking or feedback truth:

- Rust / `engine-api` stays the source of truth for event validation and persistence
- events are append-only learning signals, not replacements for feedback tables
- future analytics and personalization can build from this history without moving logic into UI or LLM prompts

## Canonical event types in v1

- `job_impression`
- `job_opened`
- `job_saved`
- `job_unsaved`
- `job_hidden`
- `job_unhidden`
- `job_bad_fit`
- `job_bad_fit_removed`
- `company_whitelisted`
- `company_blacklisted`
- `search_run`
- `fit_explanation_requested`
- `application_coach_requested`
- `cover_letter_draft_requested`
- `interview_prep_requested`
- `application_created`

## Persistence

`engine-api` stores events in `user_events`:

- `id`
- `profile_id`
- `event_type`
- `job_id` nullable
- `company_name` nullable
- `source` nullable
- `role_family` nullable
- `payload_json` nullable `jsonb`
- `created_at`

The table is intentionally simple and queryable. It keeps event history separate from mutable feedback state.

## Write points in v1

Logged directly inside `engine-api`:

- save / unsave
- hide / unhide
- bad fit / remove bad fit
- company whitelist / blacklist
- `search_run`

Logged through explicit `engine-api` event endpoint from web:

- `job_opened`
- `fit_explanation_requested`
- `application_coach_requested`
- `cover_letter_draft_requested`
- `interview_prep_requested`
- `application_created`

## API surface

- `POST /api/v1/profiles/:id/events`
- `GET /api/v1/profiles/:id/events/summary`

The explicit endpoint exists for flows that do not naturally pass through `engine-api` business routes, such as ML enrichment requests initiated from web.

## Analytics-ready summary in v1

The summary endpoint exposes raw event counts for:

- `save_count`
- `hide_count`
- `bad_fit_count`
- `search_run_count`
- `fit_explanation_requested_count`
- `application_coach_requested_count`
- `cover_letter_draft_requested_count`
- `interview_prep_requested_count`

These are event counts, not current-state counts.
