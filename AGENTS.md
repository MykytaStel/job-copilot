# AGENTS.md

This repo uses AI assistants intentionally:

- Claude: planning, architecture review, risk analysis, code explanation, slice design, prompt writing.
- Codex: bounded implementation slices with explicit files, acceptance criteria, and verification commands.

Read order before any non-trivial work:

1. `CLAUDE.md`
2. `docs/00-master-plan.md`
3. `docs/05-roadmap/current-focus.md`
4. `docs/engineering-checklist.md`
5. Relevant domain/architecture docs for the slice
6. Relevant existing code and tests

## Shared working rule

Never batch unrelated work into one implementation pass.

Every task must have:

- a goal
- assumptions
- files likely touched
- acceptance criteria
- verification commands
- a short note about risks or tradeoffs

## Claude usage

Use Claude when the request is about:

- architecture review
- roadmap planning
- product/system tradeoffs
- deciding where a change belongs
- code explanation
- designing a Codex prompt
- reviewing a Codex diff
- reducing scope before implementation

Claude should produce plans, not broad unverified rewrites.

## Codex usage

Use Codex when the request is a bounded code change.

A Codex prompt must include:

- exact goal
- exact files or directories to inspect first
- files likely allowed to modify
- files not allowed to modify
- acceptance criteria
- commands to run
- expected final response format

Codex must not:

- rewrite unrelated code
- rename public contracts casually
- introduce speculative abstractions
- add paid LLM calls by default
- move domain truth into web or ML
- delete dead code outside the requested cleanup slice

## Project invariants

- Rust `engine-api` is canonical domain authority.
- `ingestion` fetches/normalizes/dedupes/lifecycles jobs and upserts canonical data.
- `ml` enriches and reranks, but its output is validated by Rust before becoming domain state.
- `web` displays and orchestrates user actions; it must not own domain logic.
- Contracts must remain explicit and testable.
- Ranking and feedback effects must stay explainable.
