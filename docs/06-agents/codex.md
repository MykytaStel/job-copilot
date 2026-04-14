# Codex Guide

Use Codex for bounded implementation slices only.

## Good Codex tasks
- introduce canonical role catalog
- add search profile domain/service/dto/route
- implement source filters end-to-end
- fix stale page refresh/query invalidation on route changes
- add job/company list models and endpoints
- add analytics endpoints and chart-ready DTOs

## Prompt pattern
1. state exact goal
2. list touched files/modules
3. define acceptance criteria
4. prohibit unrelated refactors
5. require tests

## Never ask Codex to
- redesign the whole product at once
- invent domain models without docs
- add LLM logic without structured contracts
