import asyncio

from app.enrichment.application_coach import ApplicationCoachProviderError, ApplicationCoachRequest
from app.enrichment.cover_letter_draft import CoverLetterDraftProviderError, CoverLetterDraftRequest
from app.enrichment.interview_prep import InterviewPrepProviderError, InterviewPrepRequest
from app.enrichment.job_fit_explanation import JobFitExplanationProviderError, JobFitExplanationRequest
from app.enrichment.profile_insights import (
    LlmContextRequest,
    ProfileInsightsProviderError,
)
from app.enrichment.weekly_guidance import WeeklyGuidanceProviderError, WeeklyGuidanceRequest
from app.llm_providers.common import PromptPayload, build_retrying


class _OpenAIJsonSchemaProvider:
    def __init__(self, api_key: str, model: str, base_url: str | None = None):
        try:
            from openai import APIConnectionError, APITimeoutError, InternalServerError, OpenAI, RateLimitError
        except ImportError as exc:
            raise ProfileInsightsProviderError(
                "OpenAI provider is configured but the openai package is not installed."
            ) from exc

        self._client = OpenAI(api_key=api_key, base_url=base_url)
        self._model = model
        self._retryable_errors = (
            APIConnectionError,
            APITimeoutError,
            InternalServerError,
            RateLimitError,
        )

    async def _generate(
        self,
        *,
        prompt_name: str,
        failure_label: str,
        error_type: type[Exception],
        prompt: PromptPayload,
    ) -> str:
        return await asyncio.to_thread(
            self._generate_sync,
            prompt_name,
            failure_label,
            error_type,
            prompt,
        )

    def _request_with_retry(self, *, prompt_name: str, prompt: PromptPayload):
        retrying = build_retrying(self._retryable_errors)
        for attempt in retrying:
            with attempt:
                return self._client.responses.create(
                    model=self._model,
                    instructions=prompt.system_instructions,
                    input=prompt.context_payload,
                    text={
                        "format": {
                            "type": "json_schema",
                            "name": prompt_name,
                            "strict": True,
                            "schema": prompt.output_schema,
                        }
                    },
                    store=False,
                )

        raise AssertionError("OpenAI retry loop should always return or raise")

    def _generate_sync(
        self,
        prompt_name: str,
        failure_label: str,
        error_type: type[Exception],
        prompt: PromptPayload,
    ) -> str:
        try:
            response = self._request_with_retry(prompt_name=prompt_name, prompt=prompt)
        except self._retryable_errors as exc:
            raise error_type(f"OpenAI {failure_label} request failed: {exc}") from exc
        except Exception as exc:  # pragma: no cover - external SDK failure path
            raise error_type(f"OpenAI {failure_label} request failed: {exc}") from exc

        output_text = getattr(response, "output_text", "")
        if not output_text:
            raise error_type(f"OpenAI {failure_label} request returned an empty response.")

        return output_text


class OpenAIEnrichmentProvider(_OpenAIJsonSchemaProvider):
    async def generate_profile_insights(
        self, context: LlmContextRequest, prompt
    ) -> str:
        return await self._generate(
            prompt_name="profile_insights",
            failure_label="profile insights",
            error_type=ProfileInsightsProviderError,
            prompt=prompt,
        )

    async def generate_job_fit_explanation(
        self, context: JobFitExplanationRequest, prompt
    ) -> str:
        return await self._generate(
            prompt_name="job_fit_explanation",
            failure_label="job fit explanation",
            error_type=JobFitExplanationProviderError,
            prompt=prompt,
        )

    async def generate_application_coach(
        self, context: ApplicationCoachRequest, prompt
    ) -> str:
        return await self._generate(
            prompt_name="application_coach",
            failure_label="application coach",
            error_type=ApplicationCoachProviderError,
            prompt=prompt,
        )

    async def generate_cover_letter_draft(
        self, context: CoverLetterDraftRequest, prompt
    ) -> str:
        return await self._generate(
            prompt_name="cover_letter_draft",
            failure_label="cover letter draft",
            error_type=CoverLetterDraftProviderError,
            prompt=prompt,
        )

    async def generate_interview_prep(
        self, context: InterviewPrepRequest, prompt
    ) -> str:
        return await self._generate(
            prompt_name="interview_prep",
            failure_label="interview prep",
            error_type=InterviewPrepProviderError,
            prompt=prompt,
        )

    async def generate_weekly_guidance(
        self, context: WeeklyGuidanceRequest, prompt
    ) -> str:
        return await self._generate(
            prompt_name="weekly_guidance",
            failure_label="weekly guidance",
            error_type=WeeklyGuidanceProviderError,
            prompt=prompt,
        )
