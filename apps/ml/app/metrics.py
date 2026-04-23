from typing import Literal

from pydantic import BaseModel, Field


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
    ndcg_at_top_n: float = 0.0
    mrr_at_top_n: float = 0.0
    signal_bucket_metrics: list[dict[str, int | float | str]] = Field(default_factory=list)


class RerankerEvaluationSummary(BaseModel):
    profile_id: str
    label_policy_version: str
    metrics_version: str = "reranker_eval_v2"
    signal_weight_policy_version: str
    split_method: str
    example_count: int
    train_example_count: int
    test_example_count: int
    positive_count: int
    top_n: int
    rolling_window_count: int = 0
    variants: list[RankingVariantMetrics]


def safe_ratio(numerator: int | float, denominator: int) -> float:
    if denominator == 0:
        return 0.0
    return float(numerator) / denominator
