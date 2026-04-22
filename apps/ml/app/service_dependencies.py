from functools import lru_cache

from app.application_coach import (
    ApplicationCoachProviderError,
    http_error_from_application_coach_error,
)
from app.application_coach_service import ApplicationCoachService
from app.cover_letter_draft import (
    CoverLetterDraftProviderError,
    http_error_from_cover_letter_draft_error,
)
from app.cover_letter_draft_service import CoverLetterDraftService
from app.interview_prep import (
    InterviewPrepProviderError,
    http_error_from_interview_prep_error,
)
from app.interview_prep_service import InterviewPrepService
from app.job_fit_explanation import (
    JobFitExplanationProviderError,
    http_error_from_job_fit_explanation_error,
)
from app.job_fit_explanation_service import JobFitExplanationService
from app.llm_provider_factory import (
    build_application_coach_provider,
    build_cover_letter_draft_provider,
    build_interview_prep_provider,
    build_job_fit_explanation_provider,
    build_profile_insights_provider,
    build_weekly_guidance_provider,
)
from app.profile_insights import (
    ProfileInsightsProviderError,
    http_error_from_provider_error,
)
from app.profile_insights_service import ProfileInsightsService
from app.weekly_guidance import (
    WeeklyGuidanceProviderError,
    http_error_from_weekly_guidance_error,
)
from app.weekly_guidance_service import WeeklyGuidanceService


@lru_cache(maxsize=1)
def build_cached_job_fit_explanation_service() -> JobFitExplanationService:
    return JobFitExplanationService(build_job_fit_explanation_provider())


def get_job_fit_explanation_service() -> JobFitExplanationService:
    try:
        return build_cached_job_fit_explanation_service()
    except JobFitExplanationProviderError as exc:
        raise http_error_from_job_fit_explanation_error(exc) from exc


@lru_cache(maxsize=1)
def build_cached_application_coach_service() -> ApplicationCoachService:
    return ApplicationCoachService(build_application_coach_provider())


def get_application_coach_service() -> ApplicationCoachService:
    try:
        return build_cached_application_coach_service()
    except ApplicationCoachProviderError as exc:
        raise http_error_from_application_coach_error(exc) from exc


@lru_cache(maxsize=1)
def build_cached_cover_letter_draft_service() -> CoverLetterDraftService:
    return CoverLetterDraftService(build_cover_letter_draft_provider())


def get_cover_letter_draft_service() -> CoverLetterDraftService:
    try:
        return build_cached_cover_letter_draft_service()
    except CoverLetterDraftProviderError as exc:
        raise http_error_from_cover_letter_draft_error(exc) from exc


@lru_cache(maxsize=1)
def build_cached_interview_prep_service() -> InterviewPrepService:
    return InterviewPrepService(build_interview_prep_provider())


def get_interview_prep_service() -> InterviewPrepService:
    try:
        return build_cached_interview_prep_service()
    except InterviewPrepProviderError as exc:
        raise http_error_from_interview_prep_error(exc) from exc


@lru_cache(maxsize=1)
def build_cached_profile_insights_service() -> ProfileInsightsService:
    return ProfileInsightsService(build_profile_insights_provider())


def get_profile_insights_service() -> ProfileInsightsService:
    try:
        return build_cached_profile_insights_service()
    except ProfileInsightsProviderError as exc:
        raise http_error_from_provider_error(exc) from exc


@lru_cache(maxsize=1)
def build_cached_weekly_guidance_service() -> WeeklyGuidanceService:
    return WeeklyGuidanceService(build_weekly_guidance_provider())


def get_weekly_guidance_service() -> WeeklyGuidanceService:
    try:
        return build_cached_weekly_guidance_service()
    except WeeklyGuidanceProviderError as exc:
        raise http_error_from_weekly_guidance_error(exc) from exc
