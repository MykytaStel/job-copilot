from __future__ import annotations

from pathlib import Path
from typing import Any

from app.settings import RERANKER_PROMOTION_MIN_ACCURACY, RERANKER_PROMOTION_MIN_EXAMPLES
from app.trained_reranker import ARTIFACT_VERSION
from app.trained_reranker.model import TrainedRerankerModel
from app.trained_reranker_config import get_trained_reranker_model_path


def reranker_status(model_path: Path | None = None) -> dict[str, Any]:
    resolved_path = model_path or get_trained_reranker_model_path()

    if not resolved_path.exists():
        return {
            "model_version": ARTIFACT_VERSION,
            "trained_at": None,
            "example_count": 0,
            "accuracy": None,
            "is_functional": False,
        }

    try:
        model = TrainedRerankerModel.load(resolved_path)
    except Exception:
        return {
            "model_version": ARTIFACT_VERSION,
            "trained_at": None,
            "example_count": 0,
            "accuracy": None,
            "is_functional": False,
        }

    training = model.artifact.training
    example_count = int(getattr(training, "example_count", 0) or 0)
    accuracy = _accuracy_from_training(training)
    trained_at = getattr(training, "trained_at", None)

    return {
        "model_version": getattr(model.artifact, "artifact_version", ARTIFACT_VERSION),
        "trained_at": trained_at,
        "example_count": example_count,
        "accuracy": accuracy,
        "is_functional": example_count >= RERANKER_PROMOTION_MIN_EXAMPLES
        and accuracy is not None
        and accuracy >= RERANKER_PROMOTION_MIN_ACCURACY,
    }


def _accuracy_from_training(training: Any) -> float | None:
    for field in ("validation_accuracy", "accuracy", "positive_hit_rate"):
        value = getattr(training, field, None)

        if isinstance(value, int | float):
            return float(value)

    return None
