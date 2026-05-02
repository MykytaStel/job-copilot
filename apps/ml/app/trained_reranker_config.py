import re
from pathlib import Path

_PROFILE_SEGMENT_RE = re.compile(r"[^A-Za-z0-9._-]+")


def default_global_runtime_model_path() -> Path:
    return Path(__file__).parent.parent / "models" / "trained-reranker-v3.json"


def default_profile_artifacts_dir() -> Path:
    return Path(__file__).parent.parent / "models" / "profiles"


def get_trained_reranker_model_path() -> Path:
    from app.settings import get_runtime_settings

    settings = get_runtime_settings()
    if settings.model_path:
        return Path(settings.model_path)
    return default_global_runtime_model_path()


def get_profile_artifacts_dir() -> Path:
    from app.settings import DEFAULT_ARTIFACTS_DIR, get_runtime_settings

    settings = get_runtime_settings()
    if settings.artifacts_dir:
        return Path(settings.artifacts_dir)
    return DEFAULT_ARTIFACTS_DIR


def profile_artifact_path(profile_id: str) -> Path:
    cleaned = _PROFILE_SEGMENT_RE.sub("_", profile_id.strip()) or "unknown-profile"
    return get_profile_artifacts_dir() / cleaned / "trained-reranker-v3.json"


DEFAULT_TRAINED_RERANKER_MODEL_PATH = default_global_runtime_model_path()
