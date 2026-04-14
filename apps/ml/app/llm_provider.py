import asyncio
import os
from typing import Any, Protocol

from app.job_fit_explanation import (
    JobFitExplanationPrompt,
    JobFitExplanationProviderError,
    JobFitExplanationRequest,
)
from app.profile_insights import LlmContextRequest, ProfileInsightsPrompt, ProfileInsightsProviderError


class ProfileInsightsProvider(Protocol):
    async def generate_profile_insights(
        self, context: LlmContextRequest, prompt: ProfileInsightsPrompt
    ) -> Any: ...


class JobFitExplanationProvider(Protocol):
    async def generate_job_fit_explanation(
        self, context: JobFitExplanationRequest, prompt: JobFitExplanationPrompt
    ) -> Any: ...


class TemplateEnrichmentProvider:
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

    async def generate_job_fit_explanation(
        self, context: JobFitExplanationRequest, prompt: JobFitExplanationPrompt
    ) -> dict[str, Any]:
        analyzed_profile = context.analyzed_profile
        search_profile = context.search_profile
        job = context.ranked_job
        fit = context.deterministic_fit
        feedback_state = context.feedback_state

        fit_label = "good fit" if fit.score >= 70 else "risky fit" if fit.score < 40 else "mixed fit"
        role_anchor = (
            analyzed_profile.primary_role.replace("_", " ")
            if analyzed_profile
            else search_profile.primary_role.replace("_", " ")
            if search_profile
            else "current target role"
        )

        why_it_matches: list[str] = []
        if fit.matched_roles:
            why_it_matches.append(
                f"Deterministic role overlap exists with {', '.join(fit.matched_roles[:3])}."
            )
        if fit.matched_skills:
            why_it_matches.append(
                f"Matched skills include {', '.join(fit.matched_skills[:4])}."
            )
        if fit.reasons:
            why_it_matches.append(f"Deterministic evidence already highlights: {fit.reasons[0]}")

        risks: list[str] = []
        if fit.work_mode_match is False:
            risks.append("The job work mode conflicts with the current search profile.")
        if fit.region_match is False:
            risks.append("Region signals do not clearly align with the target search regions.")
        if feedback_state and feedback_state.current_job_feedback:
            if feedback_state.current_job_feedback.bad_fit:
                risks.append("This job is already marked as bad fit in feedback state.")
            if feedback_state.current_job_feedback.company_status == "blacklist":
                risks.append("The company is currently blacklisted, which makes the job operationally risky.")
        if fit.score < 40:
            risks.append("The deterministic fit score is low, so the match should be treated cautiously.")

        missing_signals: list[str] = []
        if not fit.matched_skills:
            missing_signals.append("The deterministic fit does not show direct matched skills yet.")
        if not fit.matched_keywords:
            missing_signals.append("Keyword overlap is limited, so narrative alignment is not strongly evidenced.")
        if not job.summary:
            missing_signals.append("The job presentation summary is missing, so the role narrative is less specific.")
        if not analyzed_profile:
            missing_signals.append("Analyzed profile context is limited, so explanation depth is constrained.")

        recommended_next_step = (
            f"Open the source posting and compare the first responsibilities against your {role_anchor} evidence before applying."
        )
        if fit.score >= 70 and fit.matched_skills:
            recommended_next_step = (
                f"Tailor the opening CV bullets to {', '.join(fit.matched_skills[:3])} and apply while the deterministic fit is strong."
            )

        application_angle_parts: list[str] = []
        if analyzed_profile and analyzed_profile.summary:
            application_angle_parts.append(analyzed_profile.summary)
        if fit.matched_skills:
            application_angle_parts.append(
                f"Lead with evidence around {', '.join(fit.matched_skills[:3])}."
            )
        elif fit.matched_roles:
            application_angle_parts.append(
                f"Frame the application around overlap with {', '.join(fit.matched_roles[:2])}."
            )

        return {
            "fit_summary": (
                f"This ranked job looks like a {fit_label} for {role_anchor} based on deterministic score {fit.score} and the available profile-job overlap."
            ),
            "why_it_matches": why_it_matches,
            "risks": risks,
            "missing_signals": missing_signals,
            "recommended_next_step": recommended_next_step,
            "application_angle": " ".join(application_angle_parts).strip(),
        }


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
        prompt: ProfileInsightsPrompt | JobFitExplanationPrompt,
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
        prompt: ProfileInsightsPrompt | JobFitExplanationPrompt,
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


def _build_enrichment_provider() -> TemplateEnrichmentProvider | OpenAIEnrichmentProvider:
    configured = os.getenv("ML_LLM_PROVIDER", "").strip().lower()
    provider_name = configured or ("openai" if os.getenv("OPENAI_API_KEY") else "template")

    if provider_name == "template":
        return TemplateEnrichmentProvider()

    if provider_name == "openai":
        api_key = os.getenv("OPENAI_API_KEY", "").strip()
        if not api_key:
            raise ProfileInsightsProviderError("OPENAI_API_KEY is required when ML_LLM_PROVIDER=openai.")

        model = os.getenv("OPENAI_MODEL", "gpt-5.4-mini").strip() or "gpt-5.4-mini"
        base_url = os.getenv("OPENAI_BASE_URL", "").strip() or None
        return OpenAIEnrichmentProvider(api_key=api_key, model=model, base_url=base_url)

    raise ProfileInsightsProviderError(f"Unsupported ML_LLM_PROVIDER: {provider_name}")


def build_profile_insights_provider() -> ProfileInsightsProvider:
    provider = _build_enrichment_provider()
    return provider


def build_job_fit_explanation_provider() -> JobFitExplanationProvider:
    try:
        provider = _build_enrichment_provider()
    except ProfileInsightsProviderError as exc:
        raise JobFitExplanationProviderError(str(exc)) from exc
    return provider
