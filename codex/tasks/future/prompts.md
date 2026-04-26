# Block K — Future / Pre-Auth (6 tasks)

These tasks are planned for AFTER the product is stable and user-tested.
Do not implement until explicitly requested.

---

## K1 — Redis integration for caching

```
You are working in the Job Copilot monorepo.
First read: CLAUDE.md, apps/engine-api/src/main.rs (AppState),
apps/ml/app/scoring_routes.py (rerank cache),
infra/docker-compose.yml

## Goal
Add Redis as a caching layer:
- Engine-API: cache job feed results per profile (TTL 2min), invalidated on feedback
- ML Sidecar: replace in-memory rerank cache with Redis (TTL 5min, per-profile key)
- Add Redis service to docker-compose.yml

Backend dependencies: redis crate for Rust, redis-py for Python.
Connection pool in AppState.

## Inspect first
- apps/engine-api/src/main.rs — AppState and dependency injection
- apps/ml/app/scoring_routes.py — current in-memory cache
- infra/docker-compose.yml — where to add redis service

## Likely files to modify
- apps/engine-api/Cargo.toml (add redis dependency)
- apps/engine-api/src/main.rs (add Redis pool to AppState)
- apps/engine-api/src/api/routes/jobs.rs (add cache layer)
- apps/ml/requirements.txt (add redis)
- apps/ml/app/scoring_routes.py (replace in-memory with Redis)
- infra/docker-compose.yml (add redis service)

## Rules
- Redis is optional dependency — if REDIS_URL not set, fall through to no-cache.
- Never cache PII or sensitive data.
- Cache keys must be namespaced: job-copilot:{type}:{profile_id}.

## Acceptance criteria
- [ ] Redis service in docker-compose.yml
- [ ] Engine-API uses Redis for job feed cache (when REDIS_URL set)
- [ ] ML sidecar uses Redis for rerank cache (when REDIS_URL set)
- [ ] Cache invalidated after feedback changes
- [ ] Graceful fallback when Redis unavailable

## Verification commands
cargo test --manifest-path apps/engine-api/Cargo.toml
cd apps/ml && python -m pytest

## Final response format
## What changed
## Why
## Files changed
## Verification
## Notes / follow-ups
```

---

## K2 — JWT auth multi-user (full implementation)

```
You are working in the Job Copilot monorepo.
First read: CLAUDE.md, apps/engine-api/src/api/routes/auth.rs,
apps/engine-api/src/domain/ (user/auth domain),
apps/web/src/pages/Auth.tsx, apps/web/src/api/auth.ts

## Goal
Implement full multi-user JWT authentication:
1. User registration (POST /api/v1/auth/register): email + password → create user + profile
2. User login (POST /api/v1/auth/login): email + password → JWT access token (15min) + refresh token (7d)
3. Token refresh (POST /api/v1/auth/refresh): refresh token → new access token
4. All protected routes require valid JWT
5. Web: login/register page, token storage in httpOnly cookie or localStorage,
   auto-refresh on 401

This task should only be implemented when moving to multi-user.

## Inspect first
- apps/engine-api/src/api/routes/auth.rs — existing auth implementation
- apps/engine-api/src/domain/ — user model, password hashing
- apps/web/src/pages/Auth.tsx — current auth UI

## Likely files to modify
- apps/engine-api/src/api/routes/auth.rs (add refresh token endpoint)
- apps/engine-api/migrations/ (add refresh_tokens table)
- apps/web/src/api/auth.ts (add token refresh interceptor)
- apps/web/src/pages/Auth.tsx (registration form)

## Rules
- Passwords hashed with argon2id (not bcrypt).
- JWT secret from environment variable (never hardcoded).
- Refresh tokens stored in DB (revocable).
- httpOnly cookie preferred for token storage.

## Acceptance criteria
- [ ] Registration creates user + profile
- [ ] Login returns access + refresh tokens
- [ ] Refresh endpoint issues new access token
- [ ] Web auto-refreshes tokens transparently
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

## K3 — Multi-profile support

```
You are working in the Job Copilot monorepo.
First read: CLAUDE.md, apps/engine-api/src/domain/ (profile/user relationship),
apps/engine-api/migrations/ (profiles, users tables),
apps/web/src/AppShell.tsx (profile switcher area)

## Goal
Allow one user account to have multiple candidate profiles (e.g. "Backend Engineer"
vs "Fullstack Engineer" persona). Add profile switcher in the sidebar.

Backend:
- Users table has many profiles (one-to-many, not one-to-one).
- GET /api/v1/profiles — list profiles for current user.
- POST /api/v1/profiles — create new profile.
- PATCH /api/v1/users/active-profile — set active profile ID (stored in users.active_profile_id).
- All job feed / feedback / application requests use active_profile_id from JWT context.

Web:
- Sidebar profile switcher dropdown.
- "New Profile" option in dropdown.
- Profile badge shows active profile name.

## Inspect first
- apps/engine-api/src/domain/ — user/profile relationship
- apps/engine-api/migrations/ — foreign keys
- apps/web/src/AppShell.tsx — sidebar

## Likely files to modify
- apps/engine-api/migrations/ (alter users table, add active_profile_id)
- apps/engine-api/src/api/routes/profile.rs (add list/create endpoints)
- apps/engine-api/src/domain/ (update AuthUser to include active_profile_id)
- apps/web/src/AppShell.tsx (profile switcher)

