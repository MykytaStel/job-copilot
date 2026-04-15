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

- `job_impression`
- `job_opened`
- `fit_explanation_requested`
- `application_coach_requested`
- `cover_letter_draft_requested`
- `interview_prep_requested`
- `application_created`

## API surface

- `POST /api/v1/profiles/:id/events`
- `GET /api/v1/profiles/:id/events/summary`
- `GET /api/v1/profiles/:id/behavior-summary`
- `GET /api/v1/profiles/:id/funnel-summary`

The explicit endpoint exists for flows that do not naturally pass through `engine-api` business routes, such as ML enrichment requests initiated from web.
It now also captures conservative web impression writes for dashboard cards and ranked search results when a profile context exists.

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

## Funnel analytics v1

The funnel summary endpoint derives a first end-to-end job funnel directly from `user_events`:

- `impression_count`
- `open_count`
- `save_count`
- `hide_count`
- `bad_fit_count`
- `application_created_count`
- `fit_explanation_requested_count`
- `application_coach_requested_count`
- `cover_letter_draft_requested_count`
- `interview_prep_requested_count`

It also exposes defensive conversion ratios:

- `open_rate_from_impressions`
- `save_rate_from_opens`
- `application_rate_from_saves`

And a simple source breakdown when source metadata exists:

- `impressions_by_source`
- `opens_by_source`
- `saves_by_source`
- `applications_by_source`

## Behavior-aware personalization v2

`engine-api` now derives explicit behavior aggregates from `user_events` for:

- save / hide / bad-fit counts by source
- save / hide / bad-fit counts by role family
- application-created counts by source and role family when available
- search run count

Job-scoped feedback writes and explicit job events now auto-fill `role_family` when it can be inferred from canonical Rust-side role signals. That keeps behavior analytics and ranking grounded in the same deterministic domain layer.

Search ranking consumes these aggregates as a small additive layer:

- deterministic fit remains the base score
- repeated positive source or role-family signals can add a small boost
- repeated hide / bad-fit source or role-family signals can add a small penalty
- fit reasons include explicit behavior explanations when those adjustments fire

## Learned reranker v1

`engine-api` now applies a conservative learned reranker after deterministic ranking, explicit feedback scoring, and behavior-aware personalization, but before final search truncation.

The learned layer is still deterministic and inspectable. It builds explicit features from accumulated behavior and funnel signals:

- source positive / negative signal
- role-family positive / negative signal
- save / bad-fit / application history strength
- source funnel quality hint
- deterministic score bucket

The final score remains the canonical Rust fit score plus a bounded learned delta, clamped to the normal score range. Learned reasons are emitted separately from deterministic and LLM explanations with `Learned reranker ...` text.

The layer is gated by `LEARNED_RERANKER_ENABLED` and defaults to enabled. `/api/v1/search/run` meta includes:

- `learned_reranker_enabled`
- `learned_reranker_adjusted_jobs`

## Outcome dataset + offline evaluation v1

`GET /api/v1/profiles/:id/reranker-dataset` exports profile-scoped labeled examples for future reranker evaluation and training. It is read-only and does not change live ranking.

Label policy is documented in `docs/03-domain/reranker-outcomes.md`:

- `positive` = `application_created`
- `medium` = saved
- `negative` = bad-fit / hidden

`apps/ml/app/reranker_evaluation.py` can compare deterministic, behavior-adjusted, and learned-reranker-adjusted score orderings with simple offline metrics.
