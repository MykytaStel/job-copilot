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
    resolve_example_signal_bucket,
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


def rolling_temporal_windows(
    examples: list[OutcomeExample],
) -> list[tuple[list[OutcomeExample], list[OutcomeExample]]]:
    ordered = sorted(
        examples,
        key=lambda example: (
            example.label_observed_at or "",
            example.job_id,
        ),
    )
    if len(ordered) <= 1:
        return [(ordered[:0], ordered)]

    min_train = max(3, math.ceil(len(ordered) * 0.5))
    window_size = max(1, math.ceil(len(ordered) * 0.2))
    windows: list[tuple[list[OutcomeExample], list[OutcomeExample]]] = []
    train_end = min_train
    while train_end < len(ordered):
        test_end = min(len(ordered), train_end + window_size)
        windows.append((ordered[:train_end], ordered[train_end:test_end]))
        if test_end >= len(ordered):
            break
        train_end = test_end
    return windows or [temporal_train_test_split(ordered)]


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
        signal_bucket_metrics=aggregated_buckets,
    )


def ndcg_at_k(ordered: list[OutcomeExample], top_n: int) -> float:
    top_examples = ordered[:top_n]
    dcg = sum(
        ((2**example.label_score) - 1) / math.log2(index + 2)
        for index, example in enumerate(top_examples)
    )
    ideal = sorted(
        ordered,
        key=lambda example: (-example.label_score, example.job_id),
    )[:top_n]
    ideal_dcg = sum(
        ((2**example.label_score) - 1) / math.log2(index + 2)
        for index, example in enumerate(ideal)
    )
    if ideal_dcg <= 0:
        return 0.0
    return dcg / ideal_dcg


def mrr_at_k(ordered: list[OutcomeExample], top_n: int) -> float:
    for index, example in enumerate(ordered[:top_n], start=1):
        if example.label == "positive":
            return 1.0 / index
    return 0.0


def map_at_k(ordered: list[OutcomeExample], top_n: int) -> float:
    total_positives = sum(1 for e in ordered if e.label == "positive")
    if total_positives == 0:
        return 0.0
    hits = 0
    precision_sum = 0.0
    for index, example in enumerate(ordered[:top_n], start=1):
        if example.label == "positive":
            hits += 1
            precision_sum += hits / index
    return precision_sum / min(top_n, total_positives)


def precision_at_k(ordered: list[OutcomeExample], k: int) -> float:
    top = ordered[:k]
    if not top:
        return 0.0
    return sum(1 for e in top if e.label == "positive") / len(top)


def build_signal_bucket_metrics(
    ordered: list[OutcomeExample],
    top_examples: list[OutcomeExample],
) -> list[dict[str, int | float | str]]:
    overall_counts: dict[str, int] = {}
    top_counts: dict[str, int] = {}
    for example in ordered:
        bucket = resolve_example_signal_bucket(example)
        overall_counts[bucket] = overall_counts.get(bucket, 0) + 1
    for example in top_examples:
        bucket = resolve_example_signal_bucket(example)
        top_counts[bucket] = top_counts.get(bucket, 0) + 1
    return [
        {
            "bucket": bucket,
            "example_count": overall_counts[bucket],
            "top_n_count": top_counts.get(bucket, 0),
            "hit_rate": round(safe_ratio(top_counts.get(bucket, 0), overall_counts[bucket]), 6),
        }
        for bucket in sorted(overall_counts)
    ]
