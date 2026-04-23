import math
from datetime import datetime, timezone
from typing import Any

from app.reranker_evaluation import OutcomeDataset, OutcomeExample
from app.reranker_signal_weights import (
    DEFAULT_OUTCOME_CONFIDENCE_WEIGHTS,
    DEFAULT_OUTCOME_SIGNAL_WEIGHTS,
    OutcomeConfidenceWeightConfig,
    OutcomeSignalWeightConfig,
    confidence_weight_for_example,
    resolve_example_signal_bucket,
    training_target_for_example,
)

from .artifact import (
    DEFAULT_FEATURE_NAMES,
    FEATURE_TRANSFORMS,
    TrainingSummary,
    TrainedRerankerArtifact,
)
from .features import extract_features
from .model import TrainedRerankerModel, compute_feature_importances, sigmoid


def train_model(
    datasets: list[OutcomeDataset | dict[str, Any]],
    *,
    epochs: int = 500,
    learning_rate: float = 0.08,
    l2: float = 0.01,
    max_score_delta: int = 8,
    signal_weights: OutcomeSignalWeightConfig = DEFAULT_OUTCOME_SIGNAL_WEIGHTS,
    confidence_weights: OutcomeConfidenceWeightConfig = DEFAULT_OUTCOME_CONFIDENCE_WEIGHTS,
    temporal_decay_lambda: float = 0.01,
    distilled_labels: list[float] | None = None,
    feature_names: list[str] | None = None,
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
    feature_names = list(feature_names or DEFAULT_FEATURE_NAMES)
    vectors = [
        [extract_features(example)[feature_name] for feature_name in feature_names]
        for example in examples
    ]
    if distilled_labels is not None and len(distilled_labels) == len(examples):
        labels = [max(0.0, min(1.0, v)) for v in distilled_labels]
    else:
        labels = [training_target_for_example(example, signal_weights) for example in examples]
    sample_weights = build_sample_weights(
        examples,
        confidence_weights=confidence_weights,
        temporal_decay_lambda=temporal_decay_lambda,
    )
    weights = [0.0 for _ in feature_names]
    intercept = smoothed_logit(sum(labels) / len(labels))

    for epoch in range(safe_epochs):
        rate = safe_learning_rate / (1.0 + epoch / 100.0)
        for vector, label, sample_weight in zip(vectors, labels, sample_weights, strict=True):
            prediction = sigmoid(intercept + dot(weights, vector))
            error = (prediction - label) * sample_weight
            for index, value in enumerate(vector):
                weights[index] -= rate * (error * value + safe_l2 * weights[index])
            intercept -= rate * error

    rounded_weights = {
        feature_name: round(weight, 8)
        for feature_name, weight in zip(feature_names, weights, strict=True)
    }
    loss = average_log_loss(vectors, labels, sample_weights, weights, intercept)
    positive_count = sum(1 for example in examples if example.label == "positive")
    medium_count = sum(1 for example in examples if example.label == "medium")
    negative_count = sum(1 for example in examples if example.label == "negative")
    bucket_counts = {
        "saved_only": 0,
        "viewed_only": 0,
        "medium_default": 0,
    }
    for example in examples:
        bucket = resolve_example_signal_bucket(example)
        if bucket in bucket_counts:
            bucket_counts[bucket] += 1
    label_policy_version = (
        next(iter(policy_versions)) if len(policy_versions) == 1 else "mixed"
    )
    total_examples = len(examples)
    bucket_distribution = {
        bucket: round(count / total_examples, 6) if total_examples > 0 else 0.0
        for bucket, count in bucket_counts.items()
    }
    artifact = TrainedRerankerArtifact(
        label_policy_version=label_policy_version,
        signal_weight_policy_version=signal_weights.policy_version,
        signal_weights=signal_weights.as_dict(),
        confidence_weight_policy_version=confidence_weights.policy_version,
        confidence_weights=confidence_weights.as_dict(),
        temporal_decay_lambda=max(0.0, temporal_decay_lambda),
        feature_names=feature_names,
        feature_transforms=dict(FEATURE_TRANSFORMS),
        weights=rounded_weights,
        feature_importances=compute_feature_importances(rounded_weights, feature_names),
        signal_bucket_distribution=bucket_distribution,
        lgbm_distilled=distilled_labels is not None and len(distilled_labels) == len(examples),
        intercept=round(intercept, 8),
        max_score_delta=max(1, min(20, max_score_delta)),
        training=TrainingSummary(
            example_count=len(examples),
            positive_count=positive_count,
            medium_count=medium_count,
            negative_count=negative_count,
            saved_only_count=bucket_counts["saved_only"],
            viewed_only_count=bucket_counts["viewed_only"],
            medium_default_count=bucket_counts["medium_default"],
            epochs=safe_epochs,
            learning_rate=safe_learning_rate,
            l2=safe_l2,
            loss=round(loss, 8),
        ),
    )
    return TrainedRerankerModel(artifact)


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
    sample_weights: list[float],
    weights: list[float],
    intercept: float,
) -> float:
    total = 0.0
    weight_total = 0.0
    for vector, label, sample_weight in zip(vectors, labels, sample_weights, strict=True):
        prediction = min(
            0.999999,
            max(0.000001, sigmoid(intercept + dot(weights, vector))),
        )
        total += sample_weight * (
            -(label * math.log(prediction) + (1.0 - label) * math.log(1.0 - prediction))
        )
        weight_total += sample_weight
    if weight_total <= 0:
        return 0.0
    return total / weight_total


def build_sample_weights(
    examples: list[OutcomeExample],
    *,
    confidence_weights: OutcomeConfidenceWeightConfig,
    temporal_decay_lambda: float,
) -> list[float]:
    now_ts = reference_now_timestamp(examples)
    return [
        confidence_weight_for_example(example, confidence_weights)
        * temporal_example_weight(example, now_ts, temporal_decay_lambda)
        for example in examples
    ]


def temporal_example_weight(
    example: OutcomeExample,
    now_ts: datetime,
    temporal_decay_lambda: float,
) -> float:
    if temporal_decay_lambda <= 0:
        return 1.0

    observed_at = parse_timestamp(example.label_observed_at)
    if observed_at is None:
        return 1.0

    days_ago = max(0.0, (now_ts - observed_at).total_seconds() / 86400.0)
    return math.exp(-temporal_decay_lambda * days_ago)


def reference_now_timestamp(examples: list[OutcomeExample]) -> datetime:
    observed_timestamps = [
        parsed
        for parsed in (parse_timestamp(example.label_observed_at) for example in examples)
        if parsed is not None
    ]
    if observed_timestamps:
        return max(observed_timestamps)
    return datetime.now(timezone.utc)


def parse_timestamp(value: str | None) -> datetime | None:
    if not value:
        return None

    normalized = value.strip()
    if not normalized:
        return None
    if normalized.endswith("Z"):
        normalized = normalized[:-1] + "+00:00"

    try:
        parsed = datetime.fromisoformat(normalized)
    except ValueError:
        return None

    if parsed.tzinfo is None:
        return parsed.replace(tzinfo=timezone.utc)
    return parsed.astimezone(timezone.utc)
