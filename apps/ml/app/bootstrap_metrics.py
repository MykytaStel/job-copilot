from __future__ import annotations

from app.metrics import RerankerEvaluationSummary
from app.reranker_evaluation import OutcomeDataset
from app.settings import (
    RERANKER_PROMOTION_MIN_ACCURACY,
    RERANKER_PROMOTION_MIN_EXAMPLES,
)


def class_mix_counts(dataset: OutcomeDataset) -> dict[str, int]:
    return {
        "positive": sum(1 for example in dataset.examples if example.label == "positive"),
        "medium": sum(1 for example in dataset.examples if example.label == "medium"),
        "negative": sum(1 for example in dataset.examples if example.label == "negative"),
    }


def metric_value(
    summary: RerankerEvaluationSummary,
    variant_name: str,
    field_name: str,
) -> float:
    variant = next(
        (variant for variant in summary.variants if variant.variant == variant_name),
        None,
    )

    if variant is None:
        return 0.0

    return float(getattr(variant, field_name, 0.0))


def promotion_accuracy(evaluation: RerankerEvaluationSummary) -> float:
    return metric_value(
        evaluation,
        "trained_reranker_prediction",
        "positive_hit_rate",
    )


def should_promote_model(example_count: int, accuracy: float) -> bool:
    return (
        example_count >= RERANKER_PROMOTION_MIN_EXAMPLES
        and accuracy >= RERANKER_PROMOTION_MIN_ACCURACY
    )
