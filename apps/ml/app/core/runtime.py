from __future__ import annotations

import inspect
import logging
from dataclasses import dataclass
from pathlib import Path
from typing import Any

from app import bootstrap_training
from app.application_coach import ApplicationCoachProviderError
from app.application_coach_service import ApplicationCoachService
from app.bootstrap.task_store import BootstrapTaskStore
from app.cover_letter_draft import CoverLetterDraftProviderError
from app.cover_letter_draft_service import CoverLetterDraftService
from app.engine_api_client import build_shared_http_client, configure_shared_client
from app.fit_analysis_service import FitAnalysisService
from app.interview_prep import InterviewPrepProviderError
from app.interview_prep_service import InterviewPrepService
from app.job_fit_explanation import JobFitExplanationProviderError
from app.job_fit_explanation_service import JobFitExplanationService
from app.llm_provider_factory import build_enrichment_provider
from app.profile_insights import ProfileInsightsProviderError
from app.profile_insights_service import ProfileInsightsService
from app.rerank_service import RerankService
from app.reranker_bootstrap_service import RerankerBootstrapService
from app.settings import DEFAULT_BOOTSTRAP_TASKS_DIR, RuntimeSettings
from app.weekly_guidance import WeeklyGuidanceProviderError
from app.weekly_guidance_service import WeeklyGuidanceService

logger = logging.getLogger(__name__)


@dataclass
class AppServices:
    settings: RuntimeSettings
    task_store: BootstrapTaskStore
    fit_analysis_service: FitAnalysisService
    rerank_service: RerankService
    reranker_bootstrap_service: RerankerBootstrapService
    profile_insights_service: ProfileInsightsService | None
    job_fit_explanation_service: JobFitExplanationService | None
    application_coach_service: ApplicationCoachService | None
    cover_letter_draft_service: CoverLetterDraftService | None
    interview_prep_service: InterviewPrepService | None
    weekly_guidance_service: WeeklyGuidanceService | None
    enrichment_provider_error: str | None
    _http_client: Any
    _enrichment_provider: Any | None


async def build_app_services(settings: RuntimeSettings) -> AppServices:
    http_client = build_shared_http_client(settings)
    configure_shared_client(http_client)

    def engine_api_client_factory():
        from app.engine_api_client import engine_api_client_context

        return engine_api_client_context()

    task_store = BootstrapTaskStore(
        Path(settings.bootstrap_tasks_dir or DEFAULT_BOOTSTRAP_TASKS_DIR),
        task_ttl_hours=settings.task_ttl_hours,
    )
    task_store.cleanup_expired()

    fit_analysis_service = FitAnalysisService(engine_api_client_factory)
    rerank_service = RerankService(engine_api_client_factory)
    reranker_bootstrap_service = RerankerBootstrapService(
        bootstrap_workflow=bootstrap_training.bootstrap_and_retrain,
        lock_dir=task_store.locks_dir,
    )

    enrichment_provider = None
    enrichment_provider_error = None
    profile_insights_service = None
    job_fit_explanation_service = None
    application_coach_service = None
    cover_letter_draft_service = None
    interview_prep_service = None
    weekly_guidance_service = None

    try:
        enrichment_provider = build_enrichment_provider()
        profile_insights_service = ProfileInsightsService(enrichment_provider)
        job_fit_explanation_service = JobFitExplanationService(enrichment_provider)
        application_coach_service = ApplicationCoachService(enrichment_provider)
        cover_letter_draft_service = CoverLetterDraftService(enrichment_provider)
        interview_prep_service = InterviewPrepService(enrichment_provider)
        weekly_guidance_service = WeeklyGuidanceService(enrichment_provider)
    except (
        ApplicationCoachProviderError,
        CoverLetterDraftProviderError,
        InterviewPrepProviderError,
        JobFitExplanationProviderError,
        ProfileInsightsProviderError,
        WeeklyGuidanceProviderError,
    ) as exc:
        enrichment_provider_error = str(exc) or "enrichment provider unavailable"
        logger.warning("enrichment provider unavailable at startup: %s", enrichment_provider_error)

    return AppServices(
        settings=settings,
        task_store=task_store,
        fit_analysis_service=fit_analysis_service,
        rerank_service=rerank_service,
        reranker_bootstrap_service=reranker_bootstrap_service,
        profile_insights_service=profile_insights_service,
        job_fit_explanation_service=job_fit_explanation_service,
        application_coach_service=application_coach_service,
        cover_letter_draft_service=cover_letter_draft_service,
        interview_prep_service=interview_prep_service,
        weekly_guidance_service=weekly_guidance_service,
        enrichment_provider_error=enrichment_provider_error,
        _http_client=http_client,
        _enrichment_provider=enrichment_provider,
    )


async def close_app_services(services: AppServices) -> None:
    if services._enrichment_provider is not None:
        close = getattr(services._enrichment_provider, "aclose", None)
        if close is not None:
            result = close()
            if inspect.isawaitable(result):
                await result
        else:
            close = getattr(services._enrichment_provider, "close", None)
            if close is not None:
                close()
    await services._http_client.aclose()
    configure_shared_client(None)
