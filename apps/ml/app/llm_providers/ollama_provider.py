import httpx

from app.enrichment.application_coach import ApplicationCoachProviderError, ApplicationCoachRequest
from app.enrichment.cover_letter_draft import CoverLetterDraftProviderError, CoverLetterDraftRequest
from app.enrichment.interview_prep import InterviewPrepProviderError, InterviewPrepRequest
from app.enrichment.job_fit_explanation import JobFitExplanationProviderError, JobFitExplanationRequest
from app.enrichment.profile_insights import (
    LlmContextRequest,
    ProfileInsightsProviderError,
)
from app.enrichment.weekly_guidance import WeeklyGuidanceProviderError, WeeklyGuidanceRequest
from app.llm_providers.common import build_async_client, build_async_retrying
from app.settings import DEFAULT_LLM_REQUEST_TIMEOUT_SECONDS


RETRYABLE_OLLAMA_ERRORS = (
    httpx.ConnectError,
    httpx.PoolTimeout,
    httpx.ReadTimeout,
    httpx.RemoteProtocolError,
    httpx.WriteError,
    httpx.WriteTimeout,
)


class OllamaEnrichmentProvider:
    def __init__(
        self,
        *,
        base_url: str,
        model: str,
        timeout_seconds: float = DEFAULT_LLM_REQUEST_TIMEOUT_SECONDS,
    ) -> None:
        self._client = build_async_client(timeout_seconds)
        self._base_url = base_url.rstrip("/")
        self._model = model

    async def aclose(self) -> None:
        await self._client.aclose()

    async def _request_with_retry(self, *, system_prompt: str, context_payload: str):
        async for attempt in build_async_retrying(RETRYABLE_OLLAMA_ERRORS):
            with attempt:
                response = await self._client.post(
                    f"{self._base_url}/api/chat",
                    json={
                        "model": self._model,
                        "format": "json",
                        "stream": False,
                        "options": {"temperature": 0.3},
                        "messages": [
                            {"role": "system", "content": system_prompt},
                            {"role": "user", "content": context_payload},
                        ],
                    },
                )
                response.raise_for_status()
                return response

        raise AssertionError("Ollama retry loop should always return or raise")

    async def _generate(
        self,
        *,
        system_instructions: str,
        context_payload: str,
        output_schema_expectations: str,
        error_type: type[Exception],
    ) -> str:
        system_prompt = (
            f"{system_instructions}\n\n"
            f"You MUST respond with a single JSON object that exactly matches this schema:\n"
            f"{output_schema_expectations}\n\n"
            f"Rules: return only valid JSON, no markdown, no code fences, no explanation text."
        )

        try:
            response = await self._request_with_retry(
                system_prompt=system_prompt,
                context_payload=context_payload,
            )
            data = response.json()
            content = data.get("message", {}).get("content", "")
            if not content:
                raise error_type("Ollama returned an empty response.")
            return content
        except error_type:
            raise
        except RETRYABLE_OLLAMA_ERRORS as exc:
            raise error_type(f"Ollama request failed: {exc}") from exc
        except Exception as exc:
            raise error_type(f"Ollama request failed: {exc}") from exc

    async def generate_profile_insights(
        self, context: LlmContextRequest, prompt
    ) -> str:
        return await self._generate(
            system_instructions=prompt.system_instructions,
            context_payload=prompt.context_payload,
            output_schema_expectations=prompt.output_schema_expectations,
            error_type=ProfileInsightsProviderError,
        )

    async def generate_job_fit_explanation(
        self, context: JobFitExplanationRequest, prompt
    ) -> str:
        return await self._generate(
            system_instructions=prompt.system_instructions,
            context_payload=prompt.context_payload,
            output_schema_expectations=prompt.output_schema_expectations,
            error_type=JobFitExplanationProviderError,
        )

    async def generate_application_coach(
        self, context: ApplicationCoachRequest, prompt
    ) -> str:
        return await self._generate(
            system_instructions=prompt.system_instructions,
            context_payload=prompt.context_payload,
            output_schema_expectations=prompt.output_schema_expectations,
            error_type=ApplicationCoachProviderError,
        )

    async def generate_cover_letter_draft(
        self, context: CoverLetterDraftRequest, prompt
    ) -> str:
        return await self._generate(
            system_instructions=prompt.system_instructions,
            context_payload=prompt.context_payload,
            output_schema_expectations=prompt.output_schema_expectations,
            error_type=CoverLetterDraftProviderError,
        )

    async def generate_interview_prep(
        self, context: InterviewPrepRequest, prompt
    ) -> str:
        return await self._generate(
            system_instructions=prompt.system_instructions,
            context_payload=prompt.context_payload,
            output_schema_expectations=prompt.output_schema_expectations,
            error_type=InterviewPrepProviderError,
        )

    async def generate_weekly_guidance(
        self, context: WeeklyGuidanceRequest, prompt
    ) -> str:
        return await self._generate(
            system_instructions=prompt.system_instructions,
            context_payload=prompt.context_payload,
            output_schema_expectations=prompt.output_schema_expectations,
            error_type=WeeklyGuidanceProviderError,
        )
