from app.llm_provider_types import ProfileInsightsProvider
from app.profile_insights import (
    LlmContextRequest,
    ProfileInsightsResponse,
    build_profile_insights_prompt,
    parse_profile_insights_output,
)


class ProfileInsightsService:
    def __init__(self, provider: ProfileInsightsProvider):
        self._provider = provider

    async def enrich(self, context: LlmContextRequest) -> ProfileInsightsResponse:
        prompt = build_profile_insights_prompt(context)
        raw_output = await self._provider.generate_profile_insights(context, prompt)
        return parse_profile_insights_output(raw_output)
