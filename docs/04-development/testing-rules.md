# Testing Rules

## Goal
Protect the core engine first.

## Testing layers

### Unit tests
Use for:
- ranking formulas
- normalization helpers
- seniority mapping
- dedupe scoring
- domain validations

### Integration tests
Use for:
- API + DB
- ingestion + normalization
- ranking + search profile
- application status updates

### End-to-end tests
Use for:
- search
- open job
- save job
- update application status

## Priority order
1. core ranking logic
2. normalization and dedupe
3. API contracts
4. user workflows
5. UI polish

## Core rules
- every important domain rule must be testable without UI
- every ranking change should be easy to verify
- ingestion changes should not silently break parsing
- test names should explain behavior

## Smoke test minimum
At minimum, keep one test flow that verifies:
1. jobs can be listed
2. job details can be opened
3. a job can be saved
4. application status can be changed

## Regression policy
When fixing a bug:
- add or update a test when practical
- capture the failing behavior
- keep the fix small if possible