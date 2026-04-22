import math
from typing import Any

from app.dataset import OutcomeDataset, OutcomeExample
from app.metrics import (
    RankingVariant,
    RankingVariantMetrics,
    RerankerEvaluationSummary,
    safe_ratio,
)
from app.reranker_signal_weights import (
    DEFAULT_OUTCOME_SIGNAL_WEIGHTS,
    OutcomeSignalWeightConfig,
    signal_weight_config_from_payload,
    training_target_for_example,
)


def evaluate_dataset(
    dataset: OutcomeDataset | dict,
    top_n: int = 10,
    trained_model: Any | None = None,
    signal_weights: OutcomeSignalWeightConfig | None = None,
) -> RerankerEvaluationSummary:
    parsed = (
        dataset
        if isinstance(dataset, OutcomeDataset)
        else OutcomeDataset.model_validate(dataset)
    )
    safe_top_n = max(1, top_n)
    train_examples, test_examples = temporal_train_test_split(parsed.examples)
    positive_count = sum(1 for example in test_examples if example.label == "positive")
    effective_signal_weights = resolve_signal_weights(
        signal_weights=signal_weights,
        trained_model=trained_model,
    )

    variants = [
        evaluate_variant(
            test_examples,
            "deterministic",
            safe_top_n,
            positive_count,
            signal_weights=effective_signal_weights,
        ),
        evaluate_variant(
            test_examples,
            "deterministic_behavior",
            safe_top_n,
            positive_count,
            signal_weights=effective_signal_weights,
        ),
        evaluate_variant(
            test_examples,
            "deterministic_behavior_learned",
            safe_top_n,
            positive_count,
            signal_weights=effective_signal_weights,
        ),
    ]
    if trained_model is not None:
        variants.append(
            evaluate_variant(
                test_examples,
                "trained_reranker_prediction",
                safe_top_n,
                positive_count,
                trained_model=trained_model,
                signal_weights=effective_signal_weights,
            )
        )

    return RerankerEvaluationSummary(
        profile_id=parsed.profile_id,
        label_policy_version=parsed.label_policy_version,
        signal_weight_policy_version=effective_signal_weights.policy_version,
        split_method="temporal",
        example_count=len(parsed.examples),
        train_example_count=len(train_examples),
        test_example_count=len(test_examples),
        positive_count=positive_count,
        top_n=safe_top_n,
        variants=variants,
    )


def evaluate_variant(
    examples: list[OutcomeExample],
    variant: RankingVariant,
    top_n: int,
    positive_count: int,
    trained_model: Any | None = None,
    signal_weights: OutcomeSignalWeightConfig = DEFAULT_OUTCOME_SIGNAL_WEIGHTS,
) -> RankingVariantMetrics:
    ordered = sorted(
        examples,
        key=lambda example: (-variant_score(example, variant, trained_model), example.job_id),
    )
    top_examples = ordered[:top_n]
    top_positive_count = sum(1 for example in top_examples if example.label == "positive")
    label_score_sum = sum(example.label_score for example in top_examples)
    training_weight_sum = sum(
        training_target_for_example(example, signal_weights) for example in top_examples
    )
    average_label_score = safe_ratio(label_score_sum, len(top_examples))
    average_training_weight = safe_ratio(training_weight_sum, len(top_examples))

    return RankingVariantMetrics(
        variant=variant,
        top_n=top_n,
        ordered_job_ids=[example.job_id for example in ordered],
        top_k_positives=top_positive_count,
        average_label_score_top_n=round(average_label_score, 6),
        average_training_weight_top_n=round(average_training_weight, 6),
        positive_hit_rate=round(safe_ratio(top_positive_count, positive_count), 6),
    )


def variant_score(
    example: OutcomeExample,
    variant: RankingVariant,
    trained_model: Any | None = None,
) -> float:
    if variant == "deterministic":
        return example.ranking.deterministic_score
    if variant == "deterministic_behavior":
        return example.ranking.behavior_score
    if variant == "deterministic_behavior_learned":
        return example.ranking.learned_reranker_score
    if trained_model is None:
        raise ValueError("trained_model is required for trained_reranker_prediction")
    return trained_model.predict_probability(example)


def resolve_signal_weights(
    *,
    signal_weights: OutcomeSignalWeightConfig | None,
    trained_model: Any | None,
) -> OutcomeSignalWeightConfig:
    if signal_weights is not None:
        return signal_weights

    artifact = getattr(trained_model, "artifact", None)
    if artifact is None:
        return DEFAULT_OUTCOME_SIGNAL_WEIGHTS

    return signal_weight_config_from_payload(
        getattr(artifact, "signal_weights", None),
        policy_version=getattr(artifact, "signal_weight_policy_version", None),
    )


def temporal_train_test_split(
    examples: list[OutcomeExample],
) -> tuple[list[OutcomeExample], list[OutcomeExample]]:
    if not examples:
        return [], []

    ordered = sorted(
        examples,
        key=lambda example: (
            example.label_observed_at or "",
            example.job_id,
        ),
    )
    if len(ordered) == 1:
        return [], ordered

    test_count = max(1, math.ceil(len(ordered) * 0.2))
    if test_count >= len(ordered):
        test_count = 1

    return ordered[:-test_count], ordered[-test_count:]
