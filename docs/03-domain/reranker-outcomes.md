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
  - saved
  - hidden
  - bad_fit
  - application_created
- ranking features:
  - `deterministic_score`
  - `behavior_score_delta`
  - `behavior_score`
  - `learned_reranker_score_delta`
  - `learned_reranker_score`
  - matched role / skill / keyword counts
  - deterministic fit, behavior, and learned-reranker reasons

## Label policy v1

`label_policy_version = outcome_label_v1`

Labels are assigned deterministically with this precedence:

1. `positive`, score `2`: an `application_created` event exists for the profile/job.
2. `negative`, score `0`: a `bad_fit` or `hidden` signal exists for the profile/job.
3. `medium`, score `1`: a `saved` signal exists for the profile/job.

Jobs without one of these signals are not exported as labeled examples.

The policy intentionally treats application creation as the strongest positive outcome. Negative feedback outranks save-only feedback when a job has both saved and hidden/bad-fit history, because explicit rejection is a stronger training signal than prior interest.

## Offline evaluation

`apps/ml/app/reranker_evaluation.py` evaluates an exported dataset without calling an LLM or training a model.

It compares three orderings:

- deterministic score
- deterministic + behavior score
- deterministic + behavior + learned reranker score

When passed a trained reranker v2 JSON artifact, the evaluator also compares:

- trained reranker prediction ordering

Metrics are defensive and deterministic:

- top-k positives
- average label score in the top N
- positive hit rate over all positive examples

Empty datasets and datasets with no positives return zero-valued metrics instead of failing.

## Trained reranker v2

`apps/ml/app/trained_reranker.py` trains the first inspectable trained reranker prototype from
one or more exported outcome datasets. It intentionally uses a dependency-light logistic
regression implemented in Python instead of a black-box model.

Feature inputs are explicit numeric or boolean fields already present in the export:

- deterministic score
- behavior score delta and behavior score
- learned reranker v1 score delta and score
- matched role, skill, and keyword counts
- source presence
- role family presence

The model does not use text embeddings and does not call an LLM.

The saved artifact is JSON and includes:

- `artifact_version = trained_reranker_v2`
- `model_type = logistic_regression`
- feature names and feature transforms
- feature weights and intercept
- training counts and loss
- `max_score_delta` for bounded optional live use

Example:

```bash
python -m app.trained_reranker \
  ./exports/profile-1-reranker-dataset.json \
  --output ./models/profile-1-trained-reranker-v2.json \
  --top-n 10
```

## Optional live integration

`engine-api` can load the trained reranker v2 artifact as a separate experimental additive layer
after deterministic ranking, explicit feedback scoring, behavior personalization, and learned
reranker v1.

The layer is disabled by default:

- `TRAINED_RERANKER_ENABLED=false`
- `TRAINED_RERANKER_MODEL_PATH=/path/to/trained-reranker-v2.json`

When enabled and loaded, the model applies only a bounded additive score delta and appends a
reason containing `Trained reranker v2`. It does not replace deterministic ranking and does not
remove learned reranker v1.
