# Job Copilot UA Starter

Starter monorepo for a personal AI-assisted job search tool focused on the Ukrainian market.

## Stack

- **Web:** React + Vite + TypeScript
- **API:** Fastify + TypeScript
- **Workspace:** pnpm workspaces
- **Shared types:** local workspace package
- **AI/docs support:** Claude Code project instructions, agents, skills, hooks

## Why this stack

- Fast frontend startup
- Simple backend without SSR complexity
- One language across web and API
- Easy to grow into Postgres, auth, queues, and LLM services

## Project structure

```text
job-copilot-ua-starter/
  apps/
    web/        # React UI
    api-legacy/ # Fastify legacy API
  packages/
    contracts/  # Shared TS contracts
  docs/         # Product and workflow docs
  .claude/      # Claude Code project config
```

## Prerequisites

- Node.js 22.x recommended
- pnpm 10+
- VS Code
- Git

## First run

```bash
pnpm install
pnpm dev
```

Then open:

- Web: http://localhost:5173
- API: http://localhost:3001/health

## Useful scripts

```bash
pnpm dev
pnpm build
pnpm lint
pnpm typecheck
```

## Next steps

1. Add Postgres and Prisma
2. Add auth
3. Add CV upload and parsing
4. Add job import by URL/text
5. Add fit scoring and tailored resume output
