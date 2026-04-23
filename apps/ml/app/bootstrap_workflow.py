import asyncio
import logging
import math
from collections.abc import Awaitable, Callable
from datetime import datetime, timezone
from pathlib import Path

from app.bootstrap_client import fetch_labeled_examples as fetch_labeled_examples_from_engine
from app.bootstrap_contract import BootstrapWorkflowResult
from app.reranker_evaluation import (
    OutcomeDataset,
    RankingVariantMetrics,
    evaluate_dataset,
    temporal_train_test_split,
)
from app.trained_reranker.artifact import DEFAULT_FEATURE_NAMES
from app.trained_reranker_config import DEFAULT_TRAINED_RERANKER_MODEL_PATH
from app.trained_reranker import train_model
from app.trained_reranker.bpr_model import bpr_candidate_available, train_bpr_model
from app.trained_reranker.lgbm_model import distill_lgbm_labels, lgbm_candidate_available
from app.trained_reranker.model import TrainedRerankerModel

logger = logging.getLogger(__name__)

DEFAULT_MODEL_PATH = DEFAULT_TRAINED_RERANKER_MODEL_PATH
METRICS_VERSION = "reranker_eval_v2"
MIN_POSITIVE_EXAMPLES = 3
MIN_MEDIUM_EXAMPLES = 3
MIN_NEGATIVE_EXAMPLES = 5
ABLATED_FEATURES = {"learned_reranker_score", "learned_reranker_score_delta"}

FetchLabeledExamples = Callable[[str, str | None], Awaitable[OutcomeDataset]]


