from typing import Any, Protocol

from app.enrichment.application_coach import ApplicationCoachPrompt, ApplicationCoachRequest
from app.enrichment.cover_letter_draft import CoverLetterDraftPrompt, CoverLetterDraftRequest
from app.enrichment.interview_prep import InterviewPrepPrompt, InterviewPrepRequest
from app.enrichment.job_fit_explanation import JobFitExplanationPrompt, JobFitExplanationRequest
from app.enrichment.profile_insights import LlmContextRequest, ProfileInsightsPrompt
from app.enrichment.weekly_guidance import WeeklyGuidancePrompt, WeeklyGuidanceRequest


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
