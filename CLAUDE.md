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

## Current state (March 2026)

**Built and working:**
- Candidate profile + resume versioning (PDF upload, text extraction)
- Job intake by URL (scraper) or pasted text + batch import (20 URLs)
- Fit scoring (Claude-powered, 0–100 + skill delta)
- Application tracking board (Kanban, drag-and-drop via @hello-pangea/dnd)
- Application detail view (notes, contacts, activities, tasks)
- Market pulse (skill demand analysis across all saved jobs)
- Job alerts (keyword filter + Telegram delivery)
- Telegram 2-way bot (/status, /jobs, /tasks) via Telegraf
- Cover letter CRUD (P3 scaffolding — AI call stubbed)
- Interview Q&A CRUD (P3 scaffolding — AI call stubbed)
- Offer tracking + side-by-side comparison table
- Full-text search (jobs + contacts, debounced sidebar widget)
- Backup / restore (JSON dump of all 14 tables)

**What's still stubbed:**
- `POST /cover-letters/:id/generate` → 503 until `ANTHROPIC_API_KEY` enabled
- `POST /interview-qa/generate` → 503

---

## Architecture

```
job-copilot-ua-starter/
├── apps/
│   ├── api/          ← Fastify 5 + Drizzle ORM + SQLite (better-sqlite3)
│   │   ├── src/
│   │   │   ├── index.ts        ← server entry, route registration
│   │   │   ├── db/
│   │   │   │   ├── schema.ts   ← Drizzle table definitions (14 tables)
│   │   │   │   └── index.ts    ← db init + CREATE TABLE IF NOT EXISTS
│   │   │   ├── lib/
│   │   │   │   ├── claude.ts   ← Anthropic SDK calls
│   │   │   │   ├── prompts.ts  ← prompt templates
│   │   │   │   ├── scraper.ts  ← job URL scraper (node-html-parser)
│   │   │   │   ├── bot.ts      ← Telegraf bot
│   │   │   │   └── telegram.ts ← Telegram webhook helpers
│   │   │   └── routes/         ← one file per domain (19 files)
│   └── web/          ← React 19 + Vite + React Router 7
│       └── src/
│           ├── App.tsx       ← route definitions
│           ├── Layout.tsx    ← app shell + sidebar + search
│           ├── api.ts        ← typed fetch client for all endpoints
│           └── pages/        ← 15 page components
├── packages/
│   └── shared/        ← shared TypeScript types + Zod schemas
│       └── src/index.ts
└── docs/              ← product decisions, roadmap
```

**Port conventions:** API = 3001, Web = 5173

---

## DB tables (SQLite + Drizzle)

| Table | Primary concern |
|-------|----------------|
| `profiles` | Candidate master profile (JSON skills) |
| `jobs` | Job postings (url, source, description, notes) |
| `resumes` | Resume versions (rawText, isActive) |
| `match_results` | Fit scores (score 0–100, JSON skill arrays) |
| `applications` | Tracking (status, appliedAt, dueDate) |
| `application_notes` | Free-text notes per application |
| `contacts` | People directory |
| `application_contacts` | Junction (applicationId, contactId, relationship) |
| `activities` | Event log (email/call/interview/follow_up) |
| `tasks` | Reminders (remindAt, done) |
| `alerts` | Keyword alerts (keywords JSON, telegramChatId) |
| `cover_letters` | Cover letters (tone: formal/casual/enthusiastic) |
| `interview_qa` | Q&A bank (category: behavioral/technical/situational/company) |
| `offers` | Job offers (salary, equity, benefits JSON) |

**DB patterns:**
- IDs: `crypto.randomUUID()` (text primary key)
- Timestamps: ISO string via `new Date().toISOString()`
- JSON columns: stored as TEXT, parsed in route handlers
- Migrations: handled inline in `db/index.ts` using `ALTER TABLE IF NOT EXISTS` pattern
- No FK enforcement (SQLite default); enforce relationships in route logic

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
