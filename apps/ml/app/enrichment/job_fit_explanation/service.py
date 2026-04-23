from app.core.enrichment_observability import run_enrichment_call
from app.enrichment.job_fit_explanation.contract import (
    JobFitExplanationRequest,
    JobFitExplanationResponse,
)
from app.enrichment.job_fit_explanation.parser import parse_job_fit_explanation_output
from app.enrichment.job_fit_explanation.prompt import build_job_fit_explanation_prompt
from app.llm_provider_types import JobFitExplanationProvider


class JobFitExplanationService:
    def __init__(self, provider: JobFitExplanationProvider):
        self._provider = provider

    async def enrich(self, context: JobFitExplanationRequest) -> JobFitExplanationResponse:
        prompt = build_job_fit_explanation_prompt(context)
        return await run_enrichment_call(
            flow="job_fit_explanation",
            provider=self._provider,
            context=context,
            prompt=prompt,
            provider_call=lambda: self._provider.generate_job_fit_explanation(context, prompt),
            parse_output=parse_job_fit_explanation_output,
        )
