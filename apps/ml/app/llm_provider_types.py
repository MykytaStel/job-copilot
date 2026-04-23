from typing import Any, Protocol

from app.enrichment.application_coach.contract import (
    ApplicationCoachPrompt,
    ApplicationCoachRequest,
)
from app.enrichment.cover_letter_draft.contract import (
    CoverLetterDraftPrompt,
    CoverLetterDraftRequest,
)
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


class WeeklyGuidanceProvider(Protocol):
    async def generate_weekly_guidance(
        self, context: WeeklyGuidanceRequest, prompt: WeeklyGuidancePrompt
    ) -> Any: ...
