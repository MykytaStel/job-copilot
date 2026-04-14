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

## Python
- LLM outputs must be structured
- no free-form role invention
- return machine-readable evidence alongside prose
