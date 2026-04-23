from __future__ import annotations

from pydantic import BaseModel, Field

from app.dataset import OutcomeExample

FALLBACK_ABLATED_FEATURES = frozenset({"learned_reranker_score", "learned_reranker_score_delta"})
_LOW_VARIANCE_THRESHOLD = 1e-4
_MIN_EXAMPLES_FOR_STATS = 5


class FeatureStatistics(BaseModel):
    matched_skill_count_p95: float = Field(default=20.0, ge=1.0)
    matched_keyword_count_p95: float = Field(default=20.0, ge=1.0)
    matched_role_count_p95: float = Field(default=10.0, ge=1.0)
    returned_count_p95: float = Field(default=5.0, ge=1.0)
    interest_rating_min: float = Field(default=-2.0)
    interest_rating_max: float = Field(default=2.0)


def compute_feature_statistics(examples: list[OutcomeExample]) -> FeatureStatistics:
    if len(examples) < _MIN_EXAMPLES_FOR_STATS:
        return FeatureStatistics()

    skill_counts = [e.ranking.matched_skill_count for e in examples]
    keyword_counts = [e.ranking.matched_keyword_count for e in examples]
    role_counts = [e.ranking.matched_role_count for e in examples]
    returned_counts = [
        getattr(e.signals, "returned_count", 0) if e.signals else 0
        for e in examples
    ]
    ratings = [
        int(e.signals.interest_rating)
        for e in examples
        if e.signals and getattr(e.signals, "interest_rating", None) is not None
    ]

    return FeatureStatistics(
        matched_skill_count_p95=max(1.0, _p95(skill_counts)),
        matched_keyword_count_p95=max(1.0, _p95(keyword_counts)),
        matched_role_count_p95=max(1.0, _p95(role_counts)),
        returned_count_p95=max(1.0, _p95(returned_counts)),
        interest_rating_min=min(ratings) if ratings else -2.0,
        interest_rating_max=max(ratings) if ratings else 2.0,
    )


def compute_ablation_candidates(
    feature_names: list[str],
    examples: list[OutcomeExample],
) -> set[str]:
    if len(examples) < _MIN_EXAMPLES_FOR_STATS:
        return set(FALLBACK_ABLATED_FEATURES)

    try:
        from app.trained_reranker.features import extract_features

        vectors: dict[str, list[float]] = {name: [] for name in feature_names}
        for example in examples:
            features = extract_features(example)
            for name in feature_names:
                vectors[name].append(features.get(name, 0.0))

        candidates = {
            name
            for name in feature_names
            if _variance(vectors[name]) < _LOW_VARIANCE_THRESHOLD
        }
        return candidates if candidates else set(FALLBACK_ABLATED_FEATURES)
    except Exception:
        return set(FALLBACK_ABLATED_FEATURES)


def _p95(values: list[float | int]) -> float:
    if not values:
        return 0.0
    sorted_values = sorted(float(v) for v in values)
    index = min(int(len(sorted_values) * 0.95), len(sorted_values) - 1)
    return sorted_values[index]


def _variance(values: list[float]) -> float:
    if len(values) < 2:
        return 0.0
    mean = sum(values) / len(values)
    return sum((v - mean) ** 2 for v in values) / len(values)
