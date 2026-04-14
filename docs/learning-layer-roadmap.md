# Learning / LLM Roadmap for Job Copilot

## Current state

The project already has a strong deterministic core:

- profile analysis
- search-profile build
- deterministic ranked search
- source-aware presentation
- feedback persistence and feedback-aware ranking
- analytics summary + LLM context
- additive ML enrichments:
  - profile insights
  - job-fit explanation
  - application coaching
  - cover letter draft

That means the next real step is **not** "make the LLM magical".
The next step is to make the system **learn from user behavior and market data** in a controlled way.

---

## Product principle

Use this rule everywhere:

- **Rust / engine-api** = facts, canonical IDs, validation, deterministic ranking, event capture
- **Python / ml** = enrichment, summaries, coaching, interpretation, suggestions
- **Learning layer** = structured memory + aggregates + reward signals + optional learned reranking

The LLM must not become the source of truth.
It should become the **best consumer of the truth**.

---

# Phase 1 — Event Logging + Memory Foundations

## Goal
Capture user behavior in a reusable form so the system can learn from it later.

## What to add
Create a `profile_events` pipeline.

### Minimum event types
- job_impression
- job_opened
- source_opened
- search_run
- search_profile_built
- saved
- unsaved
- hidden
- unhidden
- bad_fit
- unbad_fit
- company_whitelisted
- company_blacklisted
- apply_created
- interview_started
- offer_received
- fit_explanation_requested
- application_coach_requested
- cover_letter_requested
- interview_prep_requested

### Minimum fields per event
- event_id
- profile_id
- job_id (nullable when not relevant)
- source_id (nullable)
- company_name_normalized (nullable)
- event_type
- event_at
- search_profile_snapshot (json)
- deterministic_fit_snapshot (json, nullable)
- metadata (json)

## Why it matters
Without event logs the system cannot learn.
It can only react.

## Deliverables
- DB migration for events table
- domain model
- repository + service
- write points in:
  - feedback actions
  - search run
  - profile page LLM actions
  - job open / source open
  - application creation

---

# Phase 2 — Aggregates + Analytics v2

## Goal
Turn raw events into signals the engine and LLM can actually use.

## Aggregates to build

### Per profile
- saves_by_source
- hides_by_source
- bad_fit_by_role_family
- apply_rate_by_source
- save_to_apply_rate
- top_saved_role_families
- top_hidden_role_families
- most_rejected_company_patterns
- top_positive_skills
- top_negative_patterns

### Market / system level
- jobs_by_source_over_time
- fit_score_vs_apply_rate
- source effectiveness by role family
- common gaps across searched jobs
- top market skills by role/source
- company response patterns if later available

## Deliverables
- aggregate queries or materialized snapshots
- analytics summary endpoint v2
- profile behavior summary endpoint
- chart-friendly DTOs

---

# Phase 3 — Personalized Deterministic Learning Layer

## Goal
Use history to improve ranking before introducing true model training.

## Approach
Add a **behavior-aware adjustment layer** on top of deterministic ranking.

### Examples
- repeated saves on backend/platform jobs => positive boost for similar jobs
- repeated hide/bad_fit on sales/support jobs => negative penalty for similar jobs
- whitelist company => stronger boost than generic source preference
- blacklist company => exclusion or near-exclusion
- better source performance for this profile => small source bonus
- repeated positive interaction with certain skill clusters => skill cluster boost

## Implementation style
Start simple:
- weighted heuristics
- profile-specific preference vectors
- no neural training yet
- keep it inspectable and debuggable

## Deliverables
- `personalization_features` builder
- `learning_adjustments` service
- explainable ranking deltas
- tests proving ranking changes from behavior

---

# Phase 4 — Learned Reranker v1

## Goal
Move from hand-tuned behavior boosts to a lightweight learned model.

## Good first model
Use something simple and stable:
- logistic regression
- gradient boosted trees
- linear model with calibrated weights

## Input features
- deterministic score
- role overlap
- skill overlap
- keyword overlap
- source
- work mode
- region
- company whitelist/blacklist
- prior save/hide/bad_fit patterns
- role-family preference
- source preference
- repeated interaction features

## Labels
Start with:
- save
- apply
- interview
- offer (later, if enough data)

## Output
- probability_of_save
- probability_of_apply
- probability_of_interview

Use this as an additive rerank signal, not as a replacement for canonical ranking.

## Deliverables
- offline training script in `apps/ml`
- evaluation notebook/script
- model artifact format
- safe inference path
- feature parity checks

---

# Phase 5 — LLM Analyst / Strategy Layer

## Goal
Use the learned and aggregated signals to make the LLM genuinely useful.

## What the LLM should do
- explain preference drift
- explain search underperformance
- recommend filter changes
- summarize weekly trends
- identify promising source/role/company patterns
- explain why certain jobs repeatedly become bad fits
- suggest narrow expansions:
  - adjacent roles
  - adjacent skills
  - underused sources

## Example outputs
- weekly search report
- "what changed this week"
- "best sources for you lately"
- "top repeated mismatch patterns"
- "skills that correlate with saved jobs"

## Deliverables
- `weekly-insights`
- `strategy-revision`
- `search-drift-analysis`
- `learning-summary`

---

# Phase 6 — True Model Training / Fine-Tuning (Optional, Later)

## Goal
Only after enough high-quality data exists.

## Use cases
- improve application coaching style
- improve cover-letter drafting groundedness
- improve interview prep relevance
- better summarize deterministic evidence

## Not for
- replacing canonical IDs
- replacing deterministic ranking
- inventing truth

## Requirements before attempting
- enough event volume
- cleaned dataset
- prompt/output evaluations
- offline benchmark suite
- hallucination safety checks

---

# Recommended build order

## First 6 implementation slices
1. Event logging v1
2. Aggregates / analytics v2
3. Personalization features + behavior-aware adjustments
4. Learned reranker offline pipeline
5. Weekly LLM analyst / strategy summaries
6. Optional fine-tuning experiments

---

# Data model guidance

## New backend domains
- `domain/event`
- `domain/learning`
- `domain/personalization`

## New tables
- `profile_events`
- later, if needed:
  - `profile_learning_snapshots`
  - `profile_preference_vectors`
  - `training_examples`

## New ML responsibilities
- aggregate generation
- offline training
- model scoring
- strategy summaries

---

# Safety / trust rules

Never let the LLM:
- invent canonical role IDs
- invent source IDs
- override deterministic ranking directly
- invent work history or achievements
- become the write authority

Always let the LLM:
- summarize
- explain
- recommend
- coach
- help interpret learning signals

---

# What success looks like

The system should become able to say:

- "You save backend/platform jobs from Djinni most often."
- "You frequently hide jobs with sales-heavy wording."
- "Your save-to-apply conversion is stronger on Work.ua than Robota.ua."
- "React Native adjacent frontend roles look promising, but pure product roles underperform."
- "Your last 20 saved jobs repeatedly mention Rust, Postgres, platform, and API ownership."

That is the point where the LLM is no longer just generating text.
It is acting as a **learning assistant on top of your real system memory**.
