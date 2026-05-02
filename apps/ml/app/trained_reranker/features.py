from __future__ import annotations

from typing import TYPE_CHECKING

from app.reranker_evaluation import OutcomeExample

if TYPE_CHECKING:
    from app.trained_reranker.feature_stats import FeatureStatistics


def extract_features(
    example: OutcomeExample,
    stats: FeatureStatistics | None = None,
) -> dict[str, float]:
    ranking = example.ranking
    signals = example.signals

    rating = getattr(signals, "interest_rating", None) if signals else None
    rating_val = int(rating) if rating is not None else 0

    skill_max = stats.matched_skill_count_p95 if stats else 20.0
    keyword_max = stats.matched_keyword_count_p95 if stats else 20.0
    role_max = stats.matched_role_count_p95 if stats else 10.0
    returned_max = stats.returned_count_p95 if stats else 5.0
    rating_pos_max = max(1.0, stats.interest_rating_max if stats else 2.0)
    rating_neg_max = max(1.0, -(stats.interest_rating_min if stats else -2.0))

    time_to_apply = _time_to_apply_days(signals)

    return {
        "deterministic_score": clamp(ranking.deterministic_score, 0, 100) / 100.0,
        "behavior_score_delta": clamp(ranking.behavior_score_delta, -25, 25) / 25.0,
        "behavior_score": clamp(ranking.behavior_score, 0, 100) / 100.0,
        "learned_reranker_score_delta": clamp(
            ranking.learned_reranker_score_delta,
            -25,
            25,
        )
        / 25.0,
        "learned_reranker_score": clamp(ranking.learned_reranker_score, 0, 100) / 100.0,
        "matched_role_count": clamp(ranking.matched_role_count, 0, role_max) / role_max,
        "matched_skill_count": clamp(ranking.matched_skill_count, 0, skill_max) / skill_max,
        "matched_keyword_count": clamp(ranking.matched_keyword_count, 0, keyword_max) / keyword_max,
        "source_present": 1.0 if has_text(example.source) else 0.0,
        "role_family_present": 1.0 if has_text(example.role_family) else 0.0,
        # Slice 1: outcome signals
        "outcome_received_offer": 1.0 if _bool_signal(signals, "received_offer") else 0.0,
        "outcome_reached_interview": 1.0 if _bool_signal(signals, "reached_interview") else 0.0,
        "outcome_rejected": 1.0 if _bool_signal(signals, "was_rejected") else 0.0,
        # Slice 2: rejection/positive tags
        "has_salary_rejection": 1.0 if _bool_signal(signals, "has_salary_rejection") else 0.0,
        "has_remote_rejection": 1.0 if _bool_signal(signals, "has_remote_rejection") else 0.0,
        "has_tech_rejection": 1.0 if _bool_signal(signals, "has_tech_rejection") else 0.0,
        # Slice 4: interest rating as two separate positive/negative features
        "interest_rating_positive": clamp(max(0, rating_val), 0, rating_pos_max) / rating_pos_max,
        "interest_rating_negative": clamp(max(0, -rating_val), 0, rating_neg_max) / rating_neg_max,
        # Slice 5: work mode
        "work_mode_deal_breaker": 1.0 if _bool_signal(signals, "work_mode_deal_breaker") else 0.0,
        # Slice 6: engagement depth
        "scrolled_to_bottom": 1.0 if _bool_signal(signals, "scrolled_to_bottom") else 0.0,
        "returned_count": clamp(
            getattr(signals, "returned_count", 0) if signals else 0, 0, returned_max
        )
        / returned_max,
        "quick_apply": 1.0 if time_to_apply is not None and time_to_apply <= 3 else 0.0,
        "delayed_apply": 1.0 if time_to_apply is not None and time_to_apply > 14 else 0.0,
        # Slice 7: legitimacy
        "legitimacy_suspicious": 1.0 if _bool_signal(signals, "legitimacy_suspicious") else 0.0,
    }


def _bool_signal(signals: object | None, attr: str) -> bool:
    return bool(getattr(signals, attr, False)) if signals is not None else False


def _time_to_apply_days(signals: object | None) -> int | None:
    if signals is None:
        return None
    value = getattr(signals, "time_to_apply_days", None)
    if value is None:
        return None
    return int(value)


def clamp(value: int | float, lower: int | float, upper: int | float) -> int | float:
    return max(lower, min(upper, value))


def has_text(value: str | None) -> bool:
    return value is not None and value.strip() != ""
