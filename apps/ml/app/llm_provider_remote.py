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
        self._model = os.getenv("OLLAMA_MODEL", "llama3.1:8b")

    async def _generate(
        self,
        *,
        system_instructions: str,
        context_payload: str,
        output_schema_expectations: str,
        error_type: type[Exception],
    ) -> str:
        import httpx

        system_prompt = (
            f"{system_instructions}\n\n"
            f"You MUST respond with a single JSON object that exactly matches this schema:\n"
            f"{output_schema_expectations}\n\n"
            f"Rules: return only valid JSON, no markdown, no code fences, no explanation text."
        )

        try:
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
                timeout=180.0,
            )
            response.raise_for_status()
            data = response.json()
            content = data.get("message", {}).get("content", "")
            if not content:
                raise error_type("Ollama returned an empty response.")
            return content
        except error_type:
            raise
        except Exception as exc:
            raise error_type(f"Ollama request failed: {exc}") from exc

    async def generate_profile_insights(
        self, context: LlmContextRequest, prompt: ProfileInsightsPrompt
    ) -> str:
        return await self._generate(
            system_instructions=prompt.system_instructions,
            context_payload=prompt.context_payload,
            output_schema_expectations=prompt.output_schema_expectations,
            error_type=ProfileInsightsProviderError,
        )

    async def generate_job_fit_explanation(
        self, context: JobFitExplanationRequest, prompt: JobFitExplanationPrompt
    ) -> str:
        return await self._generate(
            system_instructions=prompt.system_instructions,
            context_payload=prompt.context_payload,
            output_schema_expectations=prompt.output_schema_expectations,
            error_type=JobFitExplanationProviderError,
        )

    async def generate_application_coach(
        self, context: ApplicationCoachRequest, prompt: ApplicationCoachPrompt
    ) -> str:
        return await self._generate(
            system_instructions=prompt.system_instructions,
            context_payload=prompt.context_payload,
            output_schema_expectations=prompt.output_schema_expectations,
            error_type=ApplicationCoachProviderError,
        )

    async def generate_cover_letter_draft(
        self, context: CoverLetterDraftRequest, prompt: CoverLetterDraftPrompt
    ) -> str:
        return await self._generate(
            system_instructions=prompt.system_instructions,
            context_payload=prompt.context_payload,
            output_schema_expectations=prompt.output_schema_expectations,
            error_type=CoverLetterDraftProviderError,
        )

    async def generate_interview_prep(
        self, context: InterviewPrepRequest, prompt: InterviewPrepPrompt
    ) -> str:
        return await self._generate(
            system_instructions=prompt.system_instructions,
            context_payload=prompt.context_payload,
            output_schema_expectations=prompt.output_schema_expectations,
            error_type=InterviewPrepProviderError,
        )

    async def generate_weekly_guidance(
        self, context: WeeklyGuidanceRequest, prompt: WeeklyGuidancePrompt
    ) -> str:
        return await self._generate(
            system_instructions=prompt.system_instructions,
            context_payload=prompt.context_payload,
            output_schema_expectations=prompt.output_schema_expectations,
            error_type=WeeklyGuidanceProviderError,
        )