## Rules
- This is a significant architectural change — requires careful migration.
- All existing routes must work with the active_profile_id from JWT.
- Default: existing single profile becomes the active profile.

## Acceptance criteria
- [ ] Users can have multiple profiles
- [ ] Active profile switchable from sidebar
- [ ] All data (feedback, applications, feed) scoped to active profile
- [ ] Existing single-profile behavior unchanged

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

## K4 — Stripe integration (paid tier)

```
You are working in the Job Copilot monorepo.
First read: CLAUDE.md, apps/engine-api/src/api/routes/,
apps/web/src/pages/Settings.tsx

## Goal
Add Stripe subscription integration for a "Pro" tier:
- Free tier: template enrichment, basic ranking, limited market data
- Pro tier: Ollama/OpenAI enrichment, CV tailoring, advanced analytics, unlimited history

Backend:
- POST /api/v1/billing/checkout — create Stripe Checkout session
- POST /api/v1/billing/webhook — handle Stripe webhook (subscription.created, .updated, .deleted)
- GET /api/v1/billing/subscription — current subscription status

Web:
- Upgrade prompt in Settings
- Pro badge in sidebar for paid users
- Feature gates for Pro-only features

## Rules
- Stripe secret key from environment variable only.
- Webhook endpoint verifies Stripe signature.
- Tier stored in users.subscription_tier (free|pro).
- Pro features gate on subscription_tier check in middleware.

## Acceptance criteria
- [ ] Checkout session creation works
- [ ] Webhook correctly updates tier on payment success
- [ ] Pro features blocked for free tier users (403 with upgrade prompt)
- [ ] Settings shows current plan + upgrade button

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

## K5 — Ollama provider production setup

```
You are working in the Job Copilot monorepo.
First read: CLAUDE.md, apps/ml/app/settings.py,
apps/ml/app/llm_provider_factory.py,
infra/docker-compose.yml

## Goal
Set up the Ollama provider for production use with Mistral 7B:
1. Add Ollama service to docker-compose.yml (using ollama/ollama image)
2. Add OLLAMA_MODEL=mistral env var to settings
3. Add a model pull command to docker-compose (or startup script)
4. Configure ML sidecar to use Ollama when OLLAMA_BASE_URL is set
5. Add health check that verifies Ollama model is loaded

## Inspect first
- apps/ml/app/llm_provider_factory.py — OllamaProvider implementation
- apps/ml/app/settings.py — OLLAMA_BASE_URL, ML_LLM_PROVIDER
- infra/docker-compose.yml — service definitions

## Likely files to modify
- infra/docker-compose.yml (add ollama service)
- apps/ml/app/settings.py (add OLLAMA_MODEL setting)
- apps/ml/app/ (improve Ollama provider error handling)

## Rules
- Ollama is optional — ML sidecar falls back to template if Ollama unreachable.
- Model download (~4GB) happens on first start — document this.
- Timeout for Ollama calls: 30 seconds.

## Acceptance criteria
- [ ] docker-compose.yml has ollama service
- [ ] ML sidecar uses Ollama when OLLAMA_BASE_URL configured
- [ ] Graceful fallback to template if Ollama down
- [ ] /ready reports Ollama status
- [ ] Documentation updated

## Verification commands
cd apps/ml && python -m pytest

## Final response format
## What changed
## Why
## Files changed
## Verification
## Notes / follow-ups
```

---

## K6 — Rate limiting per subscription tier

```
You are working in the Job Copilot monorepo.
First read: CLAUDE.md, apps/engine-api/src/main.rs (middleware stack),
apps/engine-api/src/api/routes/ (enrichment proxy routes)

## Goal
Add rate limiting middleware to engine-api:
- Free tier: 100 API calls/hour, 10 ML enrichment calls/day
- Pro tier: 1000 API calls/hour, 100 ML enrichment calls/day
- No-auth (health endpoints): no limit

Use a token bucket or sliding window approach. Store rate limit state in Redis
(K1 must be done first) or in-memory per process as a fallback.

Return 429 Too Many Requests with Retry-After header when limit exceeded.

## Inspect first
- apps/engine-api/src/main.rs — middleware stack
- apps/engine-api/src/domain/ — AuthUser struct (has tier info after K4)
- Cargo.toml — check for tower-governor or similar rate limit crate

## Likely files to modify
- apps/engine-api/Cargo.toml (add rate limiting crate)
- apps/engine-api/src/main.rs (add rate limiting middleware)
- apps/engine-api/src/middleware/ (new rate_limit.rs module)

## Rules
- Rate limits must be per-user, not per-IP.
- 429 response must include Retry-After: <seconds> header.
- Limits configurable via env vars (RATE_LIMIT_FREE_HOURLY=100).
- Enrichment endpoints have stricter daily limits than general API.

## Acceptance criteria
- [ ] Free tier limited to 100 req/hour
- [ ] Pro tier limited to 1000 req/hour
- [ ] 429 returned with Retry-After when limit hit
- [ ] Rate limit headers (X-RateLimit-Remaining) in all responses
- [ ] Cargo tests pass

## Verification commands
cargo test --manifest-path apps/engine-api/Cargo.toml
cargo check --manifest-path apps/engine-api/Cargo.toml

## Final response format
## What changed
## Why
## Files changed
## Verification
## Notes / follow-ups
```
