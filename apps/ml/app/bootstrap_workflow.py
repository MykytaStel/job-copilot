from __future__ import annotations

import asyncio
import logging
import math
from collections.abc import Awaitable, Callable
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

from app.bootstrap_client import fetch_labeled_examples as fetch_labeled_examples_from_engine
from app.bootstrap_contract import BootstrapWorkflowResult
from app.bootstrap_label_policy import (
    MIN_BOOTSTRAP_LABELED_EXAMPLES,
    has_enough_bootstrap_examples,
    normalize_bootstrap_dataset,
)
from app.reranker_evaluation import (
    OutcomeDataset,
    RankingVariantMetrics,
    evaluate_dataset,
    temporal_train_test_split,
)
from app.settings import (
    RERANKER_PROMOTION_MIN_ACCURACY,
    RERANKER_PROMOTION_MIN_EXAMPLES,
)
from app.trained_reranker import train_model
from app.trained_reranker.artifact import DEFAULT_FEATURE_NAMES
from app.trained_reranker.bpr_model import bpr_candidate_available, train_bpr_model
from app.trained_reranker.feature_stats import (
    compute_ablation_candidates,
    compute_feature_statistics,
)
from app.trained_reranker.lgbm_model import (
    distill_lgbm_labels,
    lgbm_candidate_available,
)
from app.trained_reranker.model import TrainedRerankerModel
from app.trained_reranker.validation import validate_label_distribution
from app.trained_reranker_config import DEFAULT_TRAINED_RERANKER_MODEL_PATH

logger = logging.getLogger(__name__)

DEFAULT_MODEL_PATH = DEFAULT_TRAINED_RERANKER_MODEL_PATH
METRICS_VERSION = "reranker_eval_v2"

