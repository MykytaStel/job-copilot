from app.llm_provider_types import WeeklyGuidanceProvider
from app.weekly_guidance_flow.contract import WeeklyGuidanceRequest, WeeklyGuidanceResponse
from app.weekly_guidance_flow.parser import parse_weekly_guidance_output
from app.weekly_guidance_flow.prompt import build_weekly_guidance_prompt


class WeeklyGuidanceService:
    def __init__(self, provider: WeeklyGuidanceProvider):
        self._provider = provider

    async def enrich(self, context: WeeklyGuidanceRequest) -> WeeklyGuidanceResponse:
        prompt = build_weekly_guidance_prompt(context)
        raw_output = await self._provider.generate_weekly_guidance(context, prompt)
        return parse_weekly_guidance_output(raw_output)
