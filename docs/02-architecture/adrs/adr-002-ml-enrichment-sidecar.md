# ADR-002: Python ML service is an enrichment sidecar

## Status

Accepted

## Context

Job Copilot needs ML and LLM capabilities: fit analysis, reranking, CV tailoring
suggestions, and enrichment of job or profile data. There are two broad options:

1. Embed ML/LLM logic inside the Rust engine-api.
2. Run a separate Python ML service as a sidecar that engine-api calls when needed.

Python has a significantly richer ecosystem for ML libraries (scikit-learn, transformers,
LLM SDKs) than Rust. However, putting ML output on the canonical write path creates
correctness risk: LLM responses are non-deterministic, providers can fail, and
output quality varies.

## Decision

The Python ML service (`apps/ml`) is an enrichment sidecar with these rules:

1. **ML output is non-canonical.** Fit scores, rerank order, enrichment text, and
   suggestions are inputs to engine-api's decision logic, not authoritative domain state.

2. **Engine-api constrains ML output.** Before using ML output in a response or
   storing anything derived from it, engine-api validates, bounds, or filters the
   result.

3. **Deterministic fallback is always available.** Engine-api has a complete
   deterministic ranking stack (role scoring, behavior scoring) that produces a usable
   response even when the ML sidecar is unavailable or returns an error.

4. **LLM/provider failures must not break core search.** The ML rerank call is
   optional. If it fails, engine-api falls back to the deterministic+behavior score
   without surfacing the error to the user as a feed-breaking failure.

5. **Paid LLM API is not the default path.** The runtime default is
   `TemplateEnrichmentProvider` (template-based, no external API calls). Ollama and
   OpenAI/Anthropic providers are available when explicitly configured.

6. **ML cannot shortcut engine-api to write canonical state.** ML output may suggest
   profile improvements or job annotations, but these must flow through engine-api
   endpoints, not be written directly to the database.

## Consequences

**Easier:**
- ML provider can be swapped or disabled without touching engine-api domain logic.
- LLM provider failures are isolated to enrichment paths.
- Python's ML library ecosystem is fully available without complicating the Rust build.
- Deterministic ranking provides a stable, explainable baseline independent of ML.

**Harder:**
- Two services must stay in sync on DTO shapes passed between engine-api and ML.
- The ML sidecar must be running for enrichment features to work; local dev requires
  Docker Compose or a running ML process.
- Testing the full enrichment flow requires integration testing across both services.

**Constraints created:**
- Do not train from raw ambiguous events; use engine-api's normalized outcome datasets.
- Do not make paid LLM the default path; template fallback must always work.
- Never log or expose ML internal token values.
- ML startup in production must fail fast if security configuration is absent.

## Current State

**Implemented:**
- `/api/v1/fit/analyze` — deterministic fit scoring.
- `/api/v1/rerank` — bounded job list reranking.
- Six enrichment endpoints using `TemplateEnrichmentProvider` as the runtime default.
- Logistic regression trained reranker architecture (`trained_reranker_v3`) with
  training script, temporal holdout evaluation, and artifact promotion.
- PII filtering, term normalization, compound term handling.
- OpenAI and Ollama providers implemented and selectable via configuration.

**Partial / gaps:**
- Runtime provider default (`template`) and Docker Compose default (`ollama`) still
  disagree. See [service-communication.md](../service-communication.md).
- Trained reranker is disabled by default (`TRAINED_RERANKER_ENABLED=false`). The
  labeled dataset is too small (~4 examples) to enable in production.
- ML production startup validation (`validate_startup_security()`) exists but is not
  yet verified as reliably enforced. See [security-model.md](../security-model.md).
- CV tailoring endpoint is planned but not yet implemented.

## Related Docs

- [security-model.md](../security-model.md)
- [service-communication.md](../service-communication.md)
- [data-flow.md](../data-flow.md)
- [reranker-architecture.md](../reranker-architecture.md)
- [ADR-001: Rust domain authority](adr-001-rust-domain-authority.md)
