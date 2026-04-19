import asyncio
import os

from app.application_coach import (
    ApplicationCoachPrompt,
    ApplicationCoachProviderError,
    ApplicationCoachRequest,
)
from app.cover_letter_draft import (
    CoverLetterDraftPrompt,
    CoverLetterDraftProviderError,
    CoverLetterDraftRequest,
)
from app.interview_prep import (
    InterviewPrepPrompt,
    InterviewPrepProviderError,
    InterviewPrepRequest,
)
from app.job_fit_explanation import (
    JobFitExplanationPrompt,
    JobFitExplanationProviderError,
    JobFitExplanationRequest,
)
from app.profile_insights import (
    LlmContextRequest,
    ProfileInsightsPrompt,
    ProfileInsightsProviderError,
)
from app.weekly_guidance import (
    WeeklyGuidancePrompt,
    WeeklyGuidanceProviderError,
    WeeklyGuidanceRequest,
)


class _OpenAIJsonSchemaProvider:
    def __init__(self, api_key: str, model: str, base_url: str | None = None):
        try:
            from openai import OpenAI
        except ImportError as exc:
            raise ProfileInsightsProviderError(
                "OpenAI provider is configured but the openai package is not installed."
            ) from exc

        self._client = OpenAI(api_key=api_key, base_url=base_url)
        self._model = model

    async def _generate(
        self,
        *,
        prompt_name: str,
        failure_label: str,
        error_type: type[Exception],
        prompt: (
            ProfileInsightsPrompt
            | JobFitExplanationPrompt
            | ApplicationCoachPrompt
            | CoverLetterDraftPrompt
            | InterviewPrepPrompt
            | WeeklyGuidancePrompt
        ),
    ) -> str:
        return await asyncio.to_thread(
            self._generate_sync,
            prompt_name,
            failure_label,
            error_type,
            prompt,
        )

    def _generate_sync(
        self,
        prompt_name: str,
        failure_label: str,
        error_type: type[Exception],
        prompt: (
            ProfileInsightsPrompt
            | JobFitExplanationPrompt
            | ApplicationCoachPrompt
            | CoverLetterDraftPrompt
            | InterviewPrepPrompt
            | WeeklyGuidancePrompt
        ),
    ) -> str:
        try:
            response = self._client.responses.create(
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


class OllamaEnrichmentProvider:
    def __init__(self) -> None:
        import httpx

        self._client = httpx.AsyncClient()
        self._base_url = os.getenv("OLLAMA_BASE_URL", "http://localhost:11434").rstrip("/")
        self._model = os.getenv("OLLAMA_MODEL", "mistral:7b")

    async def _generate(self, prompt: str, error_type: type[Exception]) -> str:
        import httpx

        try:
            response = await self._client.post(
                f"{self._base_url}/api/generate",
                json={"model": self._model, "prompt": prompt, "stream": False},
                timeout=120.0,
            )
            response.raise_for_status()
            return response.json()["response"]
        except Exception as exc:
            raise error_type(f"Ollama request failed: {exc}") from exc

    async def generate_profile_insights(
        self, context: LlmContextRequest, prompt: ProfileInsightsPrompt
    ) -> str:
        return await self._generate(prompt.context_payload, ProfileInsightsProviderError)

    async def generate_job_fit_explanation(
        self, context: JobFitExplanationRequest, prompt: JobFitExplanationPrompt
    ) -> str:
        return await self._generate(prompt.context_payload, JobFitExplanationProviderError)

    async def generate_application_coach(
        self, context: ApplicationCoachRequest, prompt: ApplicationCoachPrompt
    ) -> str:
        return await self._generate(prompt.context_payload, ApplicationCoachProviderError)

    async def generate_cover_letter_draft(
        self, context: CoverLetterDraftRequest, prompt: CoverLetterDraftPrompt
    ) -> str:
        return await self._generate(prompt.context_payload, CoverLetterDraftProviderError)

    async def generate_interview_prep(
        self, context: InterviewPrepRequest, prompt: InterviewPrepPrompt
    ) -> str:
        return await self._generate(prompt.context_payload, InterviewPrepProviderError)

    async def generate_weekly_guidance(
        self, context: WeeklyGuidanceRequest, prompt: WeeklyGuidancePrompt
    ) -> str:
        return await self._generate(prompt.context_payload, WeeklyGuidanceProviderError)

