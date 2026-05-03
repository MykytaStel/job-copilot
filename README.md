# Job Copilot

> Candidate intelligence system for Ukrainian and international job markets.
> Ingests, ranks, explains, and tracks jobs — learning from your behavior over time.

Job Copilot is not a job board. It understands the candidate: ingests and normalizes jobs from multiple sources, ranks them by fit, explains why each job is good or bad, supports the full application workflow, and improves with every saved, hidden, or applied action.

---

## What it does

| Area | What's shipped |
|------|---------------|
| **Ingestion** | 4 live scrapers: Djinni, Work.ua, Dou.ua, Robota.ua. Dedup, lifecycle, quality scoring, salary normalization |
| **Ranking** | Deterministic fit score (skills 40%, salary 15%, recency 10%, signals 35%). Learned + trained reranker optional |
| **Fit explanation** | Skill match, seniority, work mode, language, salary breakdown — per job |
| **Feedback** | Save, hide, bad-fit, whitelist, blacklist. All signals feed the learning loop |
| **Application board** | Kanban: Saved → Applied → Interview → Offer → Rejected/Ghosted |
| **CV tailoring** | Upload CV → match against job → ATS keyword gaps + evidence-based suggestions |
| **Enrichment (LLM)** | Fit explanation, cover letter, interview prep, weekly guidance. Template provider by default (no paid API) |
| **Market intelligence** | Skill demand, salary trends, top hiring companies, role demand by seniority |
| **Learning loop** | User events → labeled outcomes → reranker dataset → logistic regression reranker |
| **Search profile** | Region, work mode, roles, languages, salary range — persisted per profile |

---

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                        Browser / UI                             │
│                  React 19 + Vite (apps/web)                     │
└───────────────────────────┬─────────────────────────────────────┘
                            │ REST
        ┌───────────────────▼───────────────────┐
        │         Engine API (apps/engine-api)   │
        │   Rust · Axum · SQLx · PostgreSQL       │
        │   canonical domain · scoring · auth     │
        └───────┬─────────────────────┬──────────┘
                │ SQL                 │ HTTP (internal)
    ┌───────────▼──────┐   ┌──────────▼──────────────┐
    │    PostgreSQL     │   │   ML Sidecar (apps/ml)  │
    │   (migrations)    │   │   Python · FastAPI       │
    └───────────────────┘   │   enrichment · reranker  │
                            └─────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│                   Ingestion (apps/ingestion)                    │
