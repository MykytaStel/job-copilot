import json
from pathlib import Path

from pydantic import BaseModel, Field

from app.reranker_evaluation import OutcomeDataset
from app.reranker_signal_weights import (
    DEFAULT_OUTCOME_CONFIDENCE_WEIGHTS,
    DEFAULT_OUTCOME_SIGNAL_WEIGHTS,
)


ARTIFACT_VERSION = "trained_reranker_v3"
MODEL_TYPE = "logistic_regression"
DEFAULT_FEATURE_NAMES = [
    "deterministic_score",
    "behavior_score_delta",
    "behavior_score",
    "learned_reranker_score_delta",
    "learned_reranker_score",
    "matched_role_count",
    "matched_skill_count",
    "matched_keyword_count",
    "source_present",
    "role_family_present",
    "outcome_received_offer",
    "outcome_reached_interview",
    "outcome_rejected",
    "has_salary_rejection",
    "has_remote_rejection",
    "has_tech_rejection",
    "interest_rating_positive",
    "interest_rating_negative",
    "work_mode_deal_breaker",
    "scrolled_to_bottom",
    "returned_count",
    "quick_apply",
    "delayed_apply",
    "legitimacy_suspicious",
]
FEATURE_TRANSFORMS = {
    "deterministic_score": "score / 100",
    "behavior_score_delta": "clamp(delta, -25, 25) / 25",
    "behavior_score": "score / 100",
    "learned_reranker_score_delta": "clamp(delta, -25, 25) / 25",
    "learned_reranker_score": "score / 100",
    "matched_role_count": "clamp(count, 0, 10) / 10",
    "matched_skill_count": "clamp(count, 0, 20) / 20",
    "matched_keyword_count": "clamp(count, 0, 20) / 20",
    "source_present": "1 if source is present else 0",
    "role_family_present": "1 if role_family is present else 0",
    "outcome_received_offer": "1 if received_offer else 0",
    "outcome_reached_interview": "1 if reached_interview else 0",
    "outcome_rejected": "1 if was_rejected else 0",
    "has_salary_rejection": "1 if has_salary_rejection else 0",
    "has_remote_rejection": "1 if has_remote_rejection else 0",
    "has_tech_rejection": "1 if has_tech_rejection else 0",
    "interest_rating_positive": "clamp(max(0, rating), 0, 2) / 2",
    "interest_rating_negative": "clamp(max(0, -rating), 0, 2) / 2",
    "work_mode_deal_breaker": "1 if work_mode_deal_breaker else 0",
    "scrolled_to_bottom": "1 if scrolled_to_bottom else 0",
    "returned_count": "clamp(count, 0, 5) / 5",
    "quick_apply": "1 if time_to_apply_days is not None and <= 3 else 0",
    "delayed_apply": "1 if time_to_apply_days is not None and > 14 else 0",
    "legitimacy_suspicious": "1 if legitimacy_suspicious else 0",
}


class TrainingSummary(BaseModel):
    example_count: int
    positive_count: int
    medium_count: int
    negative_count: int
    saved_only_count: int = 0
    viewed_only_count: int = 0
    medium_default_count: int = 0
    epochs: int
    learning_rate: float
    l2: float
    loss: float


class TrainedRerankerArtifact(BaseModel):
    artifact_version: str = ARTIFACT_VERSION
    model_type: str = MODEL_TYPE
    label_policy_version: str
    signal_weight_policy_version: str = DEFAULT_OUTCOME_SIGNAL_WEIGHTS.policy_version
    signal_weights: dict[str, float] = Field(
        default_factory=DEFAULT_OUTCOME_SIGNAL_WEIGHTS.as_dict
    )
    confidence_weight_policy_version: str = DEFAULT_OUTCOME_CONFIDENCE_WEIGHTS.policy_version
    confidence_weights: dict[str, float] = Field(
        default_factory=DEFAULT_OUTCOME_CONFIDENCE_WEIGHTS.as_dict
    )
    temporal_decay_lambda: float = Field(default=0.01, ge=0.0)
    feature_names: list[str]
    feature_transforms: dict[str, str]
    weights: dict[str, float]
    feature_importances: dict[str, float] = Field(default_factory=dict)
    intercept: float
    max_score_delta: int = Field(default=8, ge=1, le=20)
    training: TrainingSummary


def load_dataset(path: str | Path) -> OutcomeDataset:
    with open(path, "r", encoding="utf-8") as handle:
        payload = json.load(handle)
    return OutcomeDataset.model_validate(payload)
