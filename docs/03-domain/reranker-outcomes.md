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

Metrics are defensive and deterministic:

- top-k positives
- average label score in the top N
- positive hit rate over all positive examples

Empty datasets and datasets with no positives return zero-valued metrics instead of failing.
