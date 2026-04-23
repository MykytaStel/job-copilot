# ML Sidecar — Developer Reference

## Training pipeline

```
bootstrap_workflow.py
  → fetch labeled examples (engine-api /api/v1/profiles/{id}/reranker/dataset)
  → temporal_train_test_split() — 80% train / 20% test by label_observed_at
  → train_model([train_dataset])          # logistic regression, SGD
  → bpr_candidate_available()?            # requires ≥20 pos + 20 neg
      → train_bpr_model() + benchmark
  → compute drift score (KL-divergence vs previous model bucket distribution)
  → train_model([full_dataset])           # final model on all examples
  → model.save(models/trained-reranker-v3.json)
```

## Artifact versions

| Version | Model type | Notes |
|---------|-----------|-------|
| `trained_reranker_v3` | `logistic_regression` or `bpr` | current |

The artifact format stays v3 regardless of LightGBM use. LightGBM is used as a **teacher** (distillation): when ≥50 examples, LightGBM trains, generates soft predictions, and those replace the hard signal-bucket labels for the logistic regression. The final artifact is still logistic regression weights — Rust scoring is unchanged. `lgbm_distilled: true` in the artifact indicates this path was taken.

## Signal weights

`reranker_signal_weights.py` has two configs:
- `OutcomeSignalWeightConfig` — training label target (0–1, e.g. `received_offer=1.0`, `viewed_only=0.4`)
- `OutcomeConfidenceWeightConfig` — SGD sample weight multiplier (e.g. `received_offer=3.0`, `viewed_only=0.5`)

Both are stored in the artifact and can be overridden per retrain.

## Auto-retrain (engine-api side)

`engine-api/src/services/reranker_automation.rs` — `spawn_retrain_poller()`:
- Polls every `ML_RETRAIN_POLL_INTERVAL_SECONDS` (default 6h)
- Triggers when `ml_examples_since_retrain >= ML_RETRAIN_THRESHOLD` (default 15)
- Calls ML sidecar `POST /api/v1/reranker/bootstrap`
- Results persisted to `profile_ml_metrics` table

## Feature list (must stay in sync: `features.py` ↔ `trained_reranker.rs`)

24 features total. Key ones:
- `deterministic_score`, `behavior_score_delta`, `behavior_score` — base scoring layer
- `matched_role_count`, `matched_skill_count`, `matched_keyword_count` — search match quality
- `outcome_received_offer`, `outcome_reached_interview` — ground-truth outcomes
- `interest_rating_positive/negative`, `work_mode_deal_breaker` — user preference signals
- `quick_apply` (≤3d), `delayed_apply` (>14d) — derived from `time_to_apply_days`
- `legitimacy_suspicious` — spam/scam signal

## Adding a new signal

1. Add field to `OutcomeSignals` in `engine-api/src/services/outcome_dataset.rs`
2. Populate it in `resolve_outcome_signals()` in the same file
3. Add feature extraction in `apps/ml/app/trained_reranker/features.py` → `extract_features()`
4. Add feature name to `DEFAULT_FEATURE_NAMES` in `apps/ml/app/trained_reranker/artifact.py`
5. Add transform description to `FEATURE_TRANSFORMS` in the same file
6. Mirror the field in `TrainedRerankerFeatures` in `engine-api/src/services/trained_reranker.rs`
7. Wire it in `feature_value()` in the same file

## Data drift detection

After each retrain, `distribution_shift_score` (KL-divergence of signal bucket distribution vs previous model) is stored in the artifact and in `BootstrapResponse`. Score > 0.3 logs a warning — indicates preference shift.

## Prompt token budget

Weekly guidance system prompt: ~60 tokens. Context payload (JSON): ~800–2000 tokens depending on evidence count. Top model signals are injected into `llm_context.top_model_signals` (top 5 features by importance weight) — ~40 tokens.
