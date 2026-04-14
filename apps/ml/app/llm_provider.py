import asyncio
import os
from typing import Any, Protocol

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
from app.profile_insights import LlmContextRequest, ProfileInsightsPrompt, ProfileInsightsProviderError


class ProfileInsightsProvider(Protocol):
    async def generate_profile_insights(
        self, context: LlmContextRequest, prompt: ProfileInsightsPrompt
    ) -> Any: ...


class JobFitExplanationProvider(Protocol):
    async def generate_job_fit_explanation(
        self, context: JobFitExplanationRequest, prompt: JobFitExplanationPrompt
    ) -> Any: ...


class ApplicationCoachProvider(Protocol):
    async def generate_application_coach(
        self, context: ApplicationCoachRequest, prompt: ApplicationCoachPrompt
    ) -> Any: ...


class CoverLetterDraftProvider(Protocol):
    async def generate_cover_letter_draft(
        self, context: CoverLetterDraftRequest, prompt: CoverLetterDraftPrompt
    ) -> Any: ...


class InterviewPrepProvider(Protocol):
    async def generate_interview_prep(
        self, context: InterviewPrepRequest, prompt: InterviewPrepPrompt
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

    async def generate_application_coach(
        self, context: ApplicationCoachRequest, prompt: ApplicationCoachPrompt
    ) -> dict[str, Any]:
        analyzed_profile = context.analyzed_profile
        search_profile = context.search_profile
        job = context.ranked_job
        fit = context.deterministic_fit
        explanation = context.job_fit_explanation
        feedback_state = context.feedback_state

        role_anchor = (
            analyzed_profile.primary_role.replace("_", " ")
            if analyzed_profile
            else search_profile.primary_role.replace("_", " ")
            if search_profile
            else "current target role"
        )

        resume_focus_points: list[str] = []
        if fit.matched_roles:
            resume_focus_points.append(
                f"Keep the resume anchored to {', '.join(fit.matched_roles[:2])} language."
            )
        if fit.matched_skills:
            resume_focus_points.append(
                f"Move {', '.join(fit.matched_skills[:3])} closer to the top of the resume."
            )
        if explanation and explanation.application_angle:
            resume_focus_points.append(explanation.application_angle)

        suggested_bullets: list[str] = []
        if analyzed_profile and analyzed_profile.summary:
            suggested_bullets.append(
                f"Highlight experience already reflected in the profile summary: {analyzed_profile.summary}"
            )
        if fit.matched_skills:
            suggested_bullets.append(
                f"Reframe existing work around {', '.join(fit.matched_skills[:3])} because those skills match this job."
            )
        if fit.reasons:
            suggested_bullets.append(
                f"Use deterministic fit evidence as framing for a bullet: {fit.reasons[0]}"
            )

        cover_letter_angles: list[str] = []
        if explanation and explanation.fit_summary:
            cover_letter_angles.append(explanation.fit_summary)
        if job.summary:
            cover_letter_angles.append(
                f"Connect existing profile evidence to the job summary: {job.summary}"
            )
        if feedback_state and feedback_state.current_job_feedback:
            if feedback_state.current_job_feedback.company_status == "whitelist":
                cover_letter_angles.append(
                    "Mention why this company already fits the current target list."
                )

        interview_focus: list[str] = []
        if fit.matched_skills:
            interview_focus.append(
                f"Prepare concrete examples around {', '.join(fit.matched_skills[:3])}."
            )
        if fit.matched_keywords:
            interview_focus.append(
                f"Be ready to explain overlap with {', '.join(fit.matched_keywords[:3])}."
            )
        if explanation and explanation.missing_signals:
            interview_focus.extend(explanation.missing_signals[:2])

        gaps_to_address: list[str] = []
        if not fit.matched_skills:
            gaps_to_address.append("The deterministic fit does not expose matched skills for this job yet.")
        if fit.work_mode_match is False:
            gaps_to_address.append("The current search work-mode preference does not align cleanly.")
        if fit.region_match is False:
            gaps_to_address.append("The current target regions do not align cleanly with this job.")
        if not analyzed_profile and not context.raw_profile_text:
            gaps_to_address.append("Profile evidence is limited, so tailoring depth is constrained.")

        red_flags: list[str] = []
        if fit.score < 40:
            red_flags.append("Low deterministic fit score means the application should be treated cautiously.")
        if feedback_state and feedback_state.current_job_feedback:
            if feedback_state.current_job_feedback.bad_fit:
                red_flags.append("This job is already marked as bad fit in feedback.")
            if feedback_state.current_job_feedback.company_status == "blacklist":
                red_flags.append("The company is blacklisted in current feedback state.")

        return {
            "application_summary": (
                f"Tailor this application around the existing {role_anchor} evidence that already overlaps with {job.title} at {job.company_name}, while keeping claims bounded to deterministic score {fit.score} and the provided profile context."
            ),
            "resume_focus_points": resume_focus_points,
            "suggested_bullets": suggested_bullets,
            "cover_letter_angles": cover_letter_angles,
            "interview_focus": interview_focus,
            "gaps_to_address": gaps_to_address,
            "red_flags": red_flags,
        }

    async def generate_cover_letter_draft(
        self, context: CoverLetterDraftRequest, prompt: CoverLetterDraftPrompt
    ) -> dict[str, Any]:
        analyzed_profile = context.analyzed_profile
        search_profile = context.search_profile
        job = context.ranked_job
        fit = context.deterministic_fit
        explanation = context.job_fit_explanation
        coaching = context.application_coach
        feedback_state = context.feedback_state

        role_anchor = (
            analyzed_profile.primary_role.replace("_", " ")
            if analyzed_profile
            else search_profile.primary_role.replace("_", " ")
            if search_profile
            else "target role"
        )

        key_claims_used: list[str] = []
        if analyzed_profile and analyzed_profile.summary:
            key_claims_used.append(analyzed_profile.summary)
        if fit.matched_roles:
            key_claims_used.append(
                f"Role overlap is explicit with {', '.join(fit.matched_roles[:2])}."
            )
        if fit.matched_skills:
            key_claims_used.append(
                f"Matched skills include {', '.join(fit.matched_skills[:3])}."
            )
        if fit.reasons:
            key_claims_used.append(f"Deterministic fit evidence: {fit.reasons[0]}")
        if explanation and explanation.application_angle:
            key_claims_used.append(explanation.application_angle)

        evidence_gaps: list[str] = []
        if explanation and explanation.missing_signals:
            evidence_gaps.extend(explanation.missing_signals[:2])
        if coaching and coaching.gaps_to_address:
            evidence_gaps.extend(coaching.gaps_to_address[:2])
        if not analyzed_profile and not context.raw_profile_text:
            evidence_gaps.append("Profile evidence is limited, so personalization depth is constrained.")
        if not fit.matched_skills:
            evidence_gaps.append("The deterministic fit does not expose matched skills for this job yet.")
        if fit.region_match is False:
            evidence_gaps.append("Region alignment is not explicit in the deterministic context.")
        if fit.work_mode_match is False:
            evidence_gaps.append("Work-mode alignment is not explicit in the deterministic context.")

        opening_parts: list[str] = [
            f"I am applying for the {job.title} role at {job.company_name}."
        ]
        if analyzed_profile and analyzed_profile.summary:
            opening_parts.append(
                f"The provided profile context positions me as {analyzed_profile.summary}"
            )
        elif fit.matched_roles:
            opening_parts.append(
                f"The deterministic fit shows direct overlap with {', '.join(fit.matched_roles[:2])} work."
            )
        if fit.matched_skills:
            opening_parts.append(
                f"The strongest explicit overlap in the available evidence is around {', '.join(fit.matched_skills[:3])}."
            )

        body_paragraphs: list[str] = []
        if fit.reasons or explanation and explanation.fit_summary:
            evidence_parts: list[str] = []
            if fit.reasons:
                evidence_parts.append(fit.reasons[0])
            if explanation and explanation.fit_summary:
                evidence_parts.append(explanation.fit_summary)
            body_paragraphs.append(
                " ".join(
                    [
                        "The deterministic context points to a grounded match for this role.",
                        *evidence_parts,
                    ]
                ).strip()
            )

        second_body_parts: list[str] = []
        if job.summary:
            second_body_parts.append(f"The job summary emphasizes {job.summary}")
        if explanation and explanation.application_angle:
            second_body_parts.append(explanation.application_angle)
        elif coaching and coaching.cover_letter_angles:
            second_body_parts.append(coaching.cover_letter_angles[0])
        elif fit.matched_keywords:
            second_body_parts.append(
                f"The available overlap also includes {', '.join(fit.matched_keywords[:3])}."
            )
        if search_profile and search_profile.target_roles:
            second_body_parts.append(
                f"This role also aligns with the active search direction toward {', '.join(search_profile.target_roles[:2])}."
            )
        if second_body_parts:
            body_paragraphs.append(" ".join(second_body_parts).strip())

        if feedback_state and feedback_state.current_job_feedback:
            if feedback_state.current_job_feedback.company_status == "whitelist":
                body_paragraphs.append(
                    "The current feedback state also shows this company is already on the positive target list."
                )
            elif feedback_state.current_job_feedback.company_status == "blacklist":
                evidence_gaps.append(
                    "The company is blacklisted in feedback state, so the draft should be used cautiously."
                )

        tone_notes: list[str] = ["Keep the tone direct, tailored, and evidence-based."]
        if evidence_gaps:
            tone_notes.append("Acknowledge alignment without overstating unsupported depth.")
        if fit.score >= 70:
            tone_notes.append("Confident language is reasonable because deterministic fit is strong.")
        else:
            tone_notes.append("Keep the draft measured because the deterministic fit is mixed.")

        closing_parts = [
            f"I would welcome the opportunity to discuss how the available {role_anchor} evidence could support the needs of {job.company_name}."
        ]
        if evidence_gaps:
            closing_parts.append(
                "Any deeper claims should stay limited to examples that can be verified from the existing profile context."
            )

        return {
            "draft_summary": (
                f"Frame the letter around explicit {role_anchor} overlap, the deterministic fit score of {fit.score}, and the matched evidence already present in the profile and job context."
            ),
            "opening_paragraph": " ".join(opening_parts).strip(),
            "body_paragraphs": body_paragraphs,
            "closing_paragraph": " ".join(closing_parts).strip(),
            "key_claims_used": key_claims_used,
            "evidence_gaps": evidence_gaps,
            "tone_notes": tone_notes,
        }

    async def generate_interview_prep(
        self, context: InterviewPrepRequest, prompt: InterviewPrepPrompt
    ) -> dict[str, Any]:
        analyzed_profile = context.analyzed_profile
        search_profile = context.search_profile
        job = context.ranked_job
        fit = context.deterministic_fit
        explanation = context.job_fit_explanation
        coaching = context.application_coach
        draft = context.cover_letter_draft
        feedback_state = context.feedback_state

        role_anchor = (
            analyzed_profile.primary_role.replace("_", " ")
            if analyzed_profile
            else search_profile.primary_role.replace("_", " ")
            if search_profile
            else "target role"
        )

        likely_topics: list[str] = []
        if fit.matched_roles:
            likely_topics.append(
                f"How your {', '.join(fit.matched_roles[:2])} background maps to this role."
            )
        if fit.matched_skills:
            likely_topics.append(
                f"Concrete discussion of {', '.join(fit.matched_skills[:3])} because those skills match deterministically."
            )
        if job.summary:
            likely_topics.append(f"Responsibilities and scope highlighted in the job summary: {job.summary}")
        if explanation and explanation.why_it_matches:
            likely_topics.extend(explanation.why_it_matches[:2])

        technical_focus: list[str] = []
        if fit.matched_skills:
            technical_focus.append(
                f"Prepare concrete examples for {', '.join(fit.matched_skills[:3])} using only experience already supported by the profile context."
            )
        if fit.matched_keywords:
            technical_focus.append(
                f"Be ready to explain overlap with {', '.join(fit.matched_keywords[:3])} in the job description."
            )
        if fit.reasons:
            technical_focus.append(f"Rehearse the deterministic evidence already called out: {fit.reasons[0]}")
        if coaching and coaching.interview_focus:
            technical_focus.extend(coaching.interview_focus[:2])

        behavioral_focus: list[str] = []
        behavioral_focus.append(
            f"Explain why this {role_anchor} direction is a grounded next step based on the current profile and search profile."
        )
        if feedback_state and feedback_state.current_job_feedback:
            if feedback_state.current_job_feedback.company_status == "whitelist":
                behavioral_focus.append(
                    "Be ready to explain why this company is already on the positive target list."
                )
        if explanation and explanation.recommended_next_step:
            behavioral_focus.append(explanation.recommended_next_step)
        if draft and draft.tone_notes:
            behavioral_focus.extend(draft.tone_notes[:1])

        stories_to_prepare: list[str] = []
        if analyzed_profile and analyzed_profile.summary:
            stories_to_prepare.append(
                f"Choose one example that supports this profile summary without adding any facts beyond it: {analyzed_profile.summary}"
            )
        if fit.matched_skills:
            stories_to_prepare.append(
                f"Choose one example for {', '.join(fit.matched_skills[:2])} that is already supported by the available profile evidence."
            )
        if fit.reasons:
            stories_to_prepare.append(
                f"Prepare a concise story that backs up this deterministic reason: {fit.reasons[0]}"
            )
        if coaching and coaching.suggested_bullets:
            stories_to_prepare.extend(coaching.suggested_bullets[:2])

        questions_to_ask: list[str] = []
        questions_to_ask.append(
            f"What outcomes matter most for the {job.title} role in the first few months?"
        )
        if fit.matched_skills:
            questions_to_ask.append(
                f"Which parts of the role depend most on {', '.join(fit.matched_skills[:2])}?"
            )
        if explanation and explanation.missing_signals:
            questions_to_ask.append(
                f"Could you clarify this area that is not explicit in the current context: {explanation.missing_signals[0]}"
            )
        if job.work_mode_label or job.location_label:
            questions_to_ask.append(
                f"How does the team operate across {job.work_mode_label or 'the expected work mode'} and {job.location_label or 'the listed location constraints'}?"
            )

        risk_areas: list[str] = []
        if explanation and explanation.risks:
            risk_areas.extend(explanation.risks[:2])
        if explanation and explanation.missing_signals:
            risk_areas.extend(explanation.missing_signals[:2])
        if coaching and coaching.gaps_to_address:
            risk_areas.extend(coaching.gaps_to_address[:2])
        if draft and draft.evidence_gaps:
            risk_areas.extend(draft.evidence_gaps[:2])
        if not fit.matched_skills:
            risk_areas.append("The deterministic fit does not expose matched skills for this job yet.")
        if not analyzed_profile and not context.raw_profile_text:
            risk_areas.append("Profile evidence is limited, so interview examples may be too generic.")
        if fit.work_mode_match is False:
            risk_areas.append("Work-mode alignment is not clean in the deterministic context.")
        if fit.region_match is False:
            risk_areas.append("Region alignment is not clean in the deterministic context.")

        follow_up_plan: list[str] = []
        follow_up_plan.append(
            "Review the deterministic fit reasons and matched terms immediately before the interview."
        )
        if fit.matched_skills:
            follow_up_plan.append(
                f"Pick one verified example for each of {', '.join(fit.matched_skills[:2])} and keep the evidence concise."
            )
        if risk_areas:
            follow_up_plan.append(
                "Prepare short, candid responses for the listed risk areas instead of improvising unsupported details."
            )
        if questions_to_ask:
            follow_up_plan.append("Bring two or three grounded questions so the conversation stays evidence-based.")

        return {
            "prep_summary": (
                f"Prepare for {job.title} at {job.company_name} by centering the discussion on explicit {role_anchor} overlap, deterministic fit score {fit.score}, and examples that are already supported by the provided profile context."
            ),
            "likely_topics": likely_topics,
            "technical_focus": technical_focus,
            "behavioral_focus": behavioral_focus,
            "stories_to_prepare": stories_to_prepare,
            "questions_to_ask": questions_to_ask,
            "risk_areas": risk_areas,
            "follow_up_plan": follow_up_plan,
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
        prompt: (
            ProfileInsightsPrompt
            | JobFitExplanationPrompt
            | ApplicationCoachPrompt
            | CoverLetterDraftPrompt
            | InterviewPrepPrompt
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


def build_application_coach_provider() -> ApplicationCoachProvider:
    try:
        provider = _build_enrichment_provider()
    except ProfileInsightsProviderError as exc:
        raise ApplicationCoachProviderError(str(exc)) from exc
    return provider


def build_cover_letter_draft_provider() -> CoverLetterDraftProvider:
    try:
        provider = _build_enrichment_provider()
    except ProfileInsightsProviderError as exc:
        raise CoverLetterDraftProviderError(str(exc)) from exc
    return provider


def build_interview_prep_provider() -> InterviewPrepProvider:
    try:
        provider = _build_enrichment_provider()
    except ProfileInsightsProviderError as exc:
        raise InterviewPrepProviderError(str(exc)) from exc
    return provider
