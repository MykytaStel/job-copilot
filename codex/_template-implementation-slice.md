# Codex Prompt Template — Implementation Slice

Use this template when asking Codex to implement a specific feature or fix.

```md
You are working in the Job Copilot monorepo.

First read:
- CLAUDE.md
- AGENTS.md
- codex/CODEX.md
- docs/00-master-plan.md
- docs/05-roadmap/current-focus.md
- docs/engineering-checklist.md

## Goal

[Describe the exact behavior we want.]

## Scope

Implement only this slice. Do not batch unrelated cleanup, refactors, UI redesign, or architecture changes.

## Inspect first

- [file or directory]
- [file or directory]
- [related tests]

## Likely files to modify

- [file]
- [file]

## Files not allowed to modify unless absolutely necessary

- [file]
- [directory]

## Rules

- Keep domain truth in Rust engine-api.
- Keep ML as enrichment only.
- Do not invent DTO fields/endpoints/env vars/DB columns.
- Preserve existing public contracts unless this task explicitly changes them.
- Match existing style.
- Add or update tests for new behavior.
- Touch only files required for this slice.

## Acceptance criteria

- [ ] [observable behavior]
- [ ] [test coverage]
- [ ] [no unrelated diff]
- [ ] [docs updated if contract/architecture changed]

## Verification commands

Run the relevant commands and report results:

```bash
pnpm typecheck
pnpm build
pnpm --dir apps/web test
pnpm guard:web-api-imports
cargo test --manifest-path apps/engine-api/Cargo.toml
cargo check --manifest-path apps/engine-api/Cargo.toml
cargo test --manifest-path apps/ingestion/Cargo.toml
cargo check --manifest-path apps/ingestion/Cargo.toml
cd apps/ml && python -m pytest
```

Only run commands relevant to this slice. Say explicitly if a command was skipped and why.

## Final response format

```md
## What changed

## Why

## Files changed

## Verification

## Notes / follow-ups
```
```
