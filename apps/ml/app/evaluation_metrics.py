from __future__ import annotations

import math

from app.dataset import OutcomeExample
from app.metrics import safe_ratio
from app.reranker_signal_weights import resolve_example_signal_bucket


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
    total_positives = sum(1 for example in ordered if example.label == "positive")
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
    return sum(1 for example in top if example.label == "positive") / len(top)


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
