from typing import TypeAlias

import httpx
from tenacity import AsyncRetrying, Retrying, retry_if_exception_type, stop_after_attempt, wait_exponential

from app.enrichment.application_coach.contract import ApplicationCoachPrompt
from app.enrichment.cover_letter_draft.contract import CoverLetterDraftPrompt
from app.enrichment.interview_prep.contract import InterviewPrepPrompt
from app.enrichment.job_fit_explanation.contract import JobFitExplanationPrompt
from app.enrichment.profile_insights.contract import ProfileInsightsPrompt
from app.enrichment.weekly_guidance.prompt import WeeklyGuidancePrompt
from app.settings import DEFAULT_LLM_REQUEST_TIMEOUT_SECONDS


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


def build_rate_limit_retrying(rate_limit_error: type[BaseException]) -> Retrying:
    return Retrying(
        stop=stop_after_attempt(5),
        wait=wait_exponential(min=1, max=60),
        retry=retry_if_exception_type(rate_limit_error),
        reraise=True,
    )


def build_async_retrying(retryable_errors: tuple[type[BaseException], ...]) -> AsyncRetrying:
    return AsyncRetrying(
        stop=stop_after_attempt(3),
        wait=wait_exponential(multiplier=0.2, min=0.2, max=1.0),
        retry=retry_if_exception_type(retryable_errors),
        reraise=True,
    )