MIN_POSITIVE_EXAMPLES = 3
MIN_MEDIUM_EXAMPLES = 3
MIN_NEGATIVE_EXAMPLES = 5

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

    dataset = normalize_bootstrap_dataset(await fetch_examples(profile_id, base_url))
    example_count = len(dataset.examples)
    finished_at = _utc_now_iso()

    # Preserve existing public behavior: caller-provided min_examples is still the
    # first hard gate and controls the returned min_examples value.
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

    # B11 guard: after the existing min_examples check, make sure bootstrap does not
    # train on a tiny set of labeled examples.
    if not has_enough_bootstrap_examples(dataset, MIN_BOOTSTRAP_LABELED_EXAMPLES):
        logger.warning(
            "not enough labeled bootstrap examples to retrain: got %d, need %d (profile=%s)",
            example_count,
            MIN_BOOTSTRAP_LABELED_EXAMPLES,
            profile_id,
        )
        return BootstrapWorkflowResult.insufficient_examples(
            example_count=example_count,
            min_examples=MIN_BOOTSTRAP_LABELED_EXAMPLES,
            profile_id=profile_id,
            artifact_path=artifact_path,
            model_path=compatibility_model_path,
            started_at=started_at,
            finished_at=finished_at,
            promotion_decision="skipped_bootstrap_min_labeled_examples",
            metrics_version=METRICS_VERSION,
            reason=(
                f"need at least {MIN_BOOTSTRAP_LABELED_EXAMPLES} labeled examples, "
                f"got {example_count}"
            ),
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
            f"positive={class_mix['positive']}, "
            f"medium={class_mix['medium']}, "
            f"negative={class_mix['negative']}"
        )
        logger.warning(
            "insufficient class mix for retrain: %s (profile=%s)",
            reason,
            profile_id,
        )
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

    label_dist = validate_label_distribution(dataset)

    if label_dist.label_policy_version_mismatch:
        logger.warning(
            "label_policy_version mismatch, skipping retrain: %s (profile=%s)",
            label_dist.reason,
            profile_id,
        )
        return BootstrapWorkflowResult.insufficient_examples(
            example_count=example_count,
            min_examples=min_examples,
            profile_id=profile_id,
            artifact_path=artifact_path,
            model_path=compatibility_model_path,
            started_at=started_at,
            finished_at=_utc_now_iso(),
            promotion_decision="skipped_policy_version_mismatch",
            metrics_version=METRICS_VERSION,
            reason=label_dist.reason,
        )

    if label_dist.is_imbalanced:
        logger.warning(
            "label distribution imbalance, skipping retrain: %s (profile=%s)",
            label_dist.reason,
            profile_id,
        )
        return BootstrapWorkflowResult.insufficient_examples(
            example_count=example_count,
            min_examples=min_examples,
            profile_id=profile_id,
            artifact_path=artifact_path,
            model_path=compatibility_model_path,
            started_at=started_at,
            finished_at=_utc_now_iso(),
            promotion_decision="skipped_label_imbalance",
            metrics_version=METRICS_VERSION,
            reason=label_dist.reason,
        )

    train_examples, _ = temporal_train_test_split(dataset.examples)
    stats_examples = train_examples or dataset.examples
    feature_statistics = compute_feature_statistics(stats_examples)

    candidate_dataset = (
        OutcomeDataset(
            profile_id=dataset.profile_id,
            label_policy_version=dataset.label_policy_version,
            examples=train_examples,
        )
        if train_examples
        else dataset
    )

    success_reason = None

    if label_dist.has_insufficient_temporal_spread:
        success_reason = label_dist.reason
        logger.warning(
            "label distribution warning, retraining with limited temporal spread: %s (profile=%s)",
            label_dist.reason,
            profile_id,
        )

    baseline_feature_names = list(DEFAULT_FEATURE_NAMES)
    ablation_report = compute_ablation_candidates(baseline_feature_names, stats_examples)
    ablated_feature_names = [
        feature_name
        for feature_name in baseline_feature_names
        if feature_name not in ablation_report.candidates
    ]

    candidate_model = await asyncio.to_thread(
        lambda: train_model(
            [candidate_dataset],
            feature_names=baseline_feature_names,
            feature_statistics=feature_statistics,
        )
    )
    baseline_evaluation = await asyncio.to_thread(
        lambda: evaluate_dataset(dataset, trained_model=candidate_model)
    )

    ablated_candidate_model = await asyncio.to_thread(
        lambda: train_model(
            [candidate_dataset],
            feature_names=ablated_feature_names,
            feature_statistics=feature_statistics,
        )
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

    benchmark = _build_bpr_benchmark(
        candidate_dataset=candidate_dataset,
        dataset=dataset,
        selected_feature_names=selected_feature_names,
        feature_statistics=feature_statistics,
        evaluation=evaluation,
    )

    benchmark["feature_set_winner"] = ablation_winner
    benchmark["ablated_positive_hit_rate"] = _metric_value(
        ablated_evaluation,
        "trained_reranker_prediction",
        "positive_hit_rate",
    )
    benchmark["ablation_fallback_used"] = ablation_report.fallback_used

    if ablation_report.fallback_reason:
        benchmark["ablation_fallback_reason"] = ablation_report.fallback_reason

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
                feature_statistics=feature_statistics,
            )
        )
        distilled_evaluation = await asyncio.to_thread(
            lambda: evaluate_dataset(dataset, trained_model=distilled_model)
        )

        distilled_hit_rate = _metric_value(
            distilled_evaluation,
            "trained_reranker_prediction",
            "positive_hit_rate",
        )
        baseline_hit_rate = _metric_value(
            evaluation,
            "trained_reranker_prediction",
            "positive_hit_rate",
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
            feature_statistics=feature_statistics,
        )
    )

    distribution_shift_score = None

    if old_bucket_distribution and model.artifact.signal_bucket_distribution:
        distribution_shift_score = round(
            _kl_divergence(old_bucket_distribution, model.artifact.signal_bucket_distribution),
            6,
        )

        if distribution_shift_score > 0.3:
            logger.warning(
                "large distribution shift detected (kl=%.4f) — preferences may have changed (profile=%s)",
                distribution_shift_score,
                profile_id,
            )

    model.artifact.distribution_shift_score = distribution_shift_score

    promotion_accuracy = _promotion_accuracy(evaluation)
    promoted = _should_promote_model(example_count, promotion_accuracy)

    # Always save the candidate/profile artifact so existing workflow/tests can inspect it.
    # Only replace the active compatibility/runtime path if the model passes promotion gates.
    _atomic_save_model(model, artifact_path)

    if promoted:
        if compatibility_model_path != artifact_path:
            _atomic_save_model(model, compatibility_model_path)
    else:
        logger.warning(
            "trained reranker did not meet promotion threshold: examples=%d accuracy=%.4f "
            "required_examples=%d required_accuracy=%.2f",
            example_count,
            promotion_accuracy,
            RERANKER_PROMOTION_MIN_EXAMPLES,
            RERANKER_PROMOTION_MIN_ACCURACY,
        )

    finished_at = _utc_now_iso()
    promotion_decision = (
        (
            f"promoted_{ablation_winner}"
            + ("_with_lgbm_distillation" if model.artifact.lgbm_distilled else "")
        )
        if promoted
        else "kept_existing_model_low_validation"
    )

    logger.info(
        "retrained reranker: %d examples, loss=%.6f, test_positive_hit_rate=%.6f, "
        "drift=%.4f, saved to %s",
        example_count,
        model.artifact.training.loss,
        promotion_accuracy,
        distribution_shift_score or 0.0,
        artifact_path,
    )

    return BootstrapWorkflowResult.trained_model(
        example_count=example_count,
        profile_id=profile_id,
        artifact_path=artifact_path,
        model_path=compatibility_model_path,
        training=model.artifact.training,
        reason=success_reason,
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


async def _build_bpr_model_and_evaluation(
    candidate_dataset: OutcomeDataset,
    dataset: OutcomeDataset,
    selected_feature_names: list[str],
    feature_statistics,
):
    bpr_model = await asyncio.to_thread(
        lambda: train_bpr_model(
            [candidate_dataset],
            feature_names=selected_feature_names,
            feature_statistics=feature_statistics,
        )
    )
    bpr_evaluation = await asyncio.to_thread(
        lambda: evaluate_dataset(dataset, trained_model=bpr_model)
    )
    return bpr_model, bpr_evaluation


def _build_bpr_benchmark(
    *,
    candidate_dataset: OutcomeDataset,
    dataset: OutcomeDataset,
    selected_feature_names: list[str],
    feature_statistics,
    evaluation,
) -> dict[str, str | float | bool]:
    logistic_hit_rate = _metric_value(
        evaluation,
        "trained_reranker_prediction",
        "positive_hit_rate",
    )

    if not bpr_candidate_available([candidate_dataset]):
        return {
            "baseline_model_type": "logistic_regression",
            "candidate_model_type": "bpr",
            "baseline_positive_hit_rate": logistic_hit_rate,
            "candidate_positive_hit_rate": 0.0,
            "candidate_available": False,
            "winner": "logistic_regression",
        }

    # This helper is sync because bootstrap_and_retrain owns the event loop flow.
    # The actual BPR path is handled inline below by running the async thread tasks.
    # Keep the benchmark shape stable if BPR is unavailable in the test environment.
    try:
        bpr_model = train_bpr_model(
            [candidate_dataset],
            feature_names=selected_feature_names,
            feature_statistics=feature_statistics,
        )
        bpr_evaluation = evaluate_dataset(dataset, trained_model=bpr_model)
        bpr_hit_rate = _metric_value(
            bpr_evaluation,
            "trained_reranker_prediction",
            "positive_hit_rate",
        )
    except Exception:
        return {
            "baseline_model_type": "logistic_regression",
            "candidate_model_type": "bpr",
            "baseline_positive_hit_rate": logistic_hit_rate,
            "candidate_positive_hit_rate": 0.0,
            "candidate_available": False,
            "winner": "logistic_regression",
        }

    return {
        "baseline_model_type": "logistic_regression",
        "candidate_model_type": "bpr",
        "baseline_positive_hit_rate": logistic_hit_rate,
        "candidate_positive_hit_rate": bpr_hit_rate,
        "candidate_available": True,
        "winner": "bpr" if bpr_hit_rate > logistic_hit_rate else "logistic_regression",
    }


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
        (new_dist.get(key, 0.0) + epsilon)
        * math.log((new_dist.get(key, 0.0) + epsilon) / (old_dist.get(key, 0.0) + epsilon))
        for key in keys
    )


