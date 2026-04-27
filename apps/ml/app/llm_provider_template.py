from typing import Any

from app.enrichment.application_coach.contract import (
    ApplicationCoachPrompt,
    ApplicationCoachRequest,
)
from app.enrichment.cover_letter_draft.contract import (
    CoverLetterDraftPrompt,
    CoverLetterDraftRequest,
)
from app.enrichment.cv_tailoring.contract import CvTailoringPrompt, CvTailoringRequest
from app.enrichment.interview_prep.contract import InterviewPrepPrompt, InterviewPrepRequest
from app.enrichment.job_fit_explanation.contract import (
    JobFitExplanationPrompt,
    JobFitExplanationRequest,
)
from app.enrichment.profile_insights.contract import (
    LlmContextRequest,
    ProfileInsightsPrompt,
)
from app.enrichment.weekly_guidance.contract import WeeklyGuidanceRequest
from app.enrichment.weekly_guidance.prompt import WeeklyGuidancePrompt
from app.template_application_coach import build_application_coach
from app.template_cover_letter_draft import build_cover_letter_draft
from app.template_cv_tailoring import build_cv_tailoring
from app.template_interview_prep import build_interview_prep
from app.template_job_fit_explanation import build_job_fit_explanation
from app.template_profile_insights import build_profile_insights
from app.template_weekly_guidance import build_weekly_guidance


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

    async def generate_cv_tailoring(
        self, context: CvTailoringRequest, prompt: CvTailoringPrompt
    ) -> dict[str, Any]:
        return build_cv_tailoring(context)
