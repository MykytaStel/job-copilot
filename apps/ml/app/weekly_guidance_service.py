from app.llm_provider_types import WeeklyGuidanceProvider
from app.weekly_guidance import (
    WeeklyGuidanceRequest,
    WeeklyGuidanceResponse,
    build_weekly_guidance_prompt,
    parse_weekly_guidance_output,
)


class WeeklyGuidanceService:
    def __init__(self, provider: WeeklyGuidanceProvider):
        self._provider = provider

    async def enrich(self, context: WeeklyGuidanceRequest) -> WeeklyGuidanceResponse:
        prompt = build_weekly_guidance_prompt(context)
        raw_output = await self._provider.generate_weekly_guidance(context, prompt)
        return parse_weekly_guidance_output(raw_output)