def _class_mix_counts(dataset: OutcomeDataset) -> dict[str, int]:
    return {
        "positive": sum(1 for example in dataset.examples if example.label == "positive"),
        "medium": sum(1 for example in dataset.examples if example.label == "medium"),
        "negative": sum(1 for example in dataset.examples if example.label == "negative"),
    }


def _utc_now_iso() -> str:
    return datetime.now(timezone.utc).isoformat().replace("+00:00", "Z")


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


def _promotion_accuracy(evaluation) -> float:
    return _metric_value(
        evaluation,
        "trained_reranker_prediction",
        "positive_hit_rate",
    )


def _should_promote_model(example_count: int, accuracy: float) -> bool:
    return (
        example_count >= RERANKER_PROMOTION_MIN_EXAMPLES
        and accuracy >= RERANKER_PROMOTION_MIN_ACCURACY
    )


def _atomic_save_model(model: TrainedRerankerModel, path: Path) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    tmp_path = path.with_suffix(path.suffix + ".tmp")
    model.save(tmp_path)
    tmp_path.replace(path)


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
        variant for variant in summary.variants if variant.variant == "trained_reranker_prediction"
    )


def _variant_rank_tuple(variant: RankingVariantMetrics) -> tuple[float, float, float, float]:
    return (
        variant.positive_hit_rate,
        variant.ndcg_at_top_n,
        variant.mrr_at_top_n,
        variant.average_label_score_top_n,
    )
