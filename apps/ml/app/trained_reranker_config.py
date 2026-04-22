from pathlib import Path


def default_trained_reranker_model_path() -> Path:
    return Path(__file__).parent.parent / "models" / "trained-reranker-v3.json"


DEFAULT_TRAINED_RERANKER_MODEL_PATH = default_trained_reranker_model_path()
