from app.core.enrichment_observability import run_enrichment_call
from app.enrichment.profile_insights.contract import (
    LlmContextRequest,
    ProfileInsightsResponse,
)
from app.enrichment.profile_insights.parser import parse_profile_insights_output
from app.enrichment.profile_insights.prompt import build_profile_insights_prompt
from app.llm_provider_types import ProfileInsightsProvider


class ProfileInsightsService:
    def __init__(self, provider: ProfileInsightsProvider):
        self._provider = provider

    async def enrich(self, context: LlmContextRequest) -> ProfileInsightsResponse:
        prompt = build_profile_insights_prompt(context)
        return await run_enrichment_call(
            flow="profile_insights",
            provider=self._provider,
            context=context,
            prompt=prompt,
            provider_call=lambda: self._provider.generate_profile_insights(context, prompt),
            parse_output=parse_profile_insights_output,
        )
