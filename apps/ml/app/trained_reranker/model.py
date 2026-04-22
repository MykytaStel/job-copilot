import json
import math
from pathlib import Path
from typing import Any

from pydantic import BaseModel

from app.reranker_evaluation import OutcomeExample

from .artifact import ARTIFACT_VERSION, FEATURE_TRANSFORMS, MODEL_TYPE, TrainedRerankerArtifact
from .features import extract_features


class TrainedRerankerModel:
    def __init__(self, artifact: TrainedRerankerArtifact):
        if artifact.artifact_version != ARTIFACT_VERSION:
            raise ValueError(f"unsupported artifact_version: {artifact.artifact_version}")
        if artifact.model_type != MODEL_TYPE:
            raise ValueError(f"unsupported model_type: {artifact.model_type}")
        for feature_name in artifact.feature_names:
            if feature_name not in FEATURE_TRANSFORMS:
                raise ValueError(f"unsupported feature: {feature_name}")
            if feature_name not in artifact.weights:
                raise ValueError(f"missing weight for feature: {feature_name}")
        self.artifact = artifact

    @classmethod
    def load(cls, path: str | Path) -> "TrainedRerankerModel":
        with open(path, "r", encoding="utf-8") as handle:
            payload = json.load(handle)
        return cls(TrainedRerankerArtifact.model_validate(payload))

    def save(self, path: str | Path) -> None:
        output_path = Path(path)
        output_path.parent.mkdir(parents=True, exist_ok=True)
        with open(output_path, "w", encoding="utf-8") as handle:
            handle.write(self.artifact.model_dump_json(indent=2))
            handle.write("\n")

    def feature_vector(self, example: OutcomeExample | dict[str, Any]) -> dict[str, float]:
        if isinstance(example, OutcomeExample):
            parsed = example
        elif isinstance(example, BaseModel):
            parsed = OutcomeExample.model_validate(example.model_dump())
        else:
            parsed = OutcomeExample.model_validate(example)
        all_features = extract_features(parsed)
        return {name: all_features[name] for name in self.artifact.feature_names}

    def predict_probability(self, example: OutcomeExample | dict[str, Any]) -> float:
        features = self.feature_vector(example)
        logit = self.artifact.intercept + sum(
            self.artifact.weights[name] * features[name]
            for name in self.artifact.feature_names
        )
        return sigmoid(logit)

    def feature_importances(self) -> dict[str, float]:
        if self.artifact.feature_importances:
            return dict(self.artifact.feature_importances)
        return compute_feature_importances(
            self.artifact.weights,
            self.artifact.feature_names,
        )

    def score_delta(self, example: OutcomeExample | dict[str, Any]) -> int:
        probability = self.predict_probability(example)
        raw_delta = round((probability - 0.5) * 2 * self.artifact.max_score_delta)
        return int(
            max(
                -self.artifact.max_score_delta,
                min(self.artifact.max_score_delta, raw_delta),
            )
        )


def sigmoid(value: float) -> float:
    if value >= 0:
        z = math.exp(-value)
        return 1.0 / (1.0 + z)
    z = math.exp(value)
    return z / (1.0 + z)


def compute_feature_importances(
    weights: dict[str, float],
    feature_names: list[str],
) -> dict[str, float]:
    total = sum(abs(weights.get(feature_name, 0.0)) for feature_name in feature_names)
    if total <= 0:
        return {feature_name: 0.0 for feature_name in feature_names}

    return {
        feature_name: round(abs(weights.get(feature_name, 0.0)) / total, 8)
        for feature_name in feature_names
    }
