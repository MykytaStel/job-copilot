from collections import defaultdict
from typing import Any

from app.reranker_evaluation import OutcomeDataset, OutcomeExample

from .artifact import (
    DEFAULT_FEATURE_NAMES,
    FEATURE_TRANSFORMS,
    TrainingSummary,
    TrainedRerankerArtifact,
)
from .features import extract_features
from .model import TrainedRerankerModel, compute_feature_importances, sigmoid


def bpr_candidate_available(datasets: list[OutcomeDataset | dict[str, Any]]) -> bool:
    examples = _load_examples(datasets)
    positive_count = sum(1 for example in examples if _is_pairwise_positive(example))
    negative_count = sum(1 for example in examples if example.label == "negative")
    return positive_count >= 20 and negative_count >= 20


def train_bpr_model(
    datasets: list[OutcomeDataset | dict[str, Any]],
    *,
    epochs: int = 300,
    learning_rate: float = 0.05,
    l2: float = 0.01,
    max_score_delta: int = 8,
) -> TrainedRerankerModel:
    examples = _load_examples(datasets)
    if not examples:
        raise ValueError("cannot train a BPR reranker without labeled examples")

    positives = [example for example in examples if _is_pairwise_positive(example)]
    negatives = [example for example in examples if example.label == "negative"]
    if len(positives) < 20 or len(negatives) < 20:
        raise ValueError("BPR requires at least 20 positive and 20 negative examples")

    feature_names = list(DEFAULT_FEATURE_NAMES)
    weights = [0.0 for _ in feature_names]
    pairs = _pair_examples(examples)

    if not pairs:
        raise ValueError("BPR requires positive/negative pairs per profile")

    safe_epochs = max(1, epochs)
    safe_learning_rate = max(0.0001, learning_rate)
    safe_l2 = max(0.0, l2)

    for epoch in range(safe_epochs):
        rate = safe_learning_rate / (1.0 + epoch / 100.0)
        for positive, negative in pairs:
            pos_vector = _feature_vector(positive, feature_names)
            neg_vector = _feature_vector(negative, feature_names)
            diff = [left - right for left, right in zip(pos_vector, neg_vector, strict=True)]
            score_diff = sum(weight * value for weight, value in zip(weights, diff, strict=True))
            prediction = sigmoid(score_diff)
            error = prediction - 1.0

            for index, value in enumerate(diff):
                weights[index] -= rate * (error * value + safe_l2 * weights[index])

    rounded_weights = {
        feature_name: round(weight, 8)
        for feature_name, weight in zip(feature_names, weights, strict=True)
    }
    loss = average_bpr_loss(pairs, weights, feature_names)
    positive_count = sum(1 for example in examples if example.label == "positive")
    medium_count = sum(1 for example in examples if example.label == "medium")
    negative_count = sum(1 for example in examples if example.label == "negative")
    policy_versions = {
        (
            dataset.label_policy_version
            if isinstance(dataset, OutcomeDataset)
            else OutcomeDataset.model_validate(dataset).label_policy_version
        )
        for dataset in datasets
    }
    label_policy_version = (
        next(iter(policy_versions)) if len(policy_versions) == 1 else "mixed"
    )

    artifact = TrainedRerankerArtifact(
        model_type="bpr",
        label_policy_version=label_policy_version,
        feature_names=feature_names,
        feature_transforms=dict(FEATURE_TRANSFORMS),
        weights=rounded_weights,
        feature_importances=compute_feature_importances(rounded_weights, feature_names),
        intercept=0.0,
        max_score_delta=max(1, min(20, max_score_delta)),
        training=TrainingSummary(
            example_count=len(examples),
            positive_count=positive_count,
            medium_count=medium_count,
            negative_count=negative_count,
            saved_only_count=0,
            viewed_only_count=0,
            medium_default_count=0,
            epochs=safe_epochs,
            learning_rate=safe_learning_rate,
            l2=safe_l2,
            loss=round(loss, 8),
        ),
    )
    return TrainedRerankerModel(artifact)


def average_bpr_loss(
    pairs: list[tuple[OutcomeExample, OutcomeExample]],
    weights: list[float],
    feature_names: list[str],
) -> float:
    if not pairs:
        return 0.0

    total = 0.0
    for positive, negative in pairs:
        pos_vector = _feature_vector(positive, feature_names)
        neg_vector = _feature_vector(negative, feature_names)
        diff = [left - right for left, right in zip(pos_vector, neg_vector, strict=True)]
        score_diff = sum(weight * value for weight, value in zip(weights, diff, strict=True))
        prediction = min(0.999999, max(0.000001, sigmoid(score_diff)))
        total += -math_log(prediction)
    return total / len(pairs)


def _load_examples(datasets: list[OutcomeDataset | dict[str, Any]]) -> list[OutcomeExample]:
    examples: list[OutcomeExample] = []
    for dataset in datasets:
        parsed = dataset if isinstance(dataset, OutcomeDataset) else OutcomeDataset.model_validate(dataset)
        examples.extend(parsed.examples)
    return examples


def _pair_examples(examples: list[OutcomeExample]) -> list[tuple[OutcomeExample, OutcomeExample]]:
    examples_by_profile = defaultdict(list)
    for example in examples:
        examples_by_profile[example.profile_id or "default"].append(example)

    pairs: list[tuple[OutcomeExample, OutcomeExample]] = []
    for profile_examples in examples_by_profile.values():
        positives = [example for example in profile_examples if _is_pairwise_positive(example)]
        negatives = [example for example in profile_examples if example.label == "negative"]
        for positive in positives:
            for negative in negatives[:5]:
                pairs.append((positive, negative))
    return pairs


def _is_pairwise_positive(example: OutcomeExample) -> bool:
    if example.label == "positive":
        return True
    return example.label == "medium" and bool(getattr(example.signals, "saved", False))


def _feature_vector(example: OutcomeExample, feature_names: list[str]) -> list[float]:
    features = extract_features(example)
    return [features[feature_name] for feature_name in feature_names]


def math_log(value: float) -> float:
    import math

    return math.log(value)
