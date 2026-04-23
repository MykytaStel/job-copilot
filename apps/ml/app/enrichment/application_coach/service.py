from app.core.enrichment_observability import run_enrichment_call
from app.enrichment.application_coach.contract import (
    ApplicationCoachRequest,
    ApplicationCoachResponse,
)
from app.enrichment.application_coach.parser import parse_application_coach_output
from app.enrichment.application_coach.prompt import build_application_coach_prompt
from app.llm_provider_types import ApplicationCoachProvider


class ApplicationCoachService:
    def __init__(self, provider: ApplicationCoachProvider):
        self._provider = provider

    async def enrich(self, context: ApplicationCoachRequest) -> ApplicationCoachResponse:
        prompt = build_application_coach_prompt(context)
        return await run_enrichment_call(
            flow="application_coach",
            provider=self._provider,
            context=context,
            prompt=prompt,
            provider_call=lambda: self._provider.generate_application_coach(context, prompt),
            parse_output=parse_application_coach_output,
        )
