# Search Profile

Search profile bridges profile analysis and job search.

## Inputs
- analyzed profile
- explicit user preferences

## Preferences
- target_regions
- work_modes
- preferred_roles
- include_keywords
- exclude_keywords
- source_filters

## Output
- primary_role
- target_roles
- seniority
- target_regions
- work_modes
- allowed_sources
- search_terms
- exclude_terms

## Deterministic Search Run
Search profile is the input to backend-first search execution.

Current deterministic baseline:
- fetch active jobs from the existing feed/recent-jobs access path
- filter out jobs whose source is not in `allowed_sources` when that filter is set
- score jobs using explicit evidence from:
  - target role overlap
  - search term overlap
  - source match
  - work mode match when present
  - target region match when inferable from the job text
- return ranked jobs with a structured `JobFit` explanation

The search run stays deterministic and Rust-owned:
- no LLM calls
- no provider-specific logic in the scoring path
- no frontend-owned ranking truth
