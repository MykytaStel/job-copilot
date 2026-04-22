# Coding Rules

## Rust
- prefer explicit domain models
- avoid free-form strings for canonical IDs
- keep services small and testable
- separate rules, matching, presentation, and orchestration when complexity grows

## TypeScript/React
- colocate page-specific query logic
- avoid stale route state
- invalidate or refetch on route param change where needed
- keep filters serializable in URL/query state when possible
- do not infer job lifecycle semantics from `postedAt` alone when engine presentation labels already exist

## Delivery
- keep the verification matrix app-local and enforce it in CI for the touched runtimes
- update runtime docs in the same slice when contracts or behavior change

## Python
- LLM outputs must be structured
- no free-form role invention
- return machine-readable evidence alongside prose
