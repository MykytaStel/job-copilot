from fastapi import APIRouter, Depends, FastAPI

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
from app.cv_tailoring import (
    CvTailoringProviderError,
    CvTailoringRequest,
    CvTailoringResponse,
    http_error_from_cv_tailoring_error,
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
from app.service_dependencies import (
    get_application_coach_service,
    get_cover_letter_draft_service,
    get_cv_tailoring_service,
    get_interview_prep_service,
    get_job_fit_explanation_service,
    get_profile_insights_service,
    get_weekly_guidance_service,
)
from app.weekly_guidance import (
    WeeklyGuidanceProviderError,
    WeeklyGuidanceRequest,
    WeeklyGuidanceResponse,
    http_error_from_weekly_guidance_error,
)

router = APIRouter()


@router.post("/v1/enrichment/profile-insights", response_model=ProfileInsightsResponse)
@router.post("/api/v1/enrichment/profile-insights", response_model=ProfileInsightsResponse)
async def enrich_profile_insights(
    payload: LlmContextRequest,
    service=Depends(get_profile_insights_service),
) -> ProfileInsightsResponse:
    try:
        return await service.enrich(payload)
    except ProfileInsightsProviderError as exc:
        raise http_error_from_provider_error(exc) from exc


@router.post(
    "/v1/enrichment/job-fit-explanation",
    response_model=JobFitExplanationResponse,
)
@router.post(
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


@router.post("/v1/enrichment/application-coach", response_model=ApplicationCoachResponse)
@router.post(
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


@router.post(
    "/v1/enrichment/cover-letter-draft",
    response_model=CoverLetterDraftResponse,
)
@router.post(
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


@router.post("/v1/enrichment/interview-prep", response_model=InterviewPrepResponse)
@router.post("/api/v1/enrichment/interview-prep", response_model=InterviewPrepResponse)
async def enrich_interview_prep(
    payload: InterviewPrepRequest,
    service=Depends(get_interview_prep_service),
) -> InterviewPrepResponse:
    try:
        return await service.enrich(payload)
    except InterviewPrepProviderError as exc:
        raise http_error_from_interview_prep_error(exc) from exc


@router.post("/v1/enrichment/weekly-guidance", response_model=WeeklyGuidanceResponse)
@router.post(
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


@router.post("/v1/cv-tailoring", response_model=CvTailoringResponse)
@router.post("/api/v1/cv-tailoring", response_model=CvTailoringResponse)
async def cv_tailoring(
    payload: CvTailoringRequest,
    service=Depends(get_cv_tailoring_service),
) -> CvTailoringResponse:
    try:
        return await service.enrich(payload)
    except CvTailoringProviderError as exc:
        raise http_error_from_cv_tailoring_error(exc) from exc


def register_enrichment_routes(application: FastAPI) -> None:
    application.include_router(router)