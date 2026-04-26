# Codex Prompt Template — Review Existing Diff

Use this when Codex should review a diff before merge.

```md
You are reviewing the current diff in the Job Copilot monorepo.

First read:
- CLAUDE.md
- AGENTS.md
- codex/CODEX.md
- docs/engineering-checklist.md

## Review goal

Review the diff for correctness, scope control, contract safety, and verification gaps.

## Focus areas

- Does every changed line belong to the requested slice?
- Did the diff move domain truth out of Rust engine-api?
- Did the diff make ML/LLM output authoritative without Rust validation?
- Did the diff invent DTO fields, role IDs, env vars, endpoints, or DB columns?
- Are React Query invalidations correct after frontend mutations?
- Are migrations and DB behavior safe?
- Are tests updated for matching/ranking/feedback/lifecycle/ownership logic?
- Are docs updated when contracts or architecture changed?

## Output format

```md
## Merge readiness

Ready / Not ready

## Blockers

- ...

## Important non-blocking issues

- ...

## Scope concerns

- ...

## Missing verification

- ...

## Suggested minimal fixes

1. ...
2. ...
```

Do not rewrite the whole implementation unless asked. Prefer targeted patch suggestions.
```
