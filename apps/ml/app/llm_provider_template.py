from typing import Any

from app.application_coach import ApplicationCoachPrompt, ApplicationCoachRequest
from app.cover_letter_draft import CoverLetterDraftPrompt, CoverLetterDraftRequest
from app.interview_prep import InterviewPrepPrompt, InterviewPrepRequest
from app.job_fit_explanation import JobFitExplanationPrompt, JobFitExplanationRequest
from app.profile_insights import LlmContextRequest, ProfileInsightsPrompt
from app.template_application_coach import build_application_coach
from app.template_cover_letter_draft import build_cover_letter_draft
from app.template_interview_prep import build_interview_prep
from app.template_job_fit_explanation import build_job_fit_explanation
from app.template_profile_insights import build_profile_insights
from app.template_weekly_guidance import build_weekly_guidance
from app.weekly_guidance import WeeklyGuidancePrompt, WeeklyGuidanceRequest


class TemplateEnrichmentProvider:
    async def generate_profile_insights(
        self, context: LlmContextRequest, prompt: ProfileInsightsPrompt
    ) -> dict[str, Any]:
        return build_profile_insights(context, prompt)

    async def generate_job_fit_explanation(
        self, context: JobFitExplanationRequest, prompt: JobFitExplanationPrompt
    ) -> dict[str, Any]:
        return build_job_fit_explanation(context, prompt)

    async def generate_application_coach(
        self, context: ApplicationCoachRequest, prompt: ApplicationCoachPrompt
    ) -> dict[str, Any]:
        return build_application_coach(context)

    async def generate_cover_letter_draft(
        self, context: CoverLetterDraftRequest, prompt: CoverLetterDraftPrompt
    ) -> dict[str, Any]:
        return build_cover_letter_draft(context)

    async def generate_interview_prep(
        self, context: InterviewPrepRequest, prompt: InterviewPrepPrompt
    ) -> dict[str, Any]:
        return build_interview_prep(context)

    async def generate_weekly_guidance(
        self, context: WeeklyGuidanceRequest, prompt: WeeklyGuidancePrompt
    ) -> dict[str, Any]:
        return build_weekly_guidance(context, prompt)
