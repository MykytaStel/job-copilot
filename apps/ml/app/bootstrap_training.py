import argparse
import logging
import sys
from pathlib import Path

from app.bootstrap_client import fetch_labeled_examples as _fetch_labeled_examples
from app.bootstrap_contract import BootstrapWorkflowResult
from app.bootstrap_workflow import (
    DEFAULT_MODEL_PATH as _DEFAULT_MODEL_PATH,
    bootstrap_and_retrain as _bootstrap_and_retrain,
)
from app.trained_reranker_config import DEFAULT_TRAINED_RERANKER_MODEL_PATH

logger = logging.getLogger(__name__)

# Compatibility export for existing imports.
DEFAULT_MODEL_PATH = _DEFAULT_MODEL_PATH


async def fetch_labeled_examples(
    profile_id: str,
    base_url: str | None = None,
):
    return await _fetch_labeled_examples(profile_id, base_url=base_url)


async def bootstrap_and_retrain(
    profile_id: str,
    min_examples: int = 30,
    model_path: Path | None = None,
    artifact_path: Path = DEFAULT_MODEL_PATH,
    compatibility_model_path: Path = DEFAULT_TRAINED_RERANKER_MODEL_PATH,
    base_url: str | None = None,
) -> BootstrapWorkflowResult:
    resolved_artifact_path = model_path or artifact_path
    resolved_compatibility_path = model_path or compatibility_model_path
    return await _bootstrap_and_retrain(
        profile_id=profile_id,
        min_examples=min_examples,
        artifact_path=resolved_artifact_path,
        compatibility_model_path=resolved_compatibility_path,
        base_url=base_url,
        fetch_examples=fetch_labeled_examples,
    )


def main() -> None:
    logging.basicConfig(level=logging.INFO, format="%(levelname)s %(message)s")
    parser = argparse.ArgumentParser(
        description="Bootstrap reranker training from real user events via engine-api."
    )
    parser.add_argument("--profile-id", required=True, help="Profile ID to fetch dataset for")
    parser.add_argument("--min-examples", type=int, default=30)
    parser.add_argument("--model-path", default=str(DEFAULT_TRAINED_RERANKER_MODEL_PATH))
    parser.add_argument("--engine-api-url", default=None, help="Override ENGINE_API_BASE_URL")
    args = parser.parse_args()

    import asyncio

    result = asyncio.run(
        bootstrap_and_retrain(
            profile_id=args.profile_id,
            min_examples=args.min_examples,
            model_path=Path(args.model_path),
            base_url=args.engine_api_url,
        )
    )

    import json

    print(json.dumps(result.to_payload(), indent=2))
    if not result.retrained:
        sys.exit(1)


if __name__ == "__main__":
    main()
