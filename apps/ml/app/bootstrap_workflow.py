import logging
from collections.abc import Awaitable, Callable
from pathlib import Path

from app.bootstrap_client import fetch_labeled_examples as fetch_labeled_examples_from_engine
from app.bootstrap_contract import BootstrapWorkflowResult
from app.reranker_evaluation import (
    OutcomeDataset,
    evaluate_dataset,
    temporal_train_test_split,
)
from app.trained_reranker_config import DEFAULT_TRAINED_RERANKER_MODEL_PATH
from app.trained_reranker import train_model
from app.trained_reranker.bpr_model import bpr_candidate_available, train_bpr_model

logger = logging.getLogger(__name__)

DEFAULT_MODEL_PATH = DEFAULT_TRAINED_RERANKER_MODEL_PATH

FetchLabeledExamples = Callable[[str, str | None], Awaitable[OutcomeDataset]]


async def bootstrap_and_retrain(
    profile_id: str,
    min_examples: int = 30,
    model_path: Path = DEFAULT_MODEL_PATH,
    base_url: str | None = None,
    *,
    fetch_examples: FetchLabeledExamples = fetch_labeled_examples_from_engine,
) -> BootstrapWorkflowResult:
    dataset = await fetch_examples(profile_id, base_url)
    example_count = len(dataset.examples)

    if example_count < min_examples:
        logger.warning(
            "not enough labeled examples to retrain: got %d, need %d (profile=%s)",
            example_count,
            min_examples,
            profile_id,
        )
        return BootstrapWorkflowResult.insufficient_examples(
            example_count=example_count,
            min_examples=min_examples,
        )

    train_examples, _ = temporal_train_test_split(dataset.examples)
    candidate_dataset = (
        OutcomeDataset(
            profile_id=dataset.profile_id,
            label_policy_version=dataset.label_policy_version,
            examples=train_examples,
        )
        if train_examples
        else dataset
    )
    candidate_model = train_model([candidate_dataset])
    evaluation = evaluate_dataset(dataset, trained_model=candidate_model)
    benchmark = None
    if bpr_candidate_available([candidate_dataset]):
        bpr_model = train_bpr_model([candidate_dataset])
        bpr_evaluation = evaluate_dataset(dataset, trained_model=bpr_model)
        logistic_hit_rate = next(
            (
                variant.positive_hit_rate
                for variant in evaluation.variants
                if variant.variant == "trained_reranker_prediction"
            ),
            0.0,
        )
        bpr_hit_rate = next(
            (
                variant.positive_hit_rate
                for variant in bpr_evaluation.variants
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
    else:
        benchmark = {
            "baseline_model_type": "logistic_regression",
            "candidate_model_type": "bpr",
            "baseline_positive_hit_rate": next(
                (
                    variant.positive_hit_rate
                    for variant in evaluation.variants
                    if variant.variant == "trained_reranker_prediction"
                ),
                0.0,
            ),
            "candidate_positive_hit_rate": 0.0,
            "candidate_available": False,
            "winner": "logistic_regression",
        }
    model = train_model([dataset])
    model.save(model_path)
    logger.info(
        "retrained reranker: %d examples, loss=%.6f, test_positive_hit_rate=%.6f, saved to %s",
        example_count,
        model.artifact.training.loss,
        next(
            (
                variant.positive_hit_rate
                for variant in evaluation.variants
                if variant.variant == "trained_reranker_prediction"
            ),
            0.0,
        ),
        model_path,
    )
    return BootstrapWorkflowResult.trained_model(
        example_count=example_count,
        model_path=model_path,
        training=model.artifact.training,
        evaluation=evaluation,
        benchmark=benchmark,
        feature_importances=model.feature_importances(),
    )
