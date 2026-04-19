from typing import Any, Protocol

from app.application_coach import ApplicationCoachPrompt, ApplicationCoachRequest
from app.cover_letter_draft import CoverLetterDraftPrompt, CoverLetterDraftRequest
from app.interview_prep import InterviewPrepPrompt, InterviewPrepRequest
from app.job_fit_explanation import JobFitExplanationPrompt, JobFitExplanationRequest
from app.profile_insights import LlmContextRequest, ProfileInsightsPrompt
from app.weekly_guidance import (
    WeeklyGuidancePrompt,
    WeeklyGuidanceRequest,
)


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


from app.llm_provider_factory import (
    build_application_coach_provider,
    build_cover_letter_draft_provider,
    build_interview_prep_provider,
    build_job_fit_explanation_provider,
    build_profile_insights_provider,
    build_weekly_guidance_provider,
)
from app.llm_provider_remote import OllamaEnrichmentProvider, OpenAIEnrichmentProvider
from app.llm_provider_template import TemplateEnrichmentProvider
