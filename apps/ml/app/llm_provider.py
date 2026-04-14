import asyncio
import os
from typing import Any, Protocol

from app.profile_insights import LlmContextRequest, ProfileInsightsPrompt, ProfileInsightsProviderError


class ProfileInsightsProvider(Protocol):
    async def generate_profile_insights(
        self, context: LlmContextRequest, prompt: ProfileInsightsPrompt
    ) -> Any: ...


class TemplateProfileInsightsProvider:
    async def generate_profile_insights(
        self, context: LlmContextRequest, prompt: ProfileInsightsPrompt
    ) -> dict[str, Any]:
        analyzed_profile = context.analyzed_profile
        role = analyzed_profile.primary_role.replace("_", " ") if analyzed_profile else "current target role"
        seniority = analyzed_profile.seniority if analyzed_profile else "current level"
        skills = analyzed_profile.skills if analyzed_profile else context.profile_skills
        keywords = analyzed_profile.keywords if analyzed_profile else context.profile_keywords

        strengths = []
        if analyzed_profile and analyzed_profile.summary:
            strengths.append(f"Clear profile positioning around {role} at {seniority} level.")
        if skills:
            strengths.append(f"Relevant skills are already explicit: {', '.join(skills[:3])}.")
        if context.feedback_summary.saved_jobs_count > 0:
            strengths.append("Saved jobs indicate the current direction already has positive traction.")

        risks = []
        if not skills:
            risks.append("The profile context does not expose enough concrete skills yet.")
        if context.feedback_summary.bad_fit_jobs_count > 0:
            risks.append("Bad-fit feedback suggests the current search may still be too broad.")
        if context.feedback_summary.blacklisted_companies_count > 0:
            risks.append("Company blacklist signals narrow the practical target set.")

        focus_areas = []
        focus_areas.extend(skills[:3])
        focus_areas.extend(keyword for keyword in keywords[:3] if keyword not in focus_areas)

        search_terms = []
        if analyzed_profile:
            search_terms.append(role)
            search_terms.extend(skill for skill in skills[:3] if skill.lower() not in {term.lower() for term in search_terms})
            search_terms.extend(
                keyword
                for keyword in keywords[:3]
                if keyword.lower() not in {term.lower() for term in search_terms}
            )

        return {
            "profile_summary": (
                analyzed_profile.summary
                if analyzed_profile
                else "The profile needs more deterministic analysis before enrichment can be specific."
            ),
            "search_strategy_summary": (
                f"Anchor the search around {role} roles, keep filters aligned with the current deterministic profile, "
                "and use feedback to narrow terms instead of expanding into unrelated directions."
            ),
            "strengths": strengths,
            "risks": risks,
            "recommended_actions": [
                "Prioritize applications where the title and first responsibilities align with the current target role.",
                "Use saved and bad-fit feedback to tighten search terms before expanding volume.",
            ],
            "top_focus_areas": focus_areas,
            "search_term_suggestions": search_terms,
            "application_strategy": [
                "Apply first to roles that match the primary role and strongest listed skills.",
                "Skip listings that conflict with repeated bad-fit or blacklist feedback signals.",
            ],
        }


class OpenAIProfileInsightsProvider:
    def __init__(self, api_key: str, model: str, base_url: str | None = None):
        try:
            from openai import OpenAI
        except ImportError as exc:
            raise ProfileInsightsProviderError(
                "OpenAI provider is configured but the openai package is not installed."
            ) from exc

        self._client = OpenAI(api_key=api_key, base_url=base_url)
        self._model = model

    async def generate_profile_insights(
        self, context: LlmContextRequest, prompt: ProfileInsightsPrompt
    ) -> str:
        return await asyncio.to_thread(self._generate_sync, prompt)

    def _generate_sync(self, prompt: ProfileInsightsPrompt) -> str:
        try:
            response = self._client.responses.create(
                model=self._model,
                instructions=prompt.system_instructions,
                input=prompt.context_payload,
                text={
                    "format": {
                        "type": "json_schema",
                        "name": "profile_insights",
                        "strict": True,
                        "schema": prompt.output_schema,
                    }
                },
                store=False,
            )
        except Exception as exc:  # pragma: no cover - external SDK failure path
            raise ProfileInsightsProviderError(f"OpenAI profile insights request failed: {exc}") from exc

        output_text = getattr(response, "output_text", "")
        if not output_text:
            raise ProfileInsightsProviderError("OpenAI profile insights request returned an empty response.")

        return output_text


def build_profile_insights_provider() -> ProfileInsightsProvider:
    configured = os.getenv("ML_LLM_PROVIDER", "").strip().lower()
    provider_name = configured or ("openai" if os.getenv("OPENAI_API_KEY") else "template")

    if provider_name == "template":
        return TemplateProfileInsightsProvider()

    if provider_name == "openai":
        api_key = os.getenv("OPENAI_API_KEY", "").strip()
        if not api_key:
            raise ProfileInsightsProviderError("OPENAI_API_KEY is required when ML_LLM_PROVIDER=openai.")

        model = os.getenv("OPENAI_MODEL", "gpt-5.4-mini").strip() or "gpt-5.4-mini"
        base_url = os.getenv("OPENAI_BASE_URL", "").strip() or None
        return OpenAIProfileInsightsProvider(api_key=api_key, model=model, base_url=base_url)

    raise ProfileInsightsProviderError(f"Unsupported ML_LLM_PROVIDER: {provider_name}")
