import asyncio

import httpx
from fastapi import Depends, FastAPI, HTTPException, status
from fastapi.middleware.cors import CORSMiddleware
from pydantic import BaseModel, Field

from app.application_coach import (
    ApplicationCoachProviderError,
    ApplicationCoachRequest,
    ApplicationCoachResponse,
    http_error_from_application_coach_error,
)
from app.bootstrap_training import DEFAULT_MODEL_PATH, bootstrap_and_retrain
from app.cover_letter_draft import (
    CoverLetterDraftProviderError,
    CoverLetterDraftRequest,
    CoverLetterDraftResponse,
    http_error_from_cover_letter_draft_error,
)
from app.engine_api_client import (
    EngineApiClient,
    engine_api_base_url,
    engine_api_timeout_seconds,
)
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
from app.profile_insights import (
    LlmContextRequest,
    ProfileInsightsProviderError,
    ProfileInsightsResponse,
    http_error_from_provider_error,
)
from app.scoring import score_job, unique_preserving_order
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


class HealthResponse(BaseModel):
    status: str
    service: str
    engine_api_base_url: str


class FitAnalyzeRequest(BaseModel):
    profile_id: str = Field(min_length=1)
    job_id: str = Field(min_length=1)


class FitAnalyzeResponse(BaseModel):
    profile_id: str
    job_id: str
    score: int
    matched_terms: list[str]
    missing_terms: list[str]
    evidence: list[str]


class RerankRequest(BaseModel):
    profile_id: str = Field(min_length=1)
    job_ids: list[str] = Field(min_length=1, max_length=50)


class RerankedJob(BaseModel):
    job_id: str
    title: str
    company_name: str
    score: int
    matched_terms: list[str]
    evidence: list[str]


class RerankResponse(BaseModel):
    profile_id: str
    jobs: list[RerankedJob]


class BootstrapRequest(BaseModel):
    profile_id: str = Field(min_length=1)
    min_examples: int = Field(default=30, ge=1)


class BootstrapResponse(BaseModel):
    retrained: bool
    example_count: int
    reason: str | None = None
    model_path: str | None = None
    training: dict | None = None


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

    @application.get("/health", response_model=HealthResponse)
    async def health() -> HealthResponse:
        return HealthResponse(
            status="ok",
            service="ml",
            engine_api_base_url=engine_api_base_url(),
        )

    @application.post("/api/v1/fit/analyze", response_model=FitAnalyzeResponse)
    async def analyze_fit(payload: FitAnalyzeRequest) -> FitAnalyzeResponse:
        timeout = httpx.Timeout(engine_api_timeout_seconds())
        async with httpx.AsyncClient(timeout=timeout) as client:
            engine_api = EngineApiClient(client)
            profile = await engine_api.fetch_profile(payload.profile_id)
            job = await engine_api.fetch_job_lifecycle(payload.job_id)

        score, matched_terms, missing_terms, evidence = score_job(profile, job)
        return FitAnalyzeResponse(
            profile_id=payload.profile_id,
            job_id=payload.job_id,
            score=score,
            matched_terms=matched_terms,
            missing_terms=missing_terms,
            evidence=evidence,
        )

    @application.post("/api/v1/rerank", response_model=RerankResponse)
    async def rerank_jobs(payload: RerankRequest) -> RerankResponse:
        unique_job_ids = unique_preserving_order(
            [job_id.strip() for job_id in payload.job_ids if job_id.strip()]
        )
        if not unique_job_ids:
            raise HTTPException(
                status_code=status.HTTP_400_BAD_REQUEST,
                detail="job_ids must contain at least one non-empty id",
            )

        timeout = httpx.Timeout(engine_api_timeout_seconds())
        async with httpx.AsyncClient(timeout=timeout) as client:
            engine_api = EngineApiClient(client)
            profile = await engine_api.fetch_profile(payload.profile_id)
            jobs = await asyncio.gather(
                *(engine_api.fetch_job_lifecycle(job_id) for job_id in unique_job_ids)
            )

        ranked_jobs: list[RerankedJob] = []
        for job in jobs:
            score, matched_terms, _, evidence = score_job(profile, job)
            ranked_jobs.append(
                RerankedJob(
                    job_id=job.id,
                    title=job.title,
                    company_name=job.company_name,
                    score=score,
                    matched_terms=matched_terms,
                    evidence=evidence,
                )
            )

        ranked_jobs.sort(key=lambda item: (-item.score, item.title.lower(), item.job_id))
        return RerankResponse(profile_id=payload.profile_id, jobs=ranked_jobs)

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

    @application.post("/api/v1/reranker/bootstrap", response_model=BootstrapResponse)
    async def bootstrap_reranker(payload: BootstrapRequest) -> BootstrapResponse:
        try:
            result = await bootstrap_and_retrain(
                profile_id=payload.profile_id,
                min_examples=payload.min_examples,
                model_path=DEFAULT_MODEL_PATH,
            )
        except httpx.HTTPStatusError as exc:
            raise HTTPException(
                status_code=status.HTTP_502_BAD_GATEWAY,
                detail=f"engine-api error: {exc.response.status_code}",
            ) from exc
        except httpx.HTTPError as exc:
            raise HTTPException(
                status_code=status.HTTP_503_SERVICE_UNAVAILABLE,
                detail=f"engine-api unreachable: {exc}",
            ) from exc
        return BootstrapResponse(**result)

    return application


app = create_app()
