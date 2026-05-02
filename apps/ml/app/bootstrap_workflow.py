from __future__ import annotations

import asyncio
import logging
from collections.abc import Awaitable, Callable
from pathlib import Path

from app.bootstrap_benchmark import build_bpr_benchmark
from app.bootstrap_client import fetch_labeled_examples as fetch_labeled_examples_from_engine
from app.bootstrap_contract import BootstrapWorkflowResult
from app.bootstrap_feature_selection import select_feature_set
from app.bootstrap_label_policy import (
    MIN_BOOTSTRAP_LABELED_EXAMPLES,
    has_enough_bootstrap_examples,
    normalize_bootstrap_dataset,
)
from app.bootstrap_metrics import (
    class_mix_counts,
    metric_value,
    promotion_accuracy,
    should_promote_model,
)
from app.bootstrap_model_io import (
    atomic_save_model,
    kl_divergence,
    load_bucket_distribution,
    utc_now_iso,
)
from app.reranker_evaluation import (
    OutcomeDataset,
    evaluate_dataset,
    temporal_train_test_split,
)
from app.settings import (
    RERANKER_PROMOTION_MIN_ACCURACY,
    RERANKER_PROMOTION_MIN_EXAMPLES,
)
from app.trained_reranker import train_model
from app.trained_reranker.artifact import DEFAULT_FEATURE_NAMES
from app.trained_reranker.feature_stats import (
    compute_ablation_candidates,
    compute_feature_statistics,
)
from app.trained_reranker.lgbm_model import (
    distill_lgbm_labels,
    lgbm_candidate_available,
)
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
    started_at = utc_now_iso()

    dataset = normalize_bootstrap_dataset(await fetch_examples(profile_id, base_url))
    example_count = len(dataset.examples)
    finished_at = utc_now_iso()

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

    class_mix = class_mix_counts(dataset)

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
            finished_at=utc_now_iso(),
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
            finished_at=utc_now_iso(),
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
            finished_at=utc_now_iso(),
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

    selected_feature_names, evaluation, ablation_winner = select_feature_set(
        baseline_evaluation,
        ablated_evaluation,
        baseline_feature_names,
        ablated_feature_names,
    )

    benchmark = build_bpr_benchmark(
        candidate_dataset=candidate_dataset,
        dataset=dataset,
        selected_feature_names=selected_feature_names,
        feature_statistics=feature_statistics,
        evaluation=evaluation,
    )

    benchmark["feature_set_winner"] = ablation_winner
    benchmark["ablated_positive_hit_rate"] = metric_value(
        ablated_evaluation,
        "trained_reranker_prediction",
        "positive_hit_rate",
    )
    benchmark["ablation_fallback_used"] = ablation_report.fallback_used

    if ablation_report.fallback_reason:
        benchmark["ablation_fallback_reason"] = ablation_report.fallback_reason

    benchmark["baseline_positive_hit_rate"] = metric_value(
        evaluation,
        "trained_reranker_prediction",
        "positive_hit_rate",
    )

    old_bucket_distribution = load_bucket_distribution(artifact_path)

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

        distilled_hit_rate = metric_value(
            distilled_evaluation,
            "trained_reranker_prediction",
            "positive_hit_rate",
        )
        baseline_hit_rate = metric_value(
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
            kl_divergence(old_bucket_distribution, model.artifact.signal_bucket_distribution),
            6,
        )

        if distribution_shift_score > 0.3:
            logger.warning(
                "large distribution shift detected (kl=%.4f) — preferences may have changed (profile=%s)",
                distribution_shift_score,
                profile_id,
            )

    model.artifact.distribution_shift_score = distribution_shift_score

    promotion_accuracy_value = promotion_accuracy(evaluation)
    promoted = should_promote_model(example_count, promotion_accuracy_value)

    # Always save the candidate/profile artifact so existing workflow/tests can inspect it.
    # Only replace the active compatibility/runtime path if the model passes promotion gates.
    atomic_save_model(model, artifact_path)

    if promoted:
        if compatibility_model_path != artifact_path:
            atomic_save_model(model, compatibility_model_path)
    else:
        logger.warning(
            "trained reranker did not meet promotion threshold: examples=%d accuracy=%.4f "
            "required_examples=%d required_accuracy=%.2f",
            example_count,
            promotion_accuracy_value,
            RERANKER_PROMOTION_MIN_EXAMPLES,
            RERANKER_PROMOTION_MIN_ACCURACY,
        )

    finished_at = utc_now_iso()
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
        promotion_accuracy_value,
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
