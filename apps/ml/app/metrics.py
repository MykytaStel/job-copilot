from typing import Literal

from pydantic import BaseModel


RankingVariant = Literal[
    "deterministic",
    "deterministic_behavior",
    "deterministic_behavior_learned",
    "trained_reranker_prediction",
]


class RankingVariantMetrics(BaseModel):
    variant: RankingVariant
    top_n: int
    ordered_job_ids: list[str]
    top_k_positives: int
    average_label_score_top_n: float
    average_training_weight_top_n: float
    positive_hit_rate: float


class RerankerEvaluationSummary(BaseModel):
    profile_id: str
    label_policy_version: str
    signal_weight_policy_version: str
    split_method: str
    example_count: int
    train_example_count: int
    test_example_count: int
    positive_count: int
    top_n: int
    variants: list[RankingVariantMetrics]


def safe_ratio(numerator: int | float, denominator: int) -> float:
    if denominator == 0:
        return 0.0
    return float(numerator) / denominator
