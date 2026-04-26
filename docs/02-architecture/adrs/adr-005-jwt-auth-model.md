# ADR-005: JWT auth model carries profile ownership context

## Status

Partially Implemented

## Context

Job Copilot is currently a single-user system, but the engine-api must enforce that a
caller can only read and write their own profile data. Without ownership checks, any
authenticated request can read or mutate any profile record simply by supplying a
different profile ID in the path.

A lightweight ownership model is needed that:
- does not require a full RBAC system;
- works for the current single-owner-per-profile structure;
- is enforceable at the route handler level in engine-api.

## Decision

JWTs carry profile ownership context. Engine-api uses this context to enforce
per-route ownership checks.

**Token format:**
- Algorithm: HS256
- Secret: `JWT_SECRET` environment variable on engine-api
- Claims: `sub` contains the caller's `profile_id` (string UUID); `exp` is the
  standard expiry claim

**AuthUser:**
After successful validation, engine-api middleware inserts an `AuthUser` struct:
```
AuthUser {
    profile_id: String,  // from `sub` claim
}
```

**Profile ownership rule:**
Every route scoped to `/profiles/{id}/...` must verify:
```
AuthUser.profile_id == path {id}
```
- Ownership mismatch → `403 Forbidden`
- Missing profile → `404 Not Found` (ownership check does not change missing-resource semantics)
- No auth configured (dev mode, `JWT_SECRET` absent) → check is skipped

**Dev-mode passthrough:**
When `JWT_SECRET` is absent or empty, the middleware logs a warning and passes all
requests through. This is intentional for local development.

**Production requirement:**
A missing `JWT_SECRET` must be rejected at startup in a production deployment. This
guard is not yet enforced.

## Consequences

**Easier:**
- Ownership is checked entirely within engine-api. ML and web do not need to reason
  about authorization.
- Adding a new profile-scoped route is a single call to `check_profile_ownership`
  in the handler.
- The rule is simple and auditable: compare one UUID from the token with one UUID in
  the path.

**Harder:**
- Every new profile-scoped route must explicitly apply the ownership check; forgetting
  it creates a silent authorization gap.
- JWT secret rotation requires reissuing all active tokens.
- The model does not support multi-user or collaborative access to a single profile
  without a future auth expansion.

**Constraints created:**
- Do not expand to multi-user auth before the product flow is stable.
- Do not add `email`, `role`, or other claims to the token without updating this ADR
  and the security model.
- `JWT_SECRET` must never be logged, exposed in responses, or included in error messages.
- ML internal token is separate from the user JWT. See [security-model.md](../security-model.md).

## Current State

**Implemented:**
- JWT middleware in engine-api validates `sub` claim and inserts `AuthUser`.
- Dev-mode passthrough when `JWT_SECRET` is absent.
- `check_profile_ownership` helper exists in
  `apps/engine-api/src/api/middleware/auth.rs` and is unit-tested.

**Partial / gaps — open security issues:**
- Profile ownership guard is **not yet applied** to all profile-scoped route handlers.
  The helper exists but is not wired into every handler. This is the next security slice.
- `JWT_SECRET` absence is **not rejected** in production mode at startup.
- ML internal token production startup validation is not reliably enforced.

See [security-model.md](../security-model.md) for the full gap list and the linked
Codex task for the next implementation slice.

## Related Docs

- [security-model.md](../security-model.md)
- [ADR-001: Rust domain authority](adr-001-rust-domain-authority.md)
- [Codex task: profile ownership and ML token](../../../codex/tasks/security/profile-ownership-and-ml-token-slice.md)
