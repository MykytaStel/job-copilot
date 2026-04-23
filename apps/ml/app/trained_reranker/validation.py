from datetime import datetime, timezone

from pydantic import BaseModel

from app.dataset import OutcomeDataset, OutcomeExample

EXPECTED_LABEL_POLICY_VERSION = "outcome_label_v3"
_MIN_TEMPORAL_SPREAD_DAYS = 1.0
_MIN_TEMPORAL_COVERAGE_RATIO = 0.5


class LabelDistributionReport(BaseModel):
    total: int
    positive: int
    medium: int
    negative: int
    positive_ratio: float
    is_imbalanced: bool
    has_insufficient_temporal_spread: bool
    label_policy_version_mismatch: bool
    expected_label_policy_version: str = EXPECTED_LABEL_POLICY_VERSION
    actual_label_policy_version: str
    healthy: bool
    reason: str | None = None


def validate_label_distribution(dataset: OutcomeDataset) -> LabelDistributionReport:
    examples = dataset.examples
    total = len(examples)
    positive = sum(1 for e in examples if e.label == "positive")
    medium = sum(1 for e in examples if e.label == "medium")
    negative = sum(1 for e in examples if e.label == "negative")
    positive_ratio = positive / total if total > 0 else 0.0

    is_imbalanced = positive_ratio < 0.05 or positive_ratio > 0.70
    has_insufficient_temporal_spread = _check_insufficient_temporal_spread(examples)
    label_policy_version_mismatch = dataset.label_policy_version != EXPECTED_LABEL_POLICY_VERSION

    reasons: list[str] = []
    if is_imbalanced:
        reasons.append(
            f"extreme label imbalance: positive_ratio={positive_ratio:.2f} (expected 0.05-0.70)"
        )
    if has_insufficient_temporal_spread:
        reasons.append("insufficient temporal spread across examples")
    if label_policy_version_mismatch:
        reasons.append(
            f"label_policy_version mismatch: "
            f"expected={EXPECTED_LABEL_POLICY_VERSION!r} actual={dataset.label_policy_version!r}"
        )

    healthy = not is_imbalanced and not label_policy_version_mismatch
    reason = "; ".join(reasons) if reasons else None

    return LabelDistributionReport(
        total=total,
        positive=positive,
        medium=medium,
        negative=negative,
        positive_ratio=round(positive_ratio, 4),
        is_imbalanced=is_imbalanced,
        has_insufficient_temporal_spread=has_insufficient_temporal_spread,
        label_policy_version_mismatch=label_policy_version_mismatch,
        actual_label_policy_version=dataset.label_policy_version,
        healthy=healthy,
        reason=reason,
    )


def _check_insufficient_temporal_spread(examples: list[OutcomeExample]) -> bool:
    timestamps = [_parse_ts(e.label_observed_at) for e in examples]
    parsed = [ts for ts in timestamps if ts is not None]

    if len(parsed) < len(examples) * _MIN_TEMPORAL_COVERAGE_RATIO:
        return True
    if len(parsed) < 2:
        return False

    spread_days = (max(parsed) - min(parsed)).total_seconds() / 86400.0
    return spread_days < _MIN_TEMPORAL_SPREAD_DAYS


def _parse_ts(value: str | None) -> datetime | None:
    if not value:
        return None
    normalized = value.strip()
    if normalized.endswith("Z"):
        normalized = normalized[:-1] + "+00:00"
    try:
        parsed = datetime.fromisoformat(normalized)
        if parsed.tzinfo is None:
            return parsed.replace(tzinfo=timezone.utc)
        return parsed.astimezone(timezone.utc)
    except ValueError:
        return None
