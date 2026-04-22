from app.interview_prep import (
    InterviewPrepRequest,
    InterviewPrepResponse,
    build_interview_prep_prompt,
    parse_interview_prep_output,
)
from app.llm_provider_types import InterviewPrepProvider


class InterviewPrepService:
    def __init__(self, provider: InterviewPrepProvider):
        self._provider = provider

    async def enrich(self, context: InterviewPrepRequest) -> InterviewPrepResponse:
        prompt = build_interview_prep_prompt(context)
        raw_output = await self._provider.generate_interview_prep(context, prompt)
        return parse_interview_prep_output(raw_output)
