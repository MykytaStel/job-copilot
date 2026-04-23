# Reranker Outcome Dataset

Profile-scoped outcome exports provide the first explicit training and evaluation foundation for learned ranking work.

## Purpose

The dataset is read-only and offline-oriented:

- it does not replace live ranking
- it does not train or load a model
- it keeps labels and features inspectable
- it is scoped to one profile at a time

## API

`GET /api/v1/profiles/:id/reranker-dataset`

The endpoint requires persisted profile analysis because deterministic scoring must use canonical role IDs. If the profile has not been analyzed yet, it returns `profile_analysis_required`.

## Example shape

Each example contains:

- `job_id`, title, company, source, inferred `role_family`
- `label`, `label_score`, `label_reasons`
- outcome signals from profile events and current feedback:
  - `viewed`, `saved`, `applied`, `dismissed`, `hidden`, `bad_fit`
  - `explicit_feedback`, `explicit_saved`, `explicit_hidden`, `explicit_bad_fit`
  - `viewed_event_count`, `saved_event_count`, `applied_event_count`, `dismissed_event_count`
- ranking features:
  - `deterministic_score`
  - `behavior_score_delta`
  - `behavior_score`
  - `learned_reranker_score_delta`
  - `learned_reranker_score`
  - matched role / skill / keyword counts
  - deterministic fit, behavior, and learned-reranker reasons

Small v3 example:

```json
{
  "profile_id": "profile-1",
  "label_policy_version": "outcome_label_v3",
  "examples": [
    {
      "job_id": "job-123",
      "label_observed_at": "2026-04-20T00:00:00Z",
      "title": "Senior Backend Engineer",
      "company_name": "NovaLedger",
      "source": "djinni",
      "role_family": "engineering",
      "label": "positive",
      "label_score": 2,
      "label_reasons": ["applied"],
      "signals": {
        "viewed": true,
        "saved": true,
        "hidden": false,
        "bad_fit": false,
        "applied": true,
        "dismissed": false,
        "outcome": "offer_received",
        "reached_interview": true,
        "received_offer": true,
        "was_rejected": false,
        "was_ghosted": false,
        "explicit_feedback": true,
        "explicit_saved": true,
        "explicit_hidden": false,
        "explicit_bad_fit": false,
        "viewed_event_count": 1,
        "saved_event_count": 1,
        "applied_event_count": 1,
        "dismissed_event_count": 0,
        "has_salary_rejection": false,
        "has_remote_rejection": false,
        "has_tech_rejection": false,
        "interest_rating": 2,
        "work_mode_deal_breaker": false,
        "scrolled_to_bottom": true,
        "returned_count": 1,
        "time_to_apply_days": 2,
        "legitimacy_suspicious": false
      },
      "ranking": {
        "deterministic_score": 78,
        "behavior_score_delta": 4,
        "behavior_score": 82,
        "learned_reranker_score_delta": 3,
        "learned_reranker_score": 85,
        "matched_role_count": 1,
        "matched_skill_count": 5,
        "matched_keyword_count": 4,
        "fit_reasons": ["Role match"],
        "behavior_reasons": ["Saved similar jobs"],
        "learned_reasons": ["Learned reranker boosted this source"]
      }
    }
  ]
}
```

## Label policy v3

`label_policy_version = outcome_label_v3`

Labels are assigned deterministically with this precedence:

1. `positive`, score `2`: at least one `application_created` event exists for the profile/job.
2. `negative`, score `0`: the normalized current state is dismissed because `hidden` or `bad_fit` is active.
3. `medium`, score `1`: the normalized current state is saved.
4. `medium`, score `1`: the job was viewed with `job_opened`, but no stronger signal exists.

Jobs without one of these normalized signals are not exported as labeled examples.

Normalization rules:

- `viewed` means `job_opened`. `job_impression` is not treated as a view.
- `saved`, `hidden`, and `bad_fit` prefer the current mutable feedback row when present.
- If no feedback row is active for a flag, the export reconstructs the effective event state from ordered event history:
  - `job_saved` / `job_unsaved`
  - `job_hidden` / `job_unhidden`
  - `job_bad_fit` / `job_bad_fit_removed`
- `dismissed = hidden || bad_fit`
- `explicit_feedback = saved || hidden || bad_fit` from the feedback table

The policy intentionally treats application creation as the strongest positive outcome. Dismissal outranks save-only or view-only history when a job has conflicting signals, because explicit rejection is a stronger reranking signal than prior interest.

The export is deterministic:

- examples are sorted by `job_id`
- event normalization is processed in ascending `(created_at, id)` order per job
- label reasons use a fixed precedence order

`label_observed_at` is exported separately from job creation time. It represents the strongest
known label evidence timestamp with this precedence:

1. application `outcome_date`, else application `updated_at`, for application-based labels
2. latest relevant user event timestamp for the profile/job
3. feedback `updated_at`

