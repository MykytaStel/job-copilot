import argparse
import json

from app.dataset import OutcomeDataset, OutcomeExample, OutcomeRankingFeatures, OutcomeSignals
from app.evaluation import (
    evaluate_dataset,
    evaluate_variant,
    temporal_train_test_split,
    resolve_signal_weights,
    variant_score,
)
from app.metrics import (
    RankingVariant,
    RankingVariantMetrics,
    RerankerEvaluationSummary,
    safe_ratio,
)
from app.reranker_signal_weights import (
    DEFAULT_OUTCOME_SIGNAL_WEIGHTS,
    OutcomeSignalWeightConfig,
)

__all__ = [
    "DEFAULT_OUTCOME_SIGNAL_WEIGHTS",
    "OutcomeDataset",
    "OutcomeExample",
    "OutcomeRankingFeatures",
    "OutcomeSignalWeightConfig",
    "OutcomeSignals",
    "RankingVariant",
    "RankingVariantMetrics",
    "RerankerEvaluationSummary",
    "evaluate_dataset",
    "evaluate_variant",
    "main",
    "temporal_train_test_split",
    "resolve_signal_weights",
    "safe_ratio",
    "variant_score",
]


def main() -> None:
    parser = argparse.ArgumentParser(description="Evaluate an exported reranker outcome dataset.")
    parser.add_argument("dataset_json", help="Path to a reranker dataset JSON export")
    parser.add_argument("--top-n", type=int, default=10)
    parser.add_argument(
        "--trained-model",
        help="Optional trained reranker JSON artifact to include as a fourth ordering",
    )
    args = parser.parse_args()

    with open(args.dataset_json, "r", encoding="utf-8") as handle:
        payload = json.load(handle)

    trained_model = None
    if args.trained_model:
        from app.trained_reranker import TrainedRerankerModel

        trained_model = TrainedRerankerModel.load(args.trained_model)

    summary = evaluate_dataset(payload, top_n=args.top_n, trained_model=trained_model)
    print(summary.model_dump_json(indent=2))


if __name__ == "__main__":
    main()
