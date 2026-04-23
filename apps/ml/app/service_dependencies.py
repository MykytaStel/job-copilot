from functools import lru_cache

from fastapi import Request

from app.application_coach import (
    ApplicationCoachProviderError,
    http_error_from_application_coach_error,
)
from app.cover_letter_draft import (
    CoverLetterDraftProviderError,
    http_error_from_cover_letter_draft_error,
)
from app.core.runtime import AppServices
from app.interview_prep import InterviewPrepProviderError, http_error_from_interview_prep_error
from app.job_fit_explanation import JobFitExplanationProviderError, http_error_from_job_fit_explanation_error
from app.profile_insights import (
    ProfileInsightsProviderError,
    http_error_from_provider_error,
)
from app.weekly_guidance import (
    WeeklyGuidanceProviderError,
    http_error_from_weekly_guidance_error,
)
from app.engine_api_client import engine_api_client_context
from app.reranker_bootstrap_service import RerankerBootstrapService
from app import bootstrap_training
from app.settings import DEFAULT_BOOTSTRAP_TASKS_DIR
from app.trained_reranker_config import DEFAULT_TRAINED_RERANKER_MODEL_PATH

def get_app_services(request: Request) -> AppServices:
    return request.app.state.services


def engine_api_client_factory():
    return engine_api_client_context()


def get_fit_analysis_service(request: Request):
    return get_app_services(request).fit_analysis_service


def get_rerank_service(request: Request):
    return get_app_services(request).rerank_service


def get_reranker_bootstrap_service(request: Request):
    return get_app_services(request).reranker_bootstrap_service


@lru_cache(maxsize=1)
def build_cached_reranker_bootstrap_service():
    return RerankerBootstrapService(
        bootstrap_workflow=bootstrap_training.bootstrap_and_retrain,
        model_path=DEFAULT_TRAINED_RERANKER_MODEL_PATH,
    )


def get_profile_insights_service(request: Request):
    services = get_app_services(request)
    if services.profile_insights_service is not None:
        return services.profile_insights_service
    raise http_error_from_provider_error(
        ProfileInsightsProviderError(services.enrichment_provider_error or "provider unavailable")
    )


def get_job_fit_explanation_service(request: Request):
    services = get_app_services(request)
    if services.job_fit_explanation_service is not None:
        return services.job_fit_explanation_service
    raise http_error_from_job_fit_explanation_error(
        JobFitExplanationProviderError(services.enrichment_provider_error or "provider unavailable")
    )


def get_application_coach_service(request: Request):
    services = get_app_services(request)
    if services.application_coach_service is not None:
        return services.application_coach_service
    raise http_error_from_application_coach_error(
        ApplicationCoachProviderError(services.enrichment_provider_error or "provider unavailable")
    )


def get_cover_letter_draft_service(request: Request):
    services = get_app_services(request)
    if services.cover_letter_draft_service is not None:
        return services.cover_letter_draft_service
    raise http_error_from_cover_letter_draft_error(
        CoverLetterDraftProviderError(services.enrichment_provider_error or "provider unavailable")
    )


def get_interview_prep_service(request: Request):
    services = get_app_services(request)
    if services.interview_prep_service is not None:
        return services.interview_prep_service
    raise http_error_from_interview_prep_error(
        InterviewPrepProviderError(services.enrichment_provider_error or "provider unavailable")
    )


def get_weekly_guidance_service(request: Request):
    services = get_app_services(request)
    if services.weekly_guidance_service is not None:
        return services.weekly_guidance_service
    raise http_error_from_weekly_guidance_error(
        WeeklyGuidanceProviderError(services.enrichment_provider_error or "provider unavailable")
    )
