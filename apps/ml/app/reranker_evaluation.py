import argparse
import json
from typing import Literal

from pydantic import BaseModel, Field


RankingVariant = Literal[
    "deterministic",
    "deterministic_behavior",
    "deterministic_behavior_learned",
]


class OutcomeRankingFeatures(BaseModel):
    deterministic_score: int = Field(ge=0, le=100)
    behavior_score: int = Field(ge=0, le=100)
    learned_reranker_score: int = Field(ge=0, le=100)


class OutcomeExample(BaseModel):
    job_id: str = Field(min_length=1)
    label: Literal["positive", "medium", "negative"]
    label_score: int = Field(ge=0, le=2)
    ranking: OutcomeRankingFeatures


class OutcomeDataset(BaseModel):
    profile_id: str = Field(min_length=1)
    label_policy_version: str = Field(min_length=1)
    examples: list[OutcomeExample] = Field(default_factory=list)


class RankingVariantMetrics(BaseModel):
    variant: RankingVariant
    top_n: int
    ordered_job_ids: list[str]
    top_k_positives: int
    average_label_score_top_n: float
    positive_hit_rate: float


class RerankerEvaluationSummary(BaseModel):
    profile_id: str
    label_policy_version: str
    example_count: int
    positive_count: int
    top_n: int
    variants: list[RankingVariantMetrics]


def evaluate_dataset(
    dataset: OutcomeDataset | dict,
    top_n: int = 10,
) -> RerankerEvaluationSummary:
    parsed = (
        dataset
        if isinstance(dataset, OutcomeDataset)
        else OutcomeDataset.model_validate(dataset)
    )
    safe_top_n = max(1, top_n)
    positive_count = sum(1 for example in parsed.examples if example.label == "positive")

    return RerankerEvaluationSummary(
        profile_id=parsed.profile_id,
        label_policy_version=parsed.label_policy_version,
        example_count=len(parsed.examples),
        positive_count=positive_count,
        top_n=safe_top_n,
        variants=[
            evaluate_variant(parsed.examples, "deterministic", safe_top_n, positive_count),
            evaluate_variant(parsed.examples, "deterministic_behavior", safe_top_n, positive_count),
            evaluate_variant(
                parsed.examples,
                "deterministic_behavior_learned",
                safe_top_n,
                positive_count,
            ),
        ],
    )


def evaluate_variant(
    examples: list[OutcomeExample],
    variant: RankingVariant,
    top_n: int,
    positive_count: int,
) -> RankingVariantMetrics:
    ordered = sorted(
        examples,
        key=lambda example: (-variant_score(example, variant), example.job_id),
    )
    top_examples = ordered[:top_n]
    top_positive_count = sum(1 for example in top_examples if example.label == "positive")
    label_score_sum = sum(example.label_score for example in top_examples)
    average_label_score = safe_ratio(label_score_sum, len(top_examples))

    return RankingVariantMetrics(
        variant=variant,
        top_n=top_n,
        ordered_job_ids=[example.job_id for example in ordered],
        top_k_positives=top_positive_count,
        average_label_score_top_n=round(average_label_score, 6),
        positive_hit_rate=round(safe_ratio(top_positive_count, positive_count), 6),
    )


def variant_score(example: OutcomeExample, variant: RankingVariant) -> int:
    if variant == "deterministic":
        return example.ranking.deterministic_score
    if variant == "deterministic_behavior":
        return example.ranking.behavior_score
    return example.ranking.learned_reranker_score


def safe_ratio(numerator: int | float, denominator: int) -> float:
    if denominator == 0:
        return 0.0
    return float(numerator) / denominator


def main() -> None:
    parser = argparse.ArgumentParser(description="Evaluate an exported reranker outcome dataset.")
    parser.add_argument("dataset_json", help="Path to a reranker dataset JSON export")
    parser.add_argument("--top-n", type=int, default=10)
    args = parser.parse_args()

    with open(args.dataset_json, "r", encoding="utf-8") as handle:
        payload = json.load(handle)

    summary = evaluate_dataset(payload, top_n=args.top_n)
    print(summary.model_dump_json(indent=2))


if __name__ == "__main__":
    main()
