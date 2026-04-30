# Security Model — 2026-04-26

This document describes the current authentication and authorization model for Job Copilot.
It is not a full security program description; rate limiting, penetration testing, and
infrastructure hardening are out of scope here.

---

## JWT Authentication

Engine-api is the sole authority for validating JWTs.
No other service validates tokens from the browser.

### Token format

- Algorithm: HS256
- Secret: configured via `JWT_SECRET` environment variable on engine-api
- Claims used:
  - `sub` — contains the `profile_id` of the authenticated user (string UUID)
  - `exp` — expiry timestamp (standard JWT claim, validated by the library)
- Claims not present: no separate `email` or `role` claim in the current implementation;
  `sub` is the only identity claim extracted into `AuthUser`

### AuthUser

After successful token validation, the middleware inserts an `AuthUser` struct into
request extensions:

```rust
pub struct AuthUser {
    pub profile_id: String,  // derived from `sub` claim
}
```

Route handlers extract `AuthUser` from extensions to determine the caller's identity.

### Dev-mode passthrough

When `JWT_SECRET` is absent or empty, the auth middleware logs a warning and passes
all requests through without validation. This is intentional for local development.

Production startup fails fast when `JWT_SECRET` is absent, empty, or still set to the
Docker Compose dev placeholder.

---

## Profile Ownership Rule

Every engine-api route scoped to `/profiles/{id}/...` must verify that the
authenticated caller owns that profile before returning data or accepting writes.

**Rule:** `AuthUser.profile_id == path {id}`

- Ownership mismatch → `403 Forbidden` (`profile_access_denied`)
- Missing profile → `404 Not Found` (ownership check does not change missing-resource semantics)
- No auth configured (dev mode) → check is skipped

The `check_profile_ownership` helper in
`apps/engine-api/src/api/middleware/auth.rs` implements this comparison and is available
to all route handlers.

The helper is wired into the current profile-scoped route set. New routes that add a
`/profiles/{id}/...` scope must call the same helper or an equivalent ownership guard.

---

## ML Internal Token

Engine-api communicates with the ML sidecar over HTTP using an internal token.

- Engine-api sends the token in requests to ML internal endpoints.
- ML validates it to restrict access to internal-only routes.
- The token value is never logged or exposed in responses.

ML production startup now fails fast if `ML_INTERNAL_TOKEN` is absent.
The `validate_startup_security()` method in `apps/ml/app/settings.py` is called from
the FastAPI lifespan in `apps/ml/app/api.py`. The `environment` field is normalized
to lowercase before the check, so values like `"Production"` are handled correctly.
This is covered by tests in `apps/ml/tests/test_runtime_config.py`.

---

## Intentionally Out of Scope

The following are not part of the current security model and should not be assumed
to exist:

- Full multi-user RBAC (role-based access control)
- Billing / Stripe access controls
- Rate limiting on any endpoint
- OAuth / social login
- Per-resource ACLs beyond single-owner profile scoping

---

## Summary of Known Gaps

| Gap | Status |
|-----|--------|
| Profile ownership guard not applied to all profile-scoped routes | Implemented for current route set; keep auditing new routes |
| ML internal token production validation not fail-fast | Enforced — tested |
| `JWT_SECRET` absence not rejected in production mode | Enforced; dev placeholder is rejected in production |
