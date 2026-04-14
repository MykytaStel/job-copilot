# Job Feedback

First explicit feedback layer for the current profile context.

## Supported states

### Job-level
- `saved`
- `hidden`
- `bad_fit`

### Company-level
- `whitelist`
- `blacklist`

## Persistence

Profile-scoped tables in `engine-api`:
- `profile_job_feedback`
  - one row per `(profile_id, job_id)`
  - boolean flags for `saved`, `hidden`, `bad_fit`
- `profile_company_feedback`
  - one row per `(profile_id, normalized_company_name)`
  - single `status` value: `whitelist` or `blacklist`

Company list status is last-write-wins for a given normalized company name.

Feedback writes also emit immutable `user_events` records so future personalization can learn from actions without changing feedback as the mutable source of truth.

## Search rules in this slice

Deterministic search now applies profile feedback before scoring:
- hidden jobs are excluded
- blacklisted companies are excluded
- saved state is returned in job response feedback
- bad fit is stored and returned, but does not change rank yet

## Jobs feed / UI rules

- recent jobs can be requested with `profile_id`
- hidden jobs are omitted from the recent jobs feed for that profile context
- job/detail responses include a `feedback` object so the UI can render badges and actions

## API shape

Explicit profile-scoped endpoints:
- `GET /api/v1/profiles/:id/feedback`
- `PUT /api/v1/profiles/:id/jobs/:job_id/saved`
- `PUT /api/v1/profiles/:id/jobs/:job_id/hidden`
- `PUT /api/v1/profiles/:id/jobs/:job_id/bad-fit`
- `PUT/DELETE /api/v1/profiles/:id/companies/whitelist`
- `PUT/DELETE /api/v1/profiles/:id/companies/blacklist`
