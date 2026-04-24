from __future__ import annotations

import logging
from typing import TYPE_CHECKING

from app.reranker_signal_weights import (
    DEFAULT_OUTCOME_CONFIDENCE_WEIGHTS,
    DEFAULT_OUTCOME_SIGNAL_WEIGHTS,
    OutcomeConfidenceWeightConfig,
    OutcomeSignalWeightConfig,
    confidence_weight_for_example,
    training_target_for_example,
)
from app.reranker_evaluation import OutcomeDataset, OutcomeExample

from .artifact import DEFAULT_FEATURE_NAMES
from .features import extract_features
from .training import reference_now_timestamp, temporal_example_weight

if TYPE_CHECKING:
    pass

logger = logging.getLogger(__name__)

_MIN_EXAMPLES_FOR_LGBM = 50
_MIN_POSITIVE_FOR_LGBM = 5
_MIN_NEGATIVE_FOR_LGBM = 5


def lgbm_available() -> bool:
    try:
        import lightgbm  # noqa: F401
        import numpy  # noqa: F401
        return True
    except (ImportError, OSError):
        return False


def lgbm_candidate_available(datasets: list[OutcomeDataset]) -> bool:
    total = sum(len(d.examples) for d in datasets)
    positive = sum(
        1 for dataset in datasets for example in dataset.examples if example.label == "positive"
    )
    negative = sum(
        1 for dataset in datasets for example in dataset.examples if example.label == "negative"
    )
    if total < _MIN_EXAMPLES_FOR_LGBM:
        return False
    if positive < _MIN_POSITIVE_FOR_LGBM:
        return False
    if negative < _MIN_NEGATIVE_FOR_LGBM:
        return False
    return lgbm_available()


def distill_lgbm_labels(
    examples: list[OutcomeExample],
    feature_names: list[str] | None = None,
    signal_weights: OutcomeSignalWeightConfig = DEFAULT_OUTCOME_SIGNAL_WEIGHTS,
    confidence_weights: OutcomeConfidenceWeightConfig = DEFAULT_OUTCOME_CONFIDENCE_WEIGHTS,
    temporal_decay_lambda: float = 0.01,
    *,
    num_leaves: int = 31,
    num_boost_round: int = 100,
    learning_rate: float = 0.05,
) -> list[float]:
    """Train LightGBM and return per-example soft predictions for logistic regression distillation.

    Returns a list of float predictions in [0, 1] aligned with the input examples.
    Falls back to hard signal-bucket labels if LightGBM training fails.
    """
    import lightgbm as lgb
    import numpy as np

    names = list(feature_names or DEFAULT_FEATURE_NAMES)
    vectors = [[extract_features(e)[f] for f in names] for e in examples]
    hard_labels = [training_target_for_example(e, signal_weights) for e in examples]
    binary_labels = [1.0 if label >= 0.5 else 0.0 for label in hard_labels]

    now_ts = reference_now_timestamp(examples)
    sample_weights = [
        confidence_weight_for_example(e, confidence_weights)
        * temporal_example_weight(e, now_ts, temporal_decay_lambda)
        for e in examples
    ]

    X = np.array(vectors, dtype=np.float32)
    y = np.array(binary_labels, dtype=np.float32)
    w = np.array(sample_weights, dtype=np.float32)
    w = np.clip(w, 0.0, None)
    if w.sum() <= 0:
        w = np.ones(len(examples), dtype=np.float32)

    dataset = lgb.Dataset(X, label=y, weight=w, feature_name=names, free_raw_data=False)
    params = {
        "objective": "binary",
        "metric": "binary_logloss",
        "num_leaves": num_leaves,
        "learning_rate": learning_rate,
        "min_data_in_leaf": max(1, len(examples) // 20),
        "verbose": -1,
        "n_jobs": 1,
    }
    try:
        model = lgb.train(params, dataset, num_boost_round=num_boost_round, callbacks=[lgb.log_evaluation(period=-1)])
        soft_labels: list[float] = model.predict(X).tolist()
        logger.info(
            "lgbm distillation: trained on %d examples, avg soft label=%.4f",
            len(examples),
            sum(soft_labels) / len(soft_labels),
        )
        return soft_labels
    except Exception as exc:
        logger.warning("lgbm training failed, falling back to hard labels: %s", exc)
        return hard_labels
