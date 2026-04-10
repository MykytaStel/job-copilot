# Coding Rules

## Main principle
Optimize for long-term clarity, not short-term cleverness.

## Architecture rules
- core domain logic belongs in Rust services
- web should not own domain logic
- ML service should not own business rules
- legacy code should not define future architecture

## File and module rules
- keep files small
- avoid giant utility files
- avoid "misc" or "helpers" dumping grounds
- group code by domain, not by framework only

## Naming rules
- use explicit names
- prefer domain names over technical names
- avoid abbreviations unless common and obvious

Good:
- `job_repository.rs`
- `application_status.rs`
- `ranking_input.rs`

Bad:
- `utils.rs`
- `common.ts`
- `dataManager.ts`

## API rules
- keep contracts explicit
- version public APIs carefully
- validate inputs early
- return stable response shapes

## Domain rules
- define domain types first
- define invariants early
- do not leak DB details into domain language
- keep ranking concepts independent from UI

## Rust rules
- prefer composition
- prefer small modules
- avoid unnecessary macros
- keep error handling explicit
- use enums for domain states where possible

## Python rules
- keep ML code isolated
- avoid placing business logic in prompts
- keep extraction and reranking outputs structured
- always prefer predictable JSON outputs

## Frontend rules
- keep business logic out of components where possible
- components should be easy to replace
- UI should consume contracts, not invent them

## Migration rules
- do not rewrite unrelated code
- preserve working user flows while migrating
- move one responsibility at a time
- document each structural change

## Test rules
- test domain logic first
- test ranking logic early
- test adapters at boundaries
- test user flows with simple E2E smoke coverage

## Documentation rules
When changing:
- domain concepts
- API contracts
- service responsibilities
- architecture

Update docs in the same change set.