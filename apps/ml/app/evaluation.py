from typing import Any

from app.dataset import OutcomeDataset, OutcomeExample
from app.evaluation_metrics import (
    build_signal_bucket_metrics,
    map_at_k,
    mrr_at_k,
    ndcg_at_k,
    precision_at_k,
)
from app.evaluation_splits import rolling_temporal_windows, temporal_train_test_split
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
    windows = rolling_temporal_windows(parsed.examples)
    positive_count = sum(1 for example in test_examples if example.label == "positive")
    effective_signal_weights = resolve_signal_weights(
        signal_weights=signal_weights,
        trained_model=trained_model,
    )

    variants = []
    variant_names: list[RankingVariant] = [
        "deterministic",
        "deterministic_behavior",
        "deterministic_behavior_learned",
    ]
    if trained_model is not None:
        variant_names.append("trained_reranker_prediction")

    for variant_name in variant_names:
        window_metrics = [
            evaluate_variant(
                window_examples,
                variant_name,
                safe_top_n,
                sum(1 for example in window_examples if example.label == "positive"),
                trained_model=trained_model,
                signal_weights=effective_signal_weights,
            )
            for _, window_examples in windows
            if window_examples
        ]
        variants.append(
            aggregate_variant_metrics(
                window_metrics or [
                    evaluate_variant(
                        test_examples,
                        variant_name,
                        safe_top_n,
                        positive_count,
                        trained_model=trained_model,
                        signal_weights=effective_signal_weights,
                    )
                ]
            )
        )

    return RerankerEvaluationSummary(
        profile_id=parsed.profile_id,
        label_policy_version=parsed.label_policy_version,
        signal_weight_policy_version=effective_signal_weights.policy_version,
        split_method="rolling_temporal" if len(windows) > 1 else "temporal",
        example_count=len(parsed.examples),
        train_example_count=len(train_examples),
        test_example_count=len(test_examples),
        positive_count=positive_count,
        top_n=safe_top_n,
        rolling_window_count=len(windows),
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
    ndcg = ndcg_at_k(ordered, top_n)
    mrr = mrr_at_k(ordered, top_n)
    map_k = map_at_k(ordered, top_n)
    prec_3 = precision_at_k(ordered, 3)
    bucket_totals = build_signal_bucket_metrics(ordered, top_examples)

    return RankingVariantMetrics(
        variant=variant,
        top_n=top_n,
        ordered_job_ids=[example.job_id for example in ordered],
        top_k_positives=top_positive_count,
        average_label_score_top_n=round(average_label_score, 6),
        average_training_weight_top_n=round(average_training_weight, 6),
        positive_hit_rate=round(safe_ratio(top_positive_count, positive_count), 6),
        ndcg_at_top_n=round(ndcg, 6),
        mrr_at_top_n=round(mrr, 6),
        map_at_top_n=round(map_k, 6),
        precision_at_3=round(prec_3, 6),
        signal_bucket_metrics=bucket_totals,
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


def aggregate_variant_metrics(metrics: list[RankingVariantMetrics]) -> RankingVariantMetrics:
    sample = metrics[-1]
    bucket_totals: dict[str, dict[str, float]] = {}
    for metric in metrics:
        for bucket_metric in metric.signal_bucket_metrics:
            bucket = str(bucket_metric["bucket"])
            aggregate = bucket_totals.setdefault(
                bucket,
                {"example_count": 0.0, "top_n_count": 0.0},
            )
            aggregate["example_count"] += float(bucket_metric["example_count"])
            aggregate["top_n_count"] += float(bucket_metric["top_n_count"])
    aggregated_buckets = [
        {
            "bucket": bucket,
            "example_count": int(values["example_count"]),
            "top_n_count": int(values["top_n_count"]),
            "hit_rate": round(
                safe_ratio(values["top_n_count"], int(values["example_count"])),
                6,
            ),
        }
        for bucket, values in sorted(bucket_totals.items())
    ]

    return RankingVariantMetrics(
        variant=sample.variant,
        top_n=sample.top_n,
        ordered_job_ids=sample.ordered_job_ids,
        top_k_positives=round(sum(metric.top_k_positives for metric in metrics) / len(metrics)),
        average_label_score_top_n=round(
            sum(metric.average_label_score_top_n for metric in metrics) / len(metrics),
            6,
        ),
        average_training_weight_top_n=round(
            sum(metric.average_training_weight_top_n for metric in metrics) / len(metrics),
            6,
        ),
        positive_hit_rate=round(
            sum(metric.positive_hit_rate for metric in metrics) / len(metrics),
            6,
        ),
        ndcg_at_top_n=round(sum(metric.ndcg_at_top_n for metric in metrics) / len(metrics), 6),
        mrr_at_top_n=round(sum(metric.mrr_at_top_n for metric in metrics) / len(metrics), 6),
        map_at_top_n=round(sum(metric.map_at_top_n for metric in metrics) / len(metrics), 6),
        precision_at_3=round(sum(metric.precision_at_3 for metric in metrics) / len(metrics), 6),
        signal_bucket_metrics=aggregated_buckets,  # type: ignore[arg-type]
    )
