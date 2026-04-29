from __future__ import annotations

import math
from datetime import datetime, timezone
from pathlib import Path

from app.trained_reranker.model import TrainedRerankerModel


def utc_now_iso() -> str:
    return datetime.now(timezone.utc).isoformat().replace("+00:00", "Z")


def load_bucket_distribution(model_path: Path) -> dict[str, float] | None:
    try:
        old_model = TrainedRerankerModel.load(model_path)
        return old_model.artifact.signal_bucket_distribution or None
    except Exception:
        return None


def kl_divergence(old_dist: dict[str, float], new_dist: dict[str, float]) -> float:
    """KL(new || old): divergence of new distribution from old."""
    epsilon = 1e-8
    keys = set(old_dist) | set(new_dist)

    return sum(
        (new_dist.get(key, 0.0) + epsilon)
        * math.log((new_dist.get(key, 0.0) + epsilon) / (old_dist.get(key, 0.0) + epsilon))
        for key in keys
    )


def atomic_save_model(model: TrainedRerankerModel, path: Path) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    tmp_path = path.with_suffix(path.suffix + ".tmp")
    model.save(tmp_path)
    tmp_path.replace(path)
