from collections.abc import Sequence
from typing import Any, Literal

from pydantic import BaseModel, ConfigDict, Field


OutcomeSignalBucket = Literal[
    "applied",
    "dismissed",
    "saved_only",
    "viewed_only",
    "medium_default",
]


class OutcomeSignalWeightConfig(BaseModel):
    model_config = ConfigDict(frozen=True)

    policy_version: str = Field(default="outcome_signal_weight_v1", min_length=1)
    applied: float = Field(default=1.0, ge=0.0, le=1.0)
    saved_only: float = Field(default=0.6, ge=0.0, le=1.0)
    viewed_only: float = Field(default=0.4, ge=0.0, le=1.0)
    medium_default: float = Field(default=0.5, ge=0.0, le=1.0)
    dismissed: float = Field(default=0.0, ge=0.0, le=1.0)

    def as_dict(self) -> dict[str, float]:
        return {
            "applied": self.applied,
            "saved_only": self.saved_only,
            "viewed_only": self.viewed_only,
            "medium_default": self.medium_default,
            "dismissed": self.dismissed,
        }

    def weight_for(self, bucket: OutcomeSignalBucket) -> float:
        return self.as_dict()[bucket]


DEFAULT_OUTCOME_SIGNAL_WEIGHTS = OutcomeSignalWeightConfig()


def signal_weight_config_from_payload(
    signal_weights: dict[str, float] | None,
    *,
    policy_version: str | None = None,
) -> OutcomeSignalWeightConfig:
    if not signal_weights:
        return DEFAULT_OUTCOME_SIGNAL_WEIGHTS

    payload = dict(signal_weights)
    if policy_version:
        payload["policy_version"] = policy_version
    return OutcomeSignalWeightConfig.model_validate(payload)


def resolve_example_signal_bucket(example: Any) -> OutcomeSignalBucket:
    reasons = normalized_reasons(getattr(example, "label_reasons", None))
    if "applied" in reasons:
        return "applied"
    if "dismissed" in reasons:
        return "dismissed"
    if "saved" in reasons:
        return "saved_only"
    if "viewed" in reasons:
        return "viewed_only"

    signals = getattr(example, "signals", None)
    if signals is not None:
        if bool(getattr(signals, "applied", False)):
            return "applied"
        if bool(getattr(signals, "dismissed", False)) or bool(
            getattr(signals, "hidden", False)
        ) or bool(getattr(signals, "bad_fit", False)):
            return "dismissed"
        if bool(getattr(signals, "saved", False)):
            return "saved_only"
        if bool(getattr(signals, "viewed", False)):
            return "viewed_only"

    label = str(getattr(example, "label", "")).strip().casefold()
    if label == "positive":
        return "applied"
    if label == "negative":
        return "dismissed"
    return "medium_default"


def training_target_for_example(
    example: Any,
    signal_weights: OutcomeSignalWeightConfig = DEFAULT_OUTCOME_SIGNAL_WEIGHTS,
) -> float:
    bucket = resolve_example_signal_bucket(example)
    return signal_weights.weight_for(bucket)


def normalized_reasons(raw_reasons: Sequence[str] | None) -> set[str]:
    return {
        reason.strip().casefold()
        for reason in (raw_reasons or [])
        if isinstance(reason, str) and reason.strip()
    }
