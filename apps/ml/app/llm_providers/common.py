from typing import TypeAlias

import httpx
from tenacity import AsyncRetrying, Retrying, retry_if_exception_type, stop_after_attempt, wait_exponential

from app.application_coach import ApplicationCoachPrompt
from app.cover_letter_draft import CoverLetterDraftPrompt
from app.interview_prep import InterviewPrepPrompt
from app.job_fit_explanation import JobFitExplanationPrompt
from app.profile_insights import ProfileInsightsPrompt
from app.settings import DEFAULT_LLM_REQUEST_TIMEOUT_SECONDS
from app.weekly_guidance import WeeklyGuidancePrompt


PromptPayload: TypeAlias = (
    ProfileInsightsPrompt
    | JobFitExplanationPrompt
    | ApplicationCoachPrompt
    | CoverLetterDraftPrompt
    | InterviewPrepPrompt
    | WeeklyGuidancePrompt
)


def build_async_client(timeout_seconds: float = DEFAULT_LLM_REQUEST_TIMEOUT_SECONDS) -> httpx.AsyncClient:
    return httpx.AsyncClient(timeout=httpx.Timeout(timeout_seconds))


def build_retrying(retryable_errors: tuple[type[BaseException], ...]) -> Retrying:
    return Retrying(
        stop=stop_after_attempt(3),
        wait=wait_exponential(multiplier=0.2, min=0.2, max=1.0),
        retry=retry_if_exception_type(retryable_errors),
        reraise=True,
    )


def build_async_retrying(retryable_errors: tuple[type[BaseException], ...]) -> AsyncRetrying:
    return AsyncRetrying(
        stop=stop_after_attempt(3),
        wait=wait_exponential(multiplier=0.2, min=0.2, max=1.0),
        retry=retry_if_exception_type(retryable_errors),
        reraise=True,
    )
