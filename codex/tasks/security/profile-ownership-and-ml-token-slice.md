# Codex Prompt — Profile Ownership + ML Internal Token Hardening

You are working in the Job Copilot monorepo.

First read:

- `CLAUDE.md`
- `AGENTS.md`
- `codex/CODEX.md`
- `docs/00-master-plan.md`
- `docs/05-roadmap/current-focus.md`
- `docs/engineering-checklist.md`

## Goal

Implement a small security hardening slice:

1. Profile-scoped routes must verify that the authenticated user owns the requested profile.
2. The ML sidecar internal token must be required in production mode.

Keep the change surgical. Do not redesign auth, routing, or ML provider logic.

## Inspect first

Backend:

- `apps/engine-api/src/api/routes/profile.rs`
- `apps/engine-api/src/api/routes/events.rs`
- any profile-scoped route modules such as feedback, saved, whitelist, blacklist
- auth extractor / `AuthUser` definition
- existing route tests for profile, events, feedback, saved, hidden, bad-fit, whitelist, blacklist

ML:

- `apps/ml/app/settings.py`
- `apps/ml/app/api.py`
- existing ML settings/tests

## Backend implementation requirements

- Extend the existing `ensure_profile_exists()` helper or add a nearby helper so it verifies ownership, not only existence.
- Compare `AuthUser.profile_id` with path `{id}` for profile-scoped routes.
- Return `403` on ownership mismatch.
- Preserve `404` for missing profile where the existing behavior expects missing resource.
- Apply the ownership guard consistently to:
  - profile read/update/analyze/build-search-profile routes
  - `log_user_event()` in events routes
  - feedback/list routes scoped to `/profiles/{id}/...`
  - saved/hidden/bad-fit/whitelist/blacklist routes if present
- Do not invent a new auth model. Use existing `AuthUser` and existing error patterns.
- Add tests for at least:
  - owner can access profile-scoped route
  - non-owner gets `403`
  - missing profile still returns expected missing-resource behavior
  - event logging rejects mismatched profile owner

## ML implementation requirements

- In `apps/ml/app/settings.py`, do not allow production startup with missing internal token.
- Keep local/dev behavior convenient if the current settings already distinguish production from development.
- Raise a clear `ValueError` during settings/startup validation when production mode requires the token and it is absent.
- Do not print token values.
- Add or update tests if ML settings tests exist.

## Acceptance criteria

- [ ] All profile-scoped engine-api routes checked in this slice reject owner mismatch with `403`.
- [ ] Existing valid owner flows still work.
- [ ] Missing profile behavior is preserved.
- [ ] Production ML settings fail fast without internal token.
- [ ] No secrets are logged or exposed.
- [ ] No unrelated auth redesign or route refactor.

## Verification commands

Run relevant commands and report results:

```bash
cargo test --manifest-path apps/engine-api/Cargo.toml
cargo check --manifest-path apps/engine-api/Cargo.toml
cd apps/ml && python -m pytest
```

If ML tests are not available or cannot run locally, explain why and at least run the import/settings test path that exists in the repo.

## Final response format

```md
## What changed

## Why

## Files changed

## Verification

## Notes / follow-ups
```
