# Reranker Architecture — 2026-04-26

This document describes the current reranker design, its operating modes, fallback
behavior, training data flow, limitations, and safety rules.

See [reranker-outcomes.md](../03-domain/reranker-outcomes.md) for the dataset export
contract and label policy. See [service-communication.md](service-communication.md)
for the ML internal token flow. See [current-state.md](current-state.md) for build status.

---

## Purpose

The reranker improves the order of jobs shown to a candidate by combining:

- a deterministic fit score based on the candidate profile;
- a behavior score that adjusts for observed user signals;
- an optional trained model score that refines ranking based on labeled outcomes.

The goal is to surface jobs that are genuinely relevant to this candidate, with
explainable reasons, while keeping the fallback path safe and predictable.

---

## Ranking layers

Ranking is applied in additive layers. Each layer can only be present if the previous
one is active. The deterministic layer is always active.

```
Deterministic score          (always on, Rust, engine-api)
  +
Behavior score delta         (always on when events exist, Rust, engine-api)
  +
ML rerank score delta        (optional, Python, ML sidecar)
  +
Trained reranker delta       (optional, disabled by default, Python/Rust, engine-api)
```

### Layer 1: Deterministic scoring

**Location:** `apps/engine-api/src/services/matching.rs`

Scores every job against the candidate profile using:

- Primary role match
- Profile skills
- Target roles
- Role candidates
- Profile keywords
- Search terms
- Bonuses and penalties: source, work mode, region, seniority, exclude terms, role mismatch

No external API calls. No ML dependencies. Always produces a score.

### Layer 2: Behavior scoring

**Location:** `apps/engine-api/src/services/behavior.rs`

Adjusts the deterministic score using aggregated user signals:

- Per-source boosts and penalties based on positive and negative feedback history
- Per-role-family adjustments from save, hide, and bad fit history
- Saturation to prevent single-signal dominance

This layer runs in Rust inside engine-api with no external calls.

### Layer 3: ML rerank (optional)

**Location:** `apps/ml/app/` — `/api/v1/rerank`

When the web UI enables ML ranking, engine-api sends a bounded candidate list to the
ML sidecar for scoring. The sidecar returns adjusted scores and reasons. Engine-api
applies these as an additive delta.

If the ML sidecar is unreachable or returns an error, engine-api falls back to the
deterministic + behavior score. The job feed must remain functional without this layer.

### Layer 4: Trained reranker (optional, disabled by default)

**Location:** `apps/ml/app/trained_reranker/`, `apps/ml/models/trained-reranker-v3.json`

An additive logistic regression model trained on exported labeled outcomes. When enabled,
it applies a bounded score delta and appends a `Trained reranker v3` reason label.

This layer does not replace deterministic ranking. It does not remove the behavior or
learned reranker layers.

**Disabled by default:**

```
TRAINED_RERANKER_ENABLED=false
RERANKER_RUNTIME_MODE=deterministic|learned|trained
TRAINED_RERANKER_MODEL_PATH=/path/to/trained-reranker-v3.json
```

---

## Fallback behavior

The ranking stack degrades gracefully from right to left:

| Requested mode | Fallback if unavailable |
|----------------|------------------------|
| `trained` | falls back to `learned` if available, else `deterministic` |
| `learned` | falls back to `deterministic` |
| `deterministic` | always available, no fallback needed |

Every search response includes:

- `reranker_mode_requested` — what mode was requested
- `reranker_mode_active` — what mode actually ran
- `reranker_fallback_reason` — populated when a fallback occurred

This makes ranking behavior observable and debuggable without inspecting logs.

---

## Training data flow

```
User actions in engine-api
  │  (save, hide, apply, bad fit, view, dismiss)
  ▼
user_events table + feedback table  (PostgreSQL)
  │
  ▼
GET /api/v1/profiles/:id/reranker-dataset
  │  outcome_label_v3 normalization
  │  deterministic label precedence (applied > dismissed > saved > viewed)
  ▼
Exported JSON file  (per profile)
  │
  ▼
apps/ml/app/trained_reranker/  (offline training script)
  │  temporal holdout split (last 20% as test set)
  │  logistic regression fit on training split
  │  evaluation metrics on test set
  │  retrain on full dataset
  ▼
apps/ml/models/trained-reranker-v3.json  (promoted artifact)
  │
  ▼
engine-api  (optional live layer, TRAINED_RERANKER_ENABLED=true)
```

### Label policy (v3)

Labels are assigned with this precedence:

1. `positive` (score 2) — at least one application event for this profile/job
2. `negative` (score 0) — current state is dismissed (hidden or bad fit active)
3. `medium` (score 1) — current state is saved
4. `medium` (score 1) — job was viewed with a `job_opened` event, no stronger signal

Jobs without one of these signals are not exported.

### Automation

```bash
PROFILE_ID=<profile-id> pnpm train:reranker:v3
```

This exports the dataset, validates label counts, trains on the temporal training split,
evaluates on the test split, retrains on the full dataset, and promotes the artifact.

---

## Current limitations

- **Small training dataset.** The system ships with ~4 labeled examples per early
  profile. Weights trained on this data are near zero and provide no meaningful signal.

- **Retrain threshold not yet reached.** The retrain trigger is set at 30+ labelable
  outcome examples per profile. In early use this has not been reached.

- **No production promotion threshold.** There is no defined minimum evaluation metric
  required before enabling the trained reranker in production. Promotion is currently
  a manual operator decision.

- **Trained reranker disabled by default.** `TRAINED_RERANKER_ENABLED=false` is the
  safe default. Do not enable it in production until a sufficient labeled dataset
  exists and the artifact has passed offline evaluation.

- **Behavior signals need validation before strong ranking influence.** User event
  volumes are low in early use. Behavior boosts are capped to prevent premature
  saturation.

---

## Safety rules

1. **engine-api is the source of truth.** Deterministic scoring lives in Rust.
   ML output is an enrichment input, not canonical domain state.

2. **Deterministic ranking is always the safe baseline.** No ML failure must break
   the job feed. See [service-communication.md](service-communication.md) for the
   failure behavior contract.

3. **Trained reranker applies only a bounded additive delta.** It does not replace
   or remove earlier ranking layers.

4. **Ranking must remain explainable.** Every score includes reasons from each active
   layer: `fit_reasons`, `behavior_reasons`, and (when active) `learned_reasons`.
   The trained reranker appends its own reason label.

5. **ML output is never canonical without Rust-side validation.** Reranked scores
   are advisory. They influence display order but do not alter feedback records,
   application state, or lifecycle state.

6. **Token values are never logged.** The ML internal token flow is described in
   [service-communication.md](service-communication.md).

---

## What is not implemented

- Automatic production promotion based on evaluation thresholds.
- Semantic embeddings in ranking features (deferred until labeled data is sufficient).
- Online/streaming reranker updates (all training is offline batch).
- A defined production readiness threshold for the trained reranker.
