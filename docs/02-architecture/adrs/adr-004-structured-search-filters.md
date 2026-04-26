# ADR-004: Search filters remain structured and explicit

## Status

Accepted

## Context

Job search relevance can be improved either by structured filters (source, region,
work mode, role, salary range, keywords) or by semantic/vector search (embedding-based
similarity matching). Semantic search can capture nuance but is a black box: it is hard
to explain why a job appeared or did not appear, hard to debug, and requires sufficient
labeled data to validate that it improves outcomes.

The candidate also needs predictable, operator-level control: "show me remote Python
jobs from Djinni only" should be exact, not approximate.

## Decision

Search filters remain structured and explicit as the primary filtering mechanism.

- **Explicit filter dimensions:** source, region, work mode, role ID, include keywords,
  exclude keywords, seniority, and salary range.
- **Filters are deterministic.** A filter match or exclusion is exact and auditable.
- **ML ranking augments, does not replace, filters.** Reranking and fit scoring operate
  on the filtered candidate set. They change the order and surface reasons; they do not
  override explicit exclusions.
- **Do not replace explicit filters with vague semantic search only.** Semantic ranking
  may be added as a scoring layer in the future, but it must not become the sole
  relevance mechanism.
- **Explainability is a hard requirement.** The system must be able to tell a user
  why a job appeared (which filters matched) and why a job ranked higher (which scoring
  signals contributed).

## Consequences

**Easier:**
- Users and operators can reason about why jobs appear.
- Debugging incorrect results has a clear audit path: inspect filters, then scores.
- The deterministic ranking layer can run without ML and still produce correct filter
  results.
- Preference persistence is straightforward: search profile stores explicit filter values.

**Harder:**
- Structured filters require the candidate to know what they want. They do not
  automatically surface unexpected but relevant jobs.
- Normalization matters: role IDs and region normalization must be correct for filters
  to work as expected. See [ADR-003](adr-003-role-taxonomy.md).
- As the job taxonomy grows, maintaining filter dimension coverage takes ongoing effort.

**Constraints created:**
- Do not introduce semantic embeddings as a primary search mechanism until sufficient
  labeled data validates the improvement. Embeddings are explicitly deferred.
- ML enrichment output (fit score, rerank) is an additive layer over the structured
  filter result, not a replacement.
- Search profile preferences (stored on the engine-api profile) must remain explicit
  typed fields, not freeform text queries.

## Current State

**Implemented:**
- Job feed supports source filter, lifecycle filter, and search profile matching.
- Deterministic ranking uses role ID, skills, work mode, region, seniority, and source
  signals.
- Search profile preferences (target roles, preferred sources, work mode, region)
  persist on the profile and hydrate into the web search profile builder.
- Include/exclude keyword filters exist in the profile search builder.
- Web filter UI surfaces source, lifecycle, and ranked mode controls.

**Partial / gaps:**
- Salary range filter is not yet exposed as a frontend filter control.
- Full keyword include/exclude filter surface in the job feed UI may be incomplete.
- Semantic embeddings are explicitly not implemented and not planned for the near term.

## Related Docs

- [current-state.md](../current-state.md)
- [data-flow.md](../data-flow.md)
- [ADR-003: Role taxonomy](adr-003-role-taxonomy.md)
- [ADR-002: ML enrichment sidecar](adr-002-ml-enrichment-sidecar.md)
