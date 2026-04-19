from fastapi import Depends, FastAPI
from fastapi.middleware.cors import CORSMiddleware

from app.application_coach import (
    ApplicationCoachProviderError,
    ApplicationCoachRequest,
    ApplicationCoachResponse,
    http_error_from_application_coach_error,
)
from app.cover_letter_draft import (
    CoverLetterDraftProviderError,
    CoverLetterDraftRequest,
    CoverLetterDraftResponse,
    http_error_from_cover_letter_draft_error,
)
from app.engine_api_client import engine_api_base_url
from app.interview_prep import (
    InterviewPrepProviderError,
    InterviewPrepRequest,
    InterviewPrepResponse,
    http_error_from_interview_prep_error,
)
from app.job_fit_explanation import (
    JobFitExplanationProviderError,
    JobFitExplanationRequest,
    JobFitExplanationResponse,
    http_error_from_job_fit_explanation_error,
)
from app.models import HealthResponse
from app.profile_insights import (
    LlmContextRequest,
    ProfileInsightsProviderError,
    ProfileInsightsResponse,
    http_error_from_provider_error,
)
from app.scoring_routes import router as scoring_router
from app.service_dependencies import (
    get_application_coach_service,
    get_cover_letter_draft_service,
    get_interview_prep_service,
    get_job_fit_explanation_service,
    get_profile_insights_service,
    get_weekly_guidance_service,
)
from app.settings import configure_logging, get_runtime_settings
from app.weekly_guidance import (
    WeeklyGuidanceProviderError,
    WeeklyGuidanceRequest,
    WeeklyGuidanceResponse,
    http_error_from_weekly_guidance_error,
)


def create_app() -> FastAPI:
    configure_logging()
    settings = get_runtime_settings()

    application = FastAPI(
        title="job-copilot-ml",
        version="0.1.0",
        description="Read-only ML sidecar over canonical engine-api data.",
    )
    application.state.runtime_settings = settings
    application.add_middleware(
        CORSMiddleware,
        allow_origins=list(settings.cors_allowed_origins),
        allow_methods=["*"],
        allow_headers=["*"],
    )

    application.include_router(scoring_router)

    @application.get("/health", response_model=HealthResponse)
    async def health() -> HealthResponse:
        return HealthResponse(
            status="ok",
            service="ml",
            engine_api_base_url=engine_api_base_url(),
        )

    @application.post("/v1/enrichment/profile-insights", response_model=ProfileInsightsResponse)
    @application.post("/api/v1/enrichment/profile-insights", response_model=ProfileInsightsResponse)
    async def enrich_profile_insights(
        payload: LlmContextRequest,
        service=Depends(get_profile_insights_service),
    ) -> ProfileInsightsResponse:
        try:
            return await service.enrich(payload)
        except ProfileInsightsProviderError as exc:
            raise http_error_from_provider_error(exc) from exc

    @application.post(
        "/v1/enrichment/job-fit-explanation",
        response_model=JobFitExplanationResponse,
    )
    @application.post(
        "/api/v1/enrichment/job-fit-explanation",
        response_model=JobFitExplanationResponse,
    )
    async def enrich_job_fit_explanation(
        payload: JobFitExplanationRequest,
        service=Depends(get_job_fit_explanation_service),
    ) -> JobFitExplanationResponse:
        try:
            return await service.enrich(payload)
        except JobFitExplanationProviderError as exc:
            raise http_error_from_job_fit_explanation_error(exc) from exc

    @application.post("/v1/enrichment/application-coach", response_model=ApplicationCoachResponse)
    @application.post(
        "/api/v1/enrichment/application-coach",
        response_model=ApplicationCoachResponse,
    )
    async def enrich_application_coach(
        payload: ApplicationCoachRequest,
        service=Depends(get_application_coach_service),
    ) -> ApplicationCoachResponse:
        try:
            return await service.enrich(payload)
        except ApplicationCoachProviderError as exc:
            raise http_error_from_application_coach_error(exc) from exc

    @application.post(
        "/v1/enrichment/cover-letter-draft",
        response_model=CoverLetterDraftResponse,
    )
    @application.post(
        "/api/v1/enrichment/cover-letter-draft",
        response_model=CoverLetterDraftResponse,
    )
    async def enrich_cover_letter_draft(
        payload: CoverLetterDraftRequest,
        service=Depends(get_cover_letter_draft_service),
    ) -> CoverLetterDraftResponse:
        try:
            return await service.enrich(payload)
        except CoverLetterDraftProviderError as exc:
            raise http_error_from_cover_letter_draft_error(exc) from exc

    @application.post("/v1/enrichment/interview-prep", response_model=InterviewPrepResponse)
    @application.post("/api/v1/enrichment/interview-prep", response_model=InterviewPrepResponse)
    async def enrich_interview_prep(
        payload: InterviewPrepRequest,
        service=Depends(get_interview_prep_service),
    ) -> InterviewPrepResponse:
        try:
            return await service.enrich(payload)
        except InterviewPrepProviderError as exc:
            raise http_error_from_interview_prep_error(exc) from exc

    @application.post("/v1/enrichment/weekly-guidance", response_model=WeeklyGuidanceResponse)
    @application.post(
        "/api/v1/enrichment/weekly-guidance",
        response_model=WeeklyGuidanceResponse,
    )
    async def enrich_weekly_guidance(
        payload: WeeklyGuidanceRequest,
        service=Depends(get_weekly_guidance_service),
    ) -> WeeklyGuidanceResponse:
        try:
            return await service.enrich(payload)
        except WeeklyGuidanceProviderError as exc:
            raise http_error_from_weekly_guidance_error(exc) from exc

    return application


app = create_app()