async def bootstrap_and_retrain(
    profile_id: str,
    min_examples: int = 30,
    artifact_path: Path = DEFAULT_MODEL_PATH,
    compatibility_model_path: Path = DEFAULT_MODEL_PATH,
    base_url: str | None = None,
    *,
    fetch_examples: FetchLabeledExamples = fetch_labeled_examples_from_engine,
) -> BootstrapWorkflowResult:
    started_at = _utc_now_iso()
    dataset = await fetch_examples(profile_id, base_url)
    example_count = len(dataset.examples)
    finished_at = _utc_now_iso()

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
            profile_id=profile_id,
            artifact_path=artifact_path,
            model_path=compatibility_model_path,
            started_at=started_at,
            finished_at=finished_at,
            promotion_decision="skipped_min_examples",
            metrics_version=METRICS_VERSION,
        )

    class_mix = _class_mix_counts(dataset)
    if (
        class_mix["positive"] < MIN_POSITIVE_EXAMPLES
        or class_mix["medium"] < MIN_MEDIUM_EXAMPLES
        or class_mix["negative"] < MIN_NEGATIVE_EXAMPLES
    ):
        reason = (
            "need class mix "
            f"positive>={MIN_POSITIVE_EXAMPLES}, medium>={MIN_MEDIUM_EXAMPLES}, "
            f"negative>={MIN_NEGATIVE_EXAMPLES}; got "
            f"positive={class_mix['positive']}, medium={class_mix['medium']}, negative={class_mix['negative']}"
        )
        logger.warning("insufficient class mix for retrain: %s (profile=%s)", reason, profile_id)
        return BootstrapWorkflowResult.insufficient_examples(
            example_count=example_count,
            min_examples=min_examples,
            profile_id=profile_id,
            artifact_path=artifact_path,
            model_path=compatibility_model_path,
            started_at=started_at,
            finished_at=_utc_now_iso(),
            promotion_decision="skipped_class_mix",
            metrics_version=METRICS_VERSION,
            reason=reason,
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
    baseline_feature_names = list(DEFAULT_FEATURE_NAMES)
    ablated_feature_names = [
        feature_name for feature_name in baseline_feature_names if feature_name not in ABLATED_FEATURES
    ]

    candidate_model = await asyncio.to_thread(
        lambda: train_model([candidate_dataset], feature_names=baseline_feature_names)
    )
    baseline_evaluation = await asyncio.to_thread(
        lambda: evaluate_dataset(dataset, trained_model=candidate_model)
    )
    ablated_candidate_model = await asyncio.to_thread(
        lambda: train_model([candidate_dataset], feature_names=ablated_feature_names)
    )
    ablated_evaluation = await asyncio.to_thread(
        lambda: evaluate_dataset(dataset, trained_model=ablated_candidate_model)
    )
    selected_feature_names, evaluation, ablation_winner = _select_feature_set(
        baseline_evaluation,
        ablated_evaluation,
        baseline_feature_names,
        ablated_feature_names,
    )
    benchmark = None
    if bpr_candidate_available([candidate_dataset]):
        bpr_model = await asyncio.to_thread(
            lambda: train_bpr_model([candidate_dataset], feature_names=selected_feature_names)
        )
        bpr_evaluation = await asyncio.to_thread(
            lambda: evaluate_dataset(dataset, trained_model=bpr_model)
        )
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
    benchmark["feature_set_winner"] = ablation_winner
    benchmark["ablated_positive_hit_rate"] = _metric_value(
        ablated_evaluation,
        "trained_reranker_prediction",
        "positive_hit_rate",
    )
    benchmark["baseline_positive_hit_rate"] = _metric_value(
        evaluation,
        "trained_reranker_prediction",
        "positive_hit_rate",
    )
    old_bucket_distribution = _load_bucket_distribution(artifact_path)

    distilled_labels = None
    if lgbm_candidate_available([dataset]):
        logger.info("lgbm candidate available (%d examples), distilling labels", example_count)
        distilled_labels = await asyncio.to_thread(
            lambda: distill_lgbm_labels(dataset.examples, feature_names=selected_feature_names)
        )
        distilled_model = await asyncio.to_thread(
            lambda: train_model(
                [dataset],
                distilled_labels=distilled_labels,
                feature_names=selected_feature_names,
            )
        )
        distilled_evaluation = await asyncio.to_thread(
            lambda: evaluate_dataset(dataset, trained_model=distilled_model)
        )
        distilled_hit_rate = next(
            (
                v.positive_hit_rate
                for v in distilled_evaluation.variants
                if v.variant == "trained_reranker_prediction"
            ),
            0.0,
        )
        baseline_hit_rate = next(
            (
                v.positive_hit_rate
                for v in evaluation.variants
                if v.variant == "trained_reranker_prediction"
            ),
            0.0,
        )
        logger.info(
            "lgbm distillation benchmark: baseline=%.4f distilled=%.4f",
            baseline_hit_rate,
            distilled_hit_rate,
        )
        if distilled_hit_rate < baseline_hit_rate:
            logger.info("lgbm distillation did not improve hit rate — using baseline labels")
            distilled_labels = None

    model = await asyncio.to_thread(
        lambda: train_model(
            [dataset],
            distilled_labels=distilled_labels,
            feature_names=selected_feature_names,
        )
    )
    distribution_shift_score = None
    if old_bucket_distribution and model.artifact.signal_bucket_distribution:
        distribution_shift_score = round(
            _kl_divergence(old_bucket_distribution, model.artifact.signal_bucket_distribution), 6
        )
        if distribution_shift_score > 0.3:
            logger.warning(
                "large distribution shift detected (kl=%.4f) — preferences may have changed (profile=%s)",
                distribution_shift_score,
                profile_id,
            )
    model.artifact.distribution_shift_score = distribution_shift_score
    model.save(artifact_path)
    if compatibility_model_path != artifact_path:
        model.save(compatibility_model_path)
    finished_at = _utc_now_iso()
    promotion_decision = (
        f"promoted_{ablation_winner}"
        + ("_with_lgbm_distillation" if model.artifact.lgbm_distilled else "")
    )
    logger.info(
        "retrained reranker: %d examples, loss=%.6f, test_positive_hit_rate=%.6f, drift=%.4f, saved to %s",
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
        distribution_shift_score or 0.0,
        artifact_path,
    )
    return BootstrapWorkflowResult.trained_model(
        example_count=example_count,
        profile_id=profile_id,
        artifact_path=artifact_path,
        model_path=compatibility_model_path,
        training=model.artifact.training,
        evaluation=evaluation,
        benchmark=benchmark,
        feature_importances=model.feature_importances(),
        distribution_shift_score=distribution_shift_score,
        started_at=started_at,
        finished_at=finished_at,
        promotion_decision=promotion_decision,
        metrics_version=METRICS_VERSION,
        lgbm_distilled=model.artifact.lgbm_distilled,
    )


def _load_bucket_distribution(model_path: Path) -> dict[str, float] | None:
    try:
        old_model = TrainedRerankerModel.load(model_path)
        return old_model.artifact.signal_bucket_distribution or None
    except Exception:
        return None


def _kl_divergence(old_dist: dict[str, float], new_dist: dict[str, float]) -> float:
    """KL(new || old): divergence of new distribution from old."""
    epsilon = 1e-8
    keys = set(old_dist) | set(new_dist)
    return sum(
        (new_dist.get(k, 0.0) + epsilon) * math.log((new_dist.get(k, 0.0) + epsilon) / (old_dist.get(k, 0.0) + epsilon))
        for k in keys
    )


def _class_mix_counts(dataset: OutcomeDataset) -> dict[str, int]:
    return {
        "positive": sum(1 for example in dataset.examples if example.label == "positive"),
        "medium": sum(1 for example in dataset.examples if example.label == "medium"),
        "negative": sum(1 for example in dataset.examples if example.label == "negative"),
    }


def _utc_now_iso() -> str:
    return datetime.now(timezone.utc).replace(microsecond=0).isoformat().replace("+00:00", "Z")


def _metric_value(
    summary,
    variant_name: str,
    field_name: str,
) -> float:
    variant = next(
        (variant for variant in summary.variants if variant.variant == variant_name),
        None,
    )
    if variant is None:
        return 0.0
    return float(getattr(variant, field_name, 0.0))


def _select_feature_set(
    baseline_evaluation,
    ablated_evaluation,
    baseline_feature_names: list[str],
    ablated_feature_names: list[str],
) -> tuple[list[str], object, str]:
    baseline_variant = _trained_variant(baseline_evaluation)
    ablated_variant = _trained_variant(ablated_evaluation)
    baseline_tuple = _variant_rank_tuple(baseline_variant)
    ablated_tuple = _variant_rank_tuple(ablated_variant)
    if ablated_tuple >= baseline_tuple:
        return ablated_feature_names, ablated_evaluation, "ablated_without_learned_scores"
    return baseline_feature_names, baseline_evaluation, "full_feature_set"


def _trained_variant(summary) -> RankingVariantMetrics:
    return next(
        variant
        for variant in summary.variants
        if variant.variant == "trained_reranker_prediction"
    )


def _variant_rank_tuple(variant: RankingVariantMetrics) -> tuple[float, float, float, float]:
    return (
        variant.positive_hit_rate,
        variant.ndcg_at_top_n,
        variant.mrr_at_top_n,
        variant.average_label_score_top_n,
    )
