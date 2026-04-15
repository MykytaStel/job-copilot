import argparse
import json
import math
import sys
from pathlib import Path
from typing import Any

from pydantic import BaseModel, Field

from app.reranker_evaluation import OutcomeDataset, OutcomeExample, evaluate_dataset


ARTIFACT_VERSION = "trained_reranker_v2"
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
}


class TrainingSummary(BaseModel):
    example_count: int
    positive_count: int
    medium_count: int
    negative_count: int
    epochs: int
    learning_rate: float
    l2: float
    loss: float


class TrainedRerankerArtifact(BaseModel):
    artifact_version: str = ARTIFACT_VERSION
    model_type: str = MODEL_TYPE
    label_policy_version: str
    feature_names: list[str]
    feature_transforms: dict[str, str]
    weights: dict[str, float]
    intercept: float
    max_score_delta: int = Field(default=8, ge=1, le=20)
    training: TrainingSummary


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

    def score_delta(self, example: OutcomeExample | dict[str, Any]) -> int:
        probability = self.predict_probability(example)
        raw_delta = round((probability - 0.5) * 2 * self.artifact.max_score_delta)
        return int(
            max(
                -self.artifact.max_score_delta,
                min(self.artifact.max_score_delta, raw_delta),
            )
        )


def extract_features(example: OutcomeExample) -> dict[str, float]:
    ranking = example.ranking
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
        "matched_role_count": clamp(ranking.matched_role_count, 0, 10) / 10.0,
        "matched_skill_count": clamp(ranking.matched_skill_count, 0, 20) / 20.0,
        "matched_keyword_count": clamp(ranking.matched_keyword_count, 0, 20) / 20.0,
        "source_present": 1.0 if has_text(example.source) else 0.0,
        "role_family_present": 1.0 if has_text(example.role_family) else 0.0,
    }


def train_model(
    datasets: list[OutcomeDataset | dict[str, Any]],
    *,
    epochs: int = 500,
    learning_rate: float = 0.08,
    l2: float = 0.01,
    max_score_delta: int = 8,
) -> TrainedRerankerModel:
    examples: list[OutcomeExample] = []
    policy_versions: set[str] = set()
    for dataset in datasets:
        parsed = (
            dataset
            if isinstance(dataset, OutcomeDataset)
            else OutcomeDataset.model_validate(dataset)
        )
        policy_versions.add(parsed.label_policy_version)
        examples.extend(parsed.examples)

    if not examples:
        raise ValueError(
            "cannot train a reranker without labeled examples. "
            "Export a dataset after creating profile/job outcome signals: "
            "save a job, hide a job, mark a job bad-fit, or create an application."
        )

    safe_epochs = max(1, epochs)
    safe_learning_rate = max(0.0001, learning_rate)
    safe_l2 = max(0.0, l2)
    feature_names = list(DEFAULT_FEATURE_NAMES)
    vectors = [
        [extract_features(example)[feature_name] for feature_name in feature_names]
        for example in examples
    ]
    labels = [example.label_score / 2.0 for example in examples]
    weights = [0.0 for _ in feature_names]
    intercept = smoothed_logit(sum(labels) / len(labels))

    for epoch in range(safe_epochs):
        rate = safe_learning_rate / (1.0 + epoch / 100.0)
        for vector, label in zip(vectors, labels, strict=True):
            prediction = sigmoid(intercept + dot(weights, vector))
            error = prediction - label
            for index, value in enumerate(vector):
                weights[index] -= rate * (error * value + safe_l2 * weights[index])
            intercept -= rate * error

    loss = average_log_loss(vectors, labels, weights, intercept)
    positive_count = sum(1 for example in examples if example.label == "positive")
    medium_count = sum(1 for example in examples if example.label == "medium")
    negative_count = sum(1 for example in examples if example.label == "negative")
    label_policy_version = (
        next(iter(policy_versions)) if len(policy_versions) == 1 else "mixed"
    )
    artifact = TrainedRerankerArtifact(
        label_policy_version=label_policy_version,
        feature_names=feature_names,
        feature_transforms=dict(FEATURE_TRANSFORMS),
        weights={
            feature_name: round(weight, 8)
            for feature_name, weight in zip(feature_names, weights, strict=True)
        },
        intercept=round(intercept, 8),
        max_score_delta=max(1, min(20, max_score_delta)),
        training=TrainingSummary(
            example_count=len(examples),
            positive_count=positive_count,
            medium_count=medium_count,
            negative_count=negative_count,
            epochs=safe_epochs,
            learning_rate=safe_learning_rate,
            l2=safe_l2,
            loss=round(loss, 8),
        ),
    )
    return TrainedRerankerModel(artifact)


def load_dataset(path: str | Path) -> OutcomeDataset:
    with open(path, "r", encoding="utf-8") as handle:
        payload = json.load(handle)
    return OutcomeDataset.model_validate(payload)


def clamp(value: int | float, lower: int | float, upper: int | float) -> int | float:
    return max(lower, min(upper, value))


def has_text(value: str | None) -> bool:
    return value is not None and value.strip() != ""


def sigmoid(value: float) -> float:
    if value >= 0:
        z = math.exp(-value)
        return 1.0 / (1.0 + z)
    z = math.exp(value)
    return z / (1.0 + z)


def smoothed_logit(probability: float) -> float:
    safe_probability = min(0.99, max(0.01, probability))
    return math.log(safe_probability / (1.0 - safe_probability))


def dot(left: list[float], right: list[float]) -> float:
    return sum(
        left_value * right_value
        for left_value, right_value in zip(left, right, strict=True)
    )


def average_log_loss(
    vectors: list[list[float]],
    labels: list[float],
    weights: list[float],
    intercept: float,
) -> float:
    total = 0.0
    for vector, label in zip(vectors, labels, strict=True):
        prediction = min(
            0.999999,
            max(0.000001, sigmoid(intercept + dot(weights, vector))),
        )
        total += -(label * math.log(prediction) + (1.0 - label) * math.log(1.0 - prediction))
    return total / len(labels)


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Train an inspectable logistic-regression reranker from outcome datasets."
    )
    parser.add_argument("dataset_json", nargs="+", help="One or more reranker dataset JSON exports")
    parser.add_argument("--output", required=True, help="Path for the trained JSON model artifact")
    parser.add_argument("--top-n", type=int, default=10)
    parser.add_argument("--epochs", type=int, default=500)
    parser.add_argument("--learning-rate", type=float, default=0.08)
    parser.add_argument("--l2", type=float, default=0.01)
    parser.add_argument("--max-score-delta", type=int, default=8)
    args = parser.parse_args()

    try:
        datasets = [load_dataset(path) for path in args.dataset_json]
        model = train_model(
            datasets,
            epochs=args.epochs,
            learning_rate=args.learning_rate,
            l2=args.l2,
            max_score_delta=args.max_score_delta,
        )
        model.save(args.output)
    except (OSError, ValueError, json.JSONDecodeError) as error:
        print(f"error: {error}", file=sys.stderr)
        raise SystemExit(1) from error

    merged = OutcomeDataset(
        profile_id="multiple" if len(datasets) > 1 else datasets[0].profile_id,
        label_policy_version=model.artifact.label_policy_version,
        examples=[example for dataset in datasets for example in dataset.examples],
    )
    summary = evaluate_dataset(merged, top_n=args.top_n, trained_model=model)
    print(
        json.dumps(
            {
                "model_artifact": args.output,
                "training": model.artifact.training.model_dump(),
                "evaluation": summary.model_dump(),
            },
            indent=2,
        )
    )


if __name__ == "__main__":
    main()
