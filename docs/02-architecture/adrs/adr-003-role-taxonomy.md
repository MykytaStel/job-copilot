# ADR-003: Canonical role taxonomy uses stable role identifiers

## Status

Accepted

## Context

Job matching quality depends on comparing a candidate's target roles with job roles.
If roles are stored and compared as free-form strings, small variations ("backend developer"
vs "back-end engineer" vs "Backend Dev") fragment what is logically the same concept.
This makes ranking less reliable, explains poorly to users, and makes it difficult
to debug why a job was or was not surfaced.

The system needs a way to normalize roles from sources (job postings) and from profiles
(user-declared target roles) into a common representation for matching and scoring.

## Decision

Role matching relies on stable role identifiers, canonical aliases, and normalization
rather than raw free-form strings as the domain authority.

- **Stable role IDs** are the unit of matching and scoring. A role ID represents a
  logical job category (e.g., `backend-engineer`, `frontend-engineer`, `devops`).
- **Aliases and normalization** map source strings and profile strings to canonical
  role IDs. Ingestion normalizes scraped role signals into IDs. Profile analysis
  normalizes user-declared roles and extracted skills into IDs.
- **Scoring is role-ID aware.** The deterministic ranking layer compares
  profile role IDs against job role IDs and applies a boost when there is a match.
- **Explainability.** When the system reports fit reasons ("role match", "role mismatch"),
  it references the canonical role label, not the raw scraped string.
- **Do not use free-form role strings as domain authority.** Raw strings from job
  descriptions or CVs may be stored as metadata, but matching logic operates on
  normalized IDs.

## Consequences

**Easier:**
- Matching quality is more consistent across sources with different naming conventions.
- Fit explanations reference stable, human-readable labels.
- Debugging mismatches is straightforward: inspect the normalized role ID, not the
  raw text.
- Reranker training data uses stable IDs, which improves feature stability.

**Harder:**
- Normalization coverage must be maintained as new role types emerge.
- Aliases must be curated; uncovered roles will not match even if semantically close.
- Profile analysis must correctly extract and normalize roles from CVs.

**Constraints created:**
- Ingestion adapters must normalize roles into canonical IDs before writing job records.
- Profile analysis must produce normalized role IDs, not store raw CV role strings only.
- Scoring and ranking logic must operate on role IDs, not raw text comparison.
- Do not introduce semantic similarity as a substitute for ID-based matching until
  there is sufficient labeled data to validate it. See [ADR-004](adr-004-structured-search-filters.md).

## Current State

**Implemented:**
- Engine-api uses role IDs in deterministic ranking (role match score in job feed).
- Profile carries target role IDs derived from profile analysis.
- Seniority normalization is implemented in the engine-api ML client.
- Ingestion scraper adapters perform basic role normalization per source.

**Partial / gaps:**
- Alias coverage and normalization completeness are not audited. Unknown roles may
  silently fail to normalize.
- Role taxonomy is not yet published as a standalone reference document in `docs/`.
- Profile analysis role extraction accuracy is not formally benchmarked.

## Related Docs

- [current-state.md](../current-state.md)
- [data-flow.md](../data-flow.md)
- [ADR-001: Rust domain authority](adr-001-rust-domain-authority.md)
- [ADR-004: Structured search filters](adr-004-structured-search-filters.md)
