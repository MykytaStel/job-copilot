import argparse
import json
import logging
import sys

logger = logging.getLogger(__name__)

from app.reranker_evaluation import OutcomeDataset, evaluate_dataset, temporal_train_test_split

from .artifact import (
    ARTIFACT_VERSION,
    DEFAULT_FEATURE_NAMES,
    FEATURE_TRANSFORMS,
    MODEL_TYPE,
    TrainedRerankerArtifact,
    TrainingSummary,
    load_dataset,
)
from .bpr_model import bpr_candidate_available, train_bpr_model
from .features import clamp, extract_features, has_text
from .lgbm_model import distill_lgbm_labels, lgbm_available, lgbm_candidate_available
from .model import TrainedRerankerModel, sigmoid
from .training import average_log_loss, dot, smoothed_logit, train_model

__all__ = [
    "ARTIFACT_VERSION",
    "MODEL_TYPE",
    "DEFAULT_FEATURE_NAMES",
    "FEATURE_TRANSFORMS",
    "TrainingSummary",
    "TrainedRerankerArtifact",
    "TrainedRerankerModel",
    "extract_features",
    "train_model",
    "train_bpr_model",
    "bpr_candidate_available",
    "distill_lgbm_labels",
    "lgbm_available",
    "lgbm_candidate_available",
    "load_dataset",
    "clamp",
    "has_text",
    "sigmoid",
    "smoothed_logit",
    "dot",
    "average_log_loss",
    "main",
]


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Train an inspectable logistic-regression reranker from outcome datasets."
    )
    parser.add_argument("dataset_json", nargs="+", help="One or more reranker dataset JSON exports")
    parser.add_argument("--output", required=True, help="Path for the trained JSON model artifact")
    parser.add_argument("--top-n", type=int, default=10)
    parser.add_argument("--epochs", type=int, default=500)
    parser.add_argument("--learning-rate", type=float, default=0.08)
    parser.add_argument("--l2", type=float, default=0.01)
    parser.add_argument("--max-score-delta", type=int, default=8)
    args = parser.parse_args()

    try:
        datasets = [load_dataset(path) for path in args.dataset_json]
        merged_input = OutcomeDataset(
            profile_id="multiple" if len(datasets) > 1 else datasets[0].profile_id,
            label_policy_version=datasets[0].label_policy_version,
            examples=[example for dataset in datasets for example in dataset.examples],
        )
        train_examples, _ = temporal_train_test_split(merged_input.examples)
        candidate_datasets = (
            [
                OutcomeDataset(
                    profile_id=merged_input.profile_id,
                    label_policy_version=merged_input.label_policy_version,
                    examples=train_examples,
                )
            ]
            if train_examples
            else [merged_input]
        )
        candidate_model = train_model(
            candidate_datasets,  # type: ignore[arg-type]
            epochs=args.epochs,
            learning_rate=args.learning_rate,
            l2=args.l2,
            max_score_delta=args.max_score_delta,
        )
        model = train_model(
            datasets,  # type: ignore[arg-type]
            epochs=args.epochs,
            learning_rate=args.learning_rate,
            l2=args.l2,
            max_score_delta=args.max_score_delta,
        )
        model.save(args.output)
    except (OSError, ValueError, json.JSONDecodeError) as error:
        logger.error("training failed: %s", error)
        raise SystemExit(1) from error

    summary = evaluate_dataset(
        merged_input,
        top_n=args.top_n,
        trained_model=candidate_model,
    )
    benchmark = None
    if bpr_candidate_available(candidate_datasets):  # type: ignore[arg-type]
        bpr_model = train_bpr_model(
            candidate_datasets,  # type: ignore[arg-type]
            epochs=args.epochs,
            learning_rate=args.learning_rate,
            l2=args.l2,
            max_score_delta=args.max_score_delta,
        )
        bpr_summary = evaluate_dataset(
            merged_input,
            top_n=args.top_n,
            trained_model=bpr_model,
        )
        logistic_hit_rate = next(
            (
                variant.positive_hit_rate
                for variant in summary.variants
                if variant.variant == "trained_reranker_prediction"
            ),
            0.0,
        )
        bpr_hit_rate = next(
            (
                variant.positive_hit_rate
                for variant in bpr_summary.variants
                if variant.variant == "trained_reranker_prediction"
            ),
            0.0,
        )
        benchmark = {
            "baseline_model_type": "logistic_regression",
            "candidate_model_type": "bpr",
            "baseline_positive_hit_rate": logistic_hit_rate,
            "candidate_positive_hit_rate": bpr_hit_rate,
            "candidate_available": True,
            "winner": "bpr" if bpr_hit_rate > logistic_hit_rate else "logistic_regression",
        }
    print(
        json.dumps(
            {
                "model_artifact": args.output,
                "training": model.artifact.training.model_dump(),
                "evaluation": summary.model_dump(),
                "benchmark": benchmark,
            },
            indent=2,
        )
    )
