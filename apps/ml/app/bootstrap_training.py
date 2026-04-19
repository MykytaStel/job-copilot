import argparse
import logging
import os
import sys
from pathlib import Path
from typing import Any

import httpx

from app.engine_api_client import engine_api_base_url, engine_api_timeout_seconds
from app.reranker_evaluation import OutcomeDataset
from app.trained_reranker import TrainedRerankerModel, train_model

logger = logging.getLogger(__name__)

DEFAULT_MODEL_PATH = Path(__file__).parent.parent / "models" / "trained-reranker-v2.json"


async def fetch_labeled_examples(
    profile_id: str,
    base_url: str | None = None,
) -> OutcomeDataset:
    url_base = (base_url or engine_api_base_url()).rstrip("/")
    timeout = httpx.Timeout(engine_api_timeout_seconds())
    async with httpx.AsyncClient(timeout=timeout) as client:
        response = await client.get(f"{url_base}/api/v1/profiles/{profile_id}/reranker-dataset")
    response.raise_for_status()
    payload: dict[str, Any] = response.json()
    return OutcomeDataset.model_validate(payload)


async def bootstrap_and_retrain(
    profile_id: str,
    min_examples: int = 30,
    model_path: Path = DEFAULT_MODEL_PATH,
    base_url: str | None = None,
) -> dict[str, Any]:
    dataset = await fetch_labeled_examples(profile_id, base_url=base_url)
    example_count = len(dataset.examples)

    if example_count < min_examples:
        logger.warning(
            "not enough labeled examples to retrain: got %d, need %d (profile=%s)",
            example_count,
            min_examples,
            profile_id,
        )
        return {
            "retrained": False,
            "example_count": example_count,
            "min_examples": min_examples,
            "reason": f"need at least {min_examples} examples, got {example_count}",
        }

    model = train_model([dataset])
    model.save(model_path)
    logger.info(
        "retrained reranker: %d examples, loss=%.6f, saved to %s",
        example_count,
        model.artifact.training.loss,
        model_path,
    )
    return {
        "retrained": True,
        "example_count": example_count,
        "model_path": str(model_path),
        "training": model.artifact.training.model_dump(),
    }


def main() -> None:
    logging.basicConfig(level=logging.INFO, format="%(levelname)s %(message)s")
    parser = argparse.ArgumentParser(
        description="Bootstrap reranker training from real user events via engine-api."
    )
    parser.add_argument("--profile-id", required=True, help="Profile ID to fetch dataset for")
    parser.add_argument("--min-examples", type=int, default=30)
    parser.add_argument("--model-path", default=str(DEFAULT_MODEL_PATH))
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

    print(json.dumps(result, indent=2))
    if not result["retrained"]:
        sys.exit(1)


if __name__ == "__main__":
    main()
