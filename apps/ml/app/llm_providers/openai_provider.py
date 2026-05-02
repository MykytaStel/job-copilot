import asyncio

from app.enrichment.application_coach.contract import (
    ApplicationCoachPrompt,
    ApplicationCoachProviderError,
    ApplicationCoachRequest,
)
from app.enrichment.cover_letter_draft.contract import (
    CoverLetterDraftPrompt,
    CoverLetterDraftProviderError,
    CoverLetterDraftRequest,
)
from app.enrichment.cv_tailoring.contract import (
    CvTailoringPrompt,
    CvTailoringProviderError,
    CvTailoringRequest,
)
from app.enrichment.interview_prep.contract import (
    InterviewPrepPrompt,
    InterviewPrepProviderError,
    InterviewPrepRequest,
)
from app.enrichment.job_fit_explanation.contract import (
    JobFitExplanationPrompt,
    JobFitExplanationProviderError,
    JobFitExplanationRequest,
)
from app.enrichment.profile_insights.contract import (
    LlmContextRequest,
    ProfileInsightsPrompt,
    ProfileInsightsProviderError,
)
from app.enrichment.weekly_guidance.contract import WeeklyGuidanceRequest
from app.enrichment.weekly_guidance.errors import WeeklyGuidanceProviderError
from app.enrichment.weekly_guidance.prompt import WeeklyGuidancePrompt
from app.llm_providers.common import PromptPayload, build_rate_limit_retrying, build_retrying


class _OpenAIJsonSchemaProvider:
    def __init__(
        self,
        api_key: str,
        model: str,
        base_url: str | None = None,
        timeout_seconds: float | None = None,
    ):
        try:
            from openai import (
                APIConnectionError,
                APITimeoutError,
                InternalServerError,
                OpenAI,
                RateLimitError,
            )
        except ImportError as exc:
            raise ProfileInsightsProviderError(
                "OpenAI provider is configured but the openai package is not installed."
            ) from exc

        self._client = OpenAI(api_key=api_key, base_url=base_url, timeout=timeout_seconds)
        self._model = model
        self._conn_errors = (APIConnectionError, APITimeoutError, InternalServerError)
        self._rate_limit_error = RateLimitError
        self._retryable_errors = (*self._conn_errors, RateLimitError)

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
        rate_limit_retrying = build_rate_limit_retrying(self._rate_limit_error)
        for rate_limit_attempt in rate_limit_retrying:
            with rate_limit_attempt:
                conn_retrying = build_retrying(self._conn_errors)
                for attempt in conn_retrying:
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
        self, context: LlmContextRequest, prompt: ProfileInsightsPrompt
    ) -> str:
        return await self._generate(
            prompt_name="profile_insights",
            failure_label="profile insights",
            error_type=ProfileInsightsProviderError,
            prompt=prompt,
        )

    async def generate_job_fit_explanation(
        self, context: JobFitExplanationRequest, prompt: JobFitExplanationPrompt
    ) -> str:
        return await self._generate(
            prompt_name="job_fit_explanation",
            failure_label="job fit explanation",
            error_type=JobFitExplanationProviderError,
            prompt=prompt,
        )

    async def generate_application_coach(
        self, context: ApplicationCoachRequest, prompt: ApplicationCoachPrompt
    ) -> str:
        return await self._generate(
            prompt_name="application_coach",
            failure_label="application coach",
            error_type=ApplicationCoachProviderError,
            prompt=prompt,
        )

    async def generate_cover_letter_draft(
        self, context: CoverLetterDraftRequest, prompt: CoverLetterDraftPrompt
    ) -> str:
        return await self._generate(
            prompt_name="cover_letter_draft",
            failure_label="cover letter draft",
            error_type=CoverLetterDraftProviderError,
            prompt=prompt,
        )

    async def generate_interview_prep(
        self, context: InterviewPrepRequest, prompt: InterviewPrepPrompt
    ) -> str:
        return await self._generate(
            prompt_name="interview_prep",
            failure_label="interview prep",
            error_type=InterviewPrepProviderError,
            prompt=prompt,
        )

    async def generate_weekly_guidance(
        self, context: WeeklyGuidanceRequest, prompt: WeeklyGuidancePrompt
    ) -> str:
        return await self._generate(
            prompt_name="weekly_guidance",
            failure_label="weekly guidance",
            error_type=WeeklyGuidanceProviderError,
            prompt=prompt,
        )

    async def generate_cv_tailoring(
        self, context: CvTailoringRequest, prompt: CvTailoringPrompt
    ) -> str:
        return await self._generate(
            prompt_name="cv_tailoring",
            failure_label="cv tailoring",
            error_type=CvTailoringProviderError,
            prompt=prompt,
        )
