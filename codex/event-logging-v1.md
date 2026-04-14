Implement the next slice:

**event logging + learning signals v1**

Context:
- deterministic profile analysis exists
- search-profile build exists
- deterministic ranked search exists
- feedback persistence exists
- analytics + LLM context exist
- additive ML enrichments exist
- now we need a learning foundation so the system can improve from user behavior over time

Goal:
Add the first event logging pipeline and expose reusable learning signals for future personalization and analytics.

Scope:
Primarily `apps/engine-api`, with minimal `apps/web` wiring where necessary.
Do not add LLM calls in this task.
Do not redesign the app.

What to implement:

1. Add a new event domain
Suggested domain:
- `src/domain/event/`
Need:
- EventType enum
- ProfileEvent model
- event metadata payload (keep it explicit and JSON-friendly)

Minimum event types:
- search_run
- search_profile_built
- job_opened
- source_opened
- saved
- unsaved
- hidden
- unhidden
- bad_fit
- unbad_fit
- company_whitelisted
- company_blacklisted
- fit_explanation_requested
- application_coach_requested
- cover_letter_requested

2. Add persistence
Create a DB migration for a `profile_events` table.

Suggested fields:
- id
- profile_id
- event_type
- job_id nullable
- source_id nullable
- normalized_company_name nullable
- occurred_at
- search_profile_snapshot jsonb nullable
- deterministic_fit_snapshot jsonb nullable
- metadata jsonb nullable

Keep it simple and explicit.

3. Add repository + service
Need:
- create event
- list recent events for profile
- optionally list by event_type
Keep API small.

4. Log events from existing flows
At minimum log:
- search run
- save / unsave
- hide / unhide
- bad_fit / unbad_fit
- company whitelist / blacklist
- fit explanation requested
- application coaching requested
- cover letter requested
- job/source open if a clean hook already exists

5. Add a lightweight profile learning summary
Backend should expose a small deterministic summary for a profile, such as:
- total_events
- saves_count
- hides_count
- bad_fit_count
- top_sources_by_positive_actions
- top_sources_by_negative_actions
- top_companies_by_positive_actions
Keep it small and derived from events.

6. Add endpoint(s)
Suggested:
- `GET /api/v1/profiles/{id}/events`
- `GET /api/v1/profiles/{id}/learning-summary`

7. Minimal frontend integration
Only if clean and small:
- wire event logging for fit explanation / coaching / cover letter actions
- optionally add a small learning summary block in Analytics
Do not redesign the UI.

8. Tests
Add backend tests for:
- event persistence
- event creation from feedback actions
- learning summary aggregation
- profile-scoped reads

Constraints:
- no LLM changes in this task
- no fine-tuning
- no ingestion rewrite
- keep code explicit and readable
- Rust remains source of truth

Acceptance criteria:
- profile events are persisted
- key user actions are logged
- a deterministic learning summary exists
- events can be reused for future personalization and analytics
