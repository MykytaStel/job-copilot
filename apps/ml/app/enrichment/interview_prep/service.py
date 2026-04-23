from app.core.enrichment_observability import run_enrichment_call
from app.enrichment.interview_prep.contract import (
    InterviewPrepRequest,
    InterviewPrepResponse,
)
from app.enrichment.interview_prep.parser import parse_interview_prep_output
from app.enrichment.interview_prep.prompt import build_interview_prep_prompt
from app.llm_provider_types import InterviewPrepProvider


class InterviewPrepService:
    def __init__(self, provider: InterviewPrepProvider):
        self._provider = provider

    async def enrich(self, context: InterviewPrepRequest) -> InterviewPrepResponse:
        prompt = build_interview_prep_prompt(context)
        return await run_enrichment_call(
            flow="interview_prep",
            provider=self._provider,
            context=context,
            prompt=prompt,
            provider_call=lambda: self._provider.generate_interview_prep(context, prompt),
            parse_output=parse_interview_prep_output,
        )