│         Rust · scrape · normalize · dedupe · upsert             │
│         Djinni · Work.ua · Dou.ua · Robota.ua                   │
└─────────────────────────────────────────────────────────────────┘
```

**Key rules:**
- `engine-api` owns all canonical domain state. Nothing else writes authoritative data.
- `apps/ml` is a read-only enrichment sidecar. It never writes to Postgres.
- `apps/web` is an operator UI. No domain logic, no scoring, no ownership checks.

---

## Quick start

**Option A — Docker (recommended, everything included):**

```bash
cp .env.example .env        # or create .env with DATABASE_URL
pnpm docker:up              # starts engine-api, ml, postgres, nginx
```

**Option B — local dev (Postgres required):**

```bash
pnpm install                # install Node workspace deps
pnpm db:up                  # start Postgres in Docker
pnpm dev                    # start engine-api + web concurrently
```

Then open [http://localhost:5173](http://localhost:5173).

**Seed some jobs:**

```bash
pnpm scrape:djinni          # scrape 3 pages from Djinni
pnpm scrape:workua          # scrape 3 pages from Work.ua
# or run the full daemon:
pnpm scrape:daemon
```

---

## Services

| Service | Path | Default port | Language |
|---------|------|-------------|----------|
| Engine API | `apps/engine-api` | `8080` | Rust |
| Ingestion | `apps/ingestion` | CLI / daemon | Rust |
| ML sidecar | `apps/ml` | `8000` | Python |
| Web UI | `apps/web` | `5173` (dev) | TypeScript |
| Shared contracts | `packages/contracts` | — | TypeScript |

See each service's `README.md` for full env var reference and endpoint docs.

---

## Feature status

### A. Ingestion & sources
- [x] Djinni, Work.ua, Dou.ua, Robota.ua scrapers
- [x] Deduplication across sources
- [x] Job lifecycle (active / inactive / reactivated)
- [x] Salary normalization (USD / UAH / EUR)
- [x] Job quality score, description quality score
- [x] Daemon mode (all sources, configurable interval)
- [ ] ATS source detection (Greenhouse, Lever, Workday)
- [ ] Salary confidence score
- [ ] PLN support
- [ ] Company career page connector
- [ ] Manual source import by URL

### B. Ranking & fit
- [x] Deterministic fit score (skills, seniority, salary, remote, language, recency)
- [x] Explainable score breakdown per job
- [x] Learned reranker (heuristic from feedback)
- [x] Trained reranker (logistic regression on outcome dataset)
- [ ] Job buckets: "Apply immediately" / "Good but not now" / "Skip"
- [ ] Red flag detection (too vague, recruiter spam, low signal)

### C. Candidate learning
- [x] Saved / hidden / bad-fit signals feed ranking
- [x] Search profile: region, work mode, roles, salary, languages
- [ ] "What I learned about you" UI
- [ ] Confidence + evidence per preference
- [ ] Manual correction ("No, this is wrong")

### D. LLM enrichment
- [x] Fit explanation, cover letter draft, interview prep, weekly guidance
- [x] Template provider (no paid API by default), OpenAI + Ollama switchable
- [x] CV tailoring endpoint
- [ ] CV tailoring web entrypoint (endpoint exists, UI pending)
- [ ] "Tell me why not to apply"

### E. CV & resume tools
- [x] CV upload + activation, multi-resume support
- [x] CV vs job match analysis
- [ ] CV quality score UI
- [ ] Missing keyword suggestions UI
- [ ] CV version history

### F. Application tracker
- [x] Kanban: Saved → Applied → Interview → Technical → Offer → Rejected / Ghosted
- [x] Notes, due dates, contacts
- [ ] Follow-up reminders, calendar integration

### G. Market intelligence
- [x] Skill demand, salary trends, top companies, role demand
- [ ] "Best source for me", "Best country for me"
- [ ] "Skills I should learn next", "Jobs I am almost qualified for"
- [ ] Analytics freshness widget

### H. UX / product polish
- [x] Notifications (DB, API, web)
- [x] Global search (Cmd/Ctrl+K)
- [x] Dark base theme
- [ ] Onboarding wizard
- [ ] "Today's best jobs" section
- [ ] Demo mode

---

## Running checks

```bash
# TypeScript / web
pnpm typecheck
pnpm --dir apps/web lint
pnpm --dir apps/web test

# Rust
pnpm fmt:rust
pnpm check:rust
pnpm lint:rust
pnpm test:rust

# Python
cd apps/ml && ruff check .
cd apps/ml && mypy app/ --ignore-missing-imports
cd apps/ml && python -m pytest

# E2E smoke
pnpm smoke:e2e
```

Or use the shorthand scripts:

```bash
pnpm lint           # all TS workspaces
pnpm lint:rust      # clippy on both Rust crates
pnpm lint:python    # ruff + mypy on ML sidecar
```

---

## Git hooks

[lefthook](https://github.com/evilmartians/lefthook) runs checks automatically before commit and push.

```bash
# Install hooks (run once after cloning)
pnpm hooks:install
# or: lefthook install
```

**pre-commit** (fast, runs on changed files only): `cargo fmt --check`, `eslint`, `ruff`

**pre-push** (full): `cargo clippy -D warnings` on both Rust crates, `cargo test` on both

---

## Contributing

See [docs/02-architecture/current-state.md](docs/02-architecture/current-state.md) for architecture decisions and current state.

See [docs/05-roadmap/current-focus.md](docs/05-roadmap/current-focus.md) for the active roadmap.

Work is organized in bounded implementation slices — see [CLAUDE.md](CLAUDE.md) and [codex/CODEX.md](codex/CODEX.md) for the working model.

**When adding a new ingestion source:**
1. Implement `SourceAdapter` trait in `apps/ingestion/src/scrapers/`
2. Register in the source registry
3. Add fixture/smoke test

**When adding a scoring signal:**
1. Add to `apps/engine-api/src/services/ranking/`
2. Keep it testable in isolation
3. Update score explanation output
