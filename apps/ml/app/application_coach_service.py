from app.application_coach import (
    ApplicationCoachRequest,
    ApplicationCoachResponse,
    build_application_coach_prompt,
    parse_application_coach_output,
)
from app.llm_provider_types import ApplicationCoachProvider


class ApplicationCoachService:
    def __init__(self, provider: ApplicationCoachProvider):
        self._provider = provider

    async def enrich(self, context: ApplicationCoachRequest) -> ApplicationCoachResponse:
        prompt = build_application_coach_prompt(context)
        raw_output = await self._provider.generate_application_coach(context, prompt)
        return parse_application_coach_output(raw_output)
