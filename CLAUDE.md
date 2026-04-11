# CLAUDE.md — Job Copilot UA

You are working on **Job Copilot UA**, a personal AI-assisted job search platform for Ukrainian tech candidates.

---

## Product goal

Help a candidate:
- Organize their master profile and CV
- Ingest job postings from Ukrainian job sites (Djinni, Work.ua, Robota.ua) or pasted text
- Understand fit and skill gaps vs each job
- Tailor CV content using AI grounded in the candidate's real experience
- Track applications, prep interviews, manage offers

---

## Current state (April 2026)

**Built and working:**
- engine-api health endpoint
- jobs recent/listing endpoints
- applications recent/listing endpoints
- persisted profile CRUD in engine-api
- persisted profile analysis and search-profile build flows
- web connected to engine-api for dashboard, job details, applications board, and profile

**What's still stubbed:**
- application write flows in engine-api
- resume ingestion/upload flow
- matching and fit-score persistence
- search, alerts, contacts, tasks, offers, backup, and other legacy-only tools

---

## Architecture

```
job-copilot-ua-starter/
├── apps/
│   ├── engine-api/   ← Rust + Axum + SQLx + Postgres
│   │   ├── src/
│   │   │   ├── api/          ← routes, DTOs, error contracts
│   │   │   ├── db/           ← database and repositories
│   │   │   ├── domain/       ← explicit domain models
│   │   │   ├── services/     ← use-case services
│   │   │   └── main.rs       ← server entry
│   └── web/          ← React 19 + Vite + React Router 7
│       └── src/
│           ├── App.tsx       ← route definitions
│           ├── Layout.tsx    ← app shell
│           ├── api.ts        ← typed fetch client for engine-api
│           └── pages/        ← active engine-api-backed screens
├── packages/
│   └── shared/        ← shared TypeScript types
│       └── src/index.ts
└── docs/              ← product decisions, roadmap
```

**Port conventions:** Engine API = 8080, Web = 5173

---

## DB tables (current engine-api scope)

| Table | Primary concern |
|-------|----------------|
| `profiles` | Candidate master profile + persisted analysis snapshot |
| `jobs` | Canonical jobs read model |
| `applications` | Application tracking read model |

---

## Tech rules

- TypeScript everywhere. No `any` on public boundaries.
- Prefer simple solutions over abstractions.
- Keep web and API fully separated — no shared runtime code, only shared types.
- Shared DTOs and Zod schemas belong in `packages/contracts/src/index.ts`.
- Do not invent product fields or fake candidate experience.
- Any AI-generated text must be grounded in user-provided resume/profile facts.
- Zod validation on all POST/PATCH inputs (API side).
- Frontend: no heavy state library — plain `useState`/`useEffect` until a clear need arises.

---

## Coding rules

- Small functions with single responsibility
- Explicit return types on all exported functions
- No hidden magic, no decorators, no DI containers
- Comments only where the reasoning is non-obvious (not what, but why)
- Readability over cleverness
- Route handler pattern:
  ```ts
  app.post('/resource', async (req, reply) => {
    const body = Schema.parse(req.body);   // validate
    const id = crypto.randomUUID();
    db.insert(table).values({ id, ...body, createdAt: new Date().toISOString() }).run();
    return reply.code(201).send({ id, ...body });
  });
  ```
- Prefer `db.select().from(table).where(eq(table.id, id)).get()` pattern (Drizzle)

---

## AI integration pattern

```ts
// lib/claude.ts — always check for key before calling
if (!process.env.ANTHROPIC_API_KEY) {
  return reply.code(503).send({ error: 'AI generation not enabled. Set ANTHROPIC_API_KEY.' });
}
const client = new Anthropic();
const msg = await client.messages.create({ model: 'claude-opus-4-6', ... });
```

Prompts live in `lib/prompts.ts`. Each prompt function returns a string.
Prompts must reference only facts from the user's profile/resume, never invented content.

---

## Agent guide

Specialized agents live in `.claude/agents/`. Use them for focused tasks:

| Agent | When to use |
|-------|------------|
| `backend-implementer` | New API routes, DB schema changes, Zod schemas |
| `frontend-builder` | New pages, UI components, form flows |
| `ai-feature-engineer` | Claude API integration, prompt engineering |
| `job-ingestion-engineer` | URL scraper improvements, new job site parsers |
| `review-guardian` | Code review before committing |
| `mvp-architect` | Scope decisions, feature slicing |
| `db-schema-guard` | DB migrations, index strategy, schema changes |

---

## Workflow

- Before implementing, restate the task in one sentence.
- Propose the smallest working change first.
- Prefer editing existing files over creating new ones.
- After implementation, list changed files and short rationale.
- Run `pnpm -r typecheck` before calling a task done.

---

## Environment variables

```
ANTHROPIC_API_KEY=sk-ant-...      # enables AI generation (P3 features)
TELEGRAM_BOT_TOKEN=               # enables Telegram bot
PORT=3001                         # API port (optional, default 3001)
```
