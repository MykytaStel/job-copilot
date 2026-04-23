from app.engine_api_client import engine_api_client_context
from app.reranker_evaluation import OutcomeDataset


async def fetch_labeled_examples(
    profile_id: str,
    base_url: str | None = None,
) -> OutcomeDataset:
    async with engine_api_client_context(base_url=base_url) as engine_api:
        return await engine_api.fetch_reranker_dataset(profile_id)
