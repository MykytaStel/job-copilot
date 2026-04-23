import logging
import os

from app.core.enrichment_observability import run_enrichment_call
from app.enrichment.weekly_guidance.contract import WeeklyGuidanceRequest, WeeklyGuidanceResponse
from app.enrichment.weekly_guidance.parser import parse_weekly_guidance_output
from app.enrichment.weekly_guidance.prompt import build_weekly_guidance_prompt
from app.llm_provider_types import WeeklyGuidanceProvider
from app.trained_reranker.model import TrainedRerankerModel
from app.trained_reranker_config import profile_artifact_path

logger = logging.getLogger(__name__)

_TOP_SIGNAL_COUNT = 5

# Module-level cache: keyed by model path for per-profile artifacts.
_signals_cache: dict[str, tuple[float, dict[str, float]]] = {}


def _load_top_model_signals(profile_id: str) -> dict[str, float] | None:
    model_path = profile_artifact_path(profile_id)
    try:
        mtime = os.path.getmtime(model_path)
    except OSError:
        return None

    cache_key = str(model_path)
    cached = _signals_cache.get(cache_key)
    if cached is not None and cached[0] == mtime:
        return cached[1]

    try:
        model = TrainedRerankerModel.load(model_path)
        importances = model.feature_importances()
        top = sorted(importances.items(), key=lambda kv: kv[1], reverse=True)
        signals = dict(top[:_TOP_SIGNAL_COUNT])
        _signals_cache[cache_key] = (mtime, signals)
        return signals
    except Exception:
        logger.debug("no trained reranker model available for weekly guidance signals")
        return None


class WeeklyGuidanceService:
    def __init__(self, provider: WeeklyGuidanceProvider):
        self._provider = provider

    async def enrich(self, context: WeeklyGuidanceRequest) -> WeeklyGuidanceResponse:
        if context.llm_context.top_model_signals is None:
            context.llm_context.top_model_signals = _load_top_model_signals(context.profile_id)
        prompt = build_weekly_guidance_prompt(context)
        return await run_enrichment_call(
            flow="weekly_guidance",
            provider=self._provider,
            context=context,
            prompt=prompt,
            provider_call=lambda: self._provider.generate_weekly_guidance(context, prompt),
            parse_output=parse_weekly_guidance_output,
        )
