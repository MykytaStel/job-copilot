from app.job_fit_explanation import (
    JobFitExplanationRequest,
    JobFitExplanationResponse,
    build_job_fit_explanation_prompt,
    parse_job_fit_explanation_output,
)
from app.llm_provider import JobFitExplanationProvider


class JobFitExplanationService:
    def __init__(self, provider: JobFitExplanationProvider):
        self._provider = provider

    async def enrich(self, context: JobFitExplanationRequest) -> JobFitExplanationResponse:
        prompt = build_job_fit_explanation_prompt(context)
        raw_output = await self._provider.generate_job_fit_explanation(context, prompt)
        return parse_job_fit_explanation_output(raw_output)
