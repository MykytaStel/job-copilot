from __future__ import annotations

from app.metrics import RankingVariantMetrics, RerankerEvaluationSummary


def select_feature_set(
    baseline_evaluation: RerankerEvaluationSummary,
    ablated_evaluation: RerankerEvaluationSummary,
    baseline_feature_names: list[str],
    ablated_feature_names: list[str],
) -> tuple[list[str], RerankerEvaluationSummary, str]:
    baseline_variant = _trained_variant(baseline_evaluation)
    ablated_variant = _trained_variant(ablated_evaluation)

    baseline_tuple = _variant_rank_tuple(baseline_variant)
    ablated_tuple = _variant_rank_tuple(ablated_variant)

    if ablated_tuple >= baseline_tuple:
        return ablated_feature_names, ablated_evaluation, "ablated_without_learned_scores"

    return baseline_feature_names, baseline_evaluation, "full_feature_set"


def _trained_variant(summary: RerankerEvaluationSummary) -> RankingVariantMetrics:
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