An export with `"examples": []` is valid when a profile has no labelable job outcomes yet.
It can be evaluated defensively, but it cannot train `trained_reranker_v3`.

Application records are not global training labels. `POST /api/v1/applications` emits the
`application_created` outcome only when the request includes `profile_id`, preserving the
profile-scoped dataset contract.

## Offline evaluation

`apps/ml/app/reranker_evaluation.py` evaluates an exported dataset without calling an LLM or training a model.

It compares three orderings:

- deterministic score
- deterministic + behavior score
- deterministic + behavior + learned reranker score

When passed a trained reranker v3 JSON artifact, the evaluator also compares:

- trained reranker prediction ordering

Metrics are defensive and deterministic:

- top-k positives
- average label score in the top N
- average training weight in the top N
- positive hit rate over all positive examples

Evaluation now uses a temporal holdout split:

- examples are sorted by `label_observed_at`
- the last 20% become the held-out test set
- reported hit-rate metrics are computed on that test set only

Empty datasets and datasets with no positives return zero-valued metrics instead of failing.

## Trained reranker v3

`apps/ml/app/trained_reranker/` trains the first inspectable trained reranker prototype from
one or more exported outcome datasets. It intentionally uses a dependency-light logistic
regression implemented in Python instead of a black-box model.

Feature inputs are explicit numeric or boolean fields already present in the export:

- deterministic score
- behavior score delta and behavior score
- learned reranker v1 score delta and score
- matched role, skill, and keyword counts
- source presence
- role family presence
- quick apply / delayed apply derived from `time_to_apply_days`

The model does not use text embeddings and does not call an LLM.
Offline bootstrap also benchmarks a linear BPR candidate on the same temporal holdout when enough
positive/negative pairs exist, but the persisted production artifact remains logistic by default.

Default ML-side outcome weights stay conservative and inspectable:

- `applied = 1.0`
- `saved_only = 0.6`
- `viewed_only = 0.4`
- `dismissed = 0.0`
- `medium_default = 0.5` only as a fallback for legacy exports that do not include
  enough signal detail to distinguish save-only from view-only examples

`apps/ml/app/reranker_signal_weights.py` is the single config/helper entry point for this
mapping, and trained artifacts record the policy version plus the exact weights used.

The saved artifact is JSON and includes:

- `artifact_version = trained_reranker_v3`
- `model_type = logistic_regression`
- signal weight policy version and exact signal weights
- confidence weight policy version and exact confidence weights
- `temporal_decay_lambda`
- feature names and feature transforms
- feature weights and intercept
- `feature_importances`
- training counts and loss
- `max_score_delta` for bounded optional live use

Example:

```bash
python -m app.trained_reranker \
  ./exports/profile-1-reranker-dataset.json \
  --output ./models/profile-1-trained-reranker-v3.json \
  --top-n 10
```

Repo automation:

```bash
PROFILE_ID=<profile-id> pnpm train:reranker:v3
```

This exports the profile dataset, validates label counts, trains a candidate artifact on the
temporal training split, evaluates it on the held-out temporal test split, then retrains on the
full dataset before promoting `apps/ml/models/trained-reranker-v3.json`. For Docker:

```bash
PROFILE_ID=<profile-id> pnpm train:reranker:v3:docker
```

The Docker command restarts `engine-api` with `TRAINED_RERANKER_ENABLED=true` and the promoted
artifact mounted at `/app/models/trained-reranker-v3.json`.

## Operational loop

`engine-api` now keeps profile-scoped reranker learning state and history:

- `ml_examples_since_retrain`
- `ml_last_retrained_at`
- `ml_last_artifact_version`
- `ml_last_training_status`

New labelable outcomes increment the counter only once per `(profile_id, job_id)` via a dedicated
dedupe table. A background poller checks for profiles above the retrain threshold and calls
`POST /api/v1/reranker/bootstrap` asynchronously. Retrain outcomes are stored in
`profile_ml_metrics` and exposed via:

- `GET /api/v1/profiles/{id}/reranker/metrics`

## Optional live integration

`engine-api` can load the trained reranker v3 artifact as a separate experimental additive layer
after deterministic ranking, explicit feedback scoring, behavior personalization, and learned
reranker v1.

The layer is disabled by default:

- `RERANKER_RUNTIME_MODE=deterministic|learned|trained`
- `TRAINED_RERANKER_ENABLED=false`
- `TRAINED_RERANKER_MODEL_PATH=/path/to/trained-reranker-v3.json`

When enabled and loaded, the model applies only a bounded additive score delta and appends a
reason containing `Trained reranker v3`. It does not replace deterministic ranking and does not
remove learned reranker v1.

For rollout safety, live search now reports the requested mode, the active mode, and any fallback reason:

- `reranker_mode_requested`
- `reranker_mode_active`
- `reranker_fallback_reason`

If `trained` is requested but the artifact cannot be loaded, `engine-api` falls back to
`learned` when available and otherwise keeps deterministic ranking.
