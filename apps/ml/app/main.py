import asyncio
from functools import lru_cache

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
from app.application_coach_service import ApplicationCoachService
from app.cover_letter_draft import (
    CoverLetterDraftProviderError,
    CoverLetterDraftRequest,
    CoverLetterDraftResponse,
    http_error_from_cover_letter_draft_error,
)
from app.cover_letter_draft_service import CoverLetterDraftService
from app.engine_api_client import (
    EngineApiClient,
    EngineJobLifecycle,
    EngineProfile,
    engine_api_base_url,
    engine_api_timeout_seconds,
)
from app.interview_prep import (
    InterviewPrepProviderError,
    InterviewPrepRequest,
    InterviewPrepResponse,
    http_error_from_interview_prep_error,
)
from app.interview_prep_service import InterviewPrepService
from app.job_fit_explanation import (
    JobFitExplanationProviderError,
    JobFitExplanationRequest,
    JobFitExplanationResponse,
    http_error_from_job_fit_explanation_error,
)
from app.job_fit_explanation_service import JobFitExplanationService
from app.llm_provider import (
    build_application_coach_provider,
    build_cover_letter_draft_provider,
    build_interview_prep_provider,
    build_job_fit_explanation_provider,
    build_profile_insights_provider,
    build_weekly_guidance_provider,
)
from app.profile_insights import (
    LlmContextRequest,
    ProfileInsightsProviderError,
    ProfileInsightsResponse,
    http_error_from_provider_error,
)
from app.profile_insights_service import ProfileInsightsService
from app.bootstrap_training import DEFAULT_MODEL_PATH, bootstrap_and_retrain
from app.text_normalization import normalize_term_for_output, normalize_text, tokenize
from app.weekly_guidance import (
    WeeklyGuidanceProviderError,
    WeeklyGuidanceRequest,
    WeeklyGuidanceResponse,
    http_error_from_weekly_guidance_error,
)
from app.weekly_guidance_service import WeeklyGuidanceService

app = FastAPI(
    title="job-copilot-ml",
    version="0.1.0",
    description="Read-only ML sidecar over canonical engine-api data.",
)

app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_methods=["*"],
    allow_headers=["*"],
)

SENIORITY_TERMS = {
    "intern",
    "junior",
    "middle",
    "mid",
    "senior",
    "lead",
    "staff",
    "principal",
    "head",
    "director",
}


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


def unique_preserving_order(values: list[str]) -> list[str]:
    seen: set[str] = set()
    result: list[str] = []
    for value in values:
        if value in seen:
            continue
        seen.add(value)
        result.append(value)
    return result


def term_key(value: str) -> str:
    return normalize_text(value)


def term_weight(value: str) -> float:
    normalized = term_key(value)
    if not normalized:
        return 0.0
    if "_" in normalized:
        return 1.35
    return 1.0


def indexed_terms(values: list[str]) -> list[tuple[str, str, float]]:
    indexed: list[tuple[str, str, float]] = []
    seen: set[str] = set()

    for value in values:
        key = term_key(value)
        if not key or key in SENIORITY_TERMS or key in seen:
            continue

        seen.add(key)
        indexed.append((key, normalize_term_for_output(value), term_weight(value)))

    return indexed


CONTACT_NOISE_TERMS = {
    "email",
    "mail",
    "phone",
    "telegram",
    "linkedin",
    "contact",
    "contacts",
}


def _profile_identity_terms(profile: EngineProfile) -> set[str]:
    return set(tokenize(profile.name, profile.email, profile.location))


def _is_profile_noise_term(term: str, identity_terms: set[str]) -> bool:
    normalized = term_key(term)
    if not normalized:
        return True
    if normalized in identity_terms or normalized in CONTACT_NOISE_TERMS:
        return True
    return normalized.isdigit() and len(normalized) >= 7


def _append_analysis_terms(terms: list[str], values: list[str]) -> None:
    for value in values:
        normalized = normalize_term_for_output(value)
        if normalized:
            terms.append(normalized)


def profile_terms(profile: EngineProfile) -> list[str]:
    analysis = profile.analysis
    if analysis:
        terms = tokenize(analysis.primary_role)
        _append_analysis_terms(terms, analysis.skills)
        _append_analysis_terms(terms, analysis.keywords)
    else:
        identity_terms = _profile_identity_terms(profile)
        terms = [
            term
            for term in tokenize(profile.raw_text)
            if not _is_profile_noise_term(term, identity_terms)
        ]
    return unique_preserving_order(terms)[:40]


def job_terms(job: EngineJobLifecycle) -> list[str]:
    return unique_preserving_order(
        tokenize(
            job.title,
            job.company_name,
            job.location,
            job.remote_type,
            job.seniority,
            job.description_text,
            job.presentation.summary,
            job.presentation.location_label,
            job.presentation.work_mode_label,
        )
    )


def overlap(profile_values: list[str], job_values: list[str]) -> list[str]:
    job_keys = {key for key, _, _ in indexed_terms(job_values)}
    return [display for key, display, _ in indexed_terms(profile_values) if key in job_keys]


def build_evidence(
    profile: EngineProfile,
    job: EngineJobLifecycle,
    matched_terms: list[str],
    title_matches: list[str],
) -> list[str]:
    evidence: list[str] = []

    if title_matches:
        evidence.append(f"title overlap: {', '.join(title_matches[:5])}")
    if matched_terms:
        evidence.append(f"shared terms: {', '.join(matched_terms[:8])}")

    seniority = profile.analysis.seniority if profile.analysis else ""
    if seniority and job.seniority and seniority.lower() == job.seniority.lower():
        evidence.append(f"seniority match: {job.seniority}")

    if job.lifecycle_stage != "active":
        evidence.append(f"lifecycle: {job.lifecycle_stage}")
    if job.presentation.work_mode_label:
        evidence.append(f"job mode: {job.presentation.work_mode_label}")
    elif job.remote_type:
        evidence.append(f"job mode: {job.remote_type}")
    if job.presentation.location_label:
        evidence.append(f"location: {job.presentation.location_label}")
    elif job.primary_variant:
        evidence.append(f"source: {job.primary_variant.source}")

    return evidence[:4]


def score_job(
    profile: EngineProfile, job: EngineJobLifecycle
) -> tuple[int, list[str], list[str], list[str]]:
    profile_values = profile_terms(profile)
    job_values = job_terms(job)
    profile_index = indexed_terms(profile_values)
    job_index = indexed_terms(job_values)

    if not profile_index:
        return 0, [], [], ["profile has no usable terms yet"]

    job_keys = {key for key, _, _ in job_index}
    matched_terms = [display for key, display, _ in profile_index if key in job_keys]
    title_keys = {key for key, _, _ in indexed_terms(tokenize(job.title))}
    title_matches = [display for key, display, _ in profile_index if key in title_keys]

    total_weight = sum(weight for _, _, weight in profile_index)
    matched_weight = sum(weight for key, _, weight in profile_index if key in job_keys)
    title_weight = sum(weight for key, _, weight in profile_index if key in title_keys)

    overlap_ratio = matched_weight / total_weight if total_weight else 0.0
    title_bonus = min(int(round(title_weight * 8)), 24)

    seniority_bonus = 0
    if profile.analysis and profile.analysis.seniority and job.seniority:
        if profile.analysis.seniority.lower() == job.seniority.lower():
            seniority_bonus = 10

    active_bonus = 5 if job.is_active else 0
    lifecycle_bonus = 3 if job.lifecycle_stage == "reactivated" else 0
    score = min(
        int(round(overlap_ratio * 70)) + title_bonus + seniority_bonus + active_bonus + lifecycle_bonus,
        100,
    )

    missing_terms = [display for key, display, _ in profile_index if key not in job_keys][:8]
    evidence = build_evidence(profile, job, matched_terms, title_matches)
    return score, matched_terms[:10], missing_terms, evidence


class BootstrapRequest(BaseModel):
    profile_id: str = Field(min_length=1)
    min_examples: int = Field(default=30, ge=1)


class BootstrapResponse(BaseModel):
    retrained: bool
    example_count: int
    reason: str | None = None
    model_path: str | None = None
    training: dict | None = None


@app.get("/health", response_model=HealthResponse)
async def health() -> HealthResponse:
    return HealthResponse(
        status="ok",
        service="ml",
        engine_api_base_url=engine_api_base_url(),
    )


@app.post("/api/v1/fit/analyze", response_model=FitAnalyzeResponse)
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


@app.post("/api/v1/rerank", response_model=RerankResponse)
async def rerank_jobs(payload: RerankRequest) -> RerankResponse:
    unique_job_ids = unique_preserving_order([job_id.strip() for job_id in payload.job_ids if job_id.strip()])
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


@app.post("/v1/enrichment/profile-insights", response_model=ProfileInsightsResponse)
@app.post("/api/v1/enrichment/profile-insights", response_model=ProfileInsightsResponse)
async def enrich_profile_insights(
    payload: LlmContextRequest,
    service: ProfileInsightsService = Depends(get_profile_insights_service),
) -> ProfileInsightsResponse:
    try:
        return await service.enrich(payload)
    except ProfileInsightsProviderError as exc:
        raise http_error_from_provider_error(exc) from exc


@app.post("/v1/enrichment/job-fit-explanation", response_model=JobFitExplanationResponse)
@app.post("/api/v1/enrichment/job-fit-explanation", response_model=JobFitExplanationResponse)
async def enrich_job_fit_explanation(
    payload: JobFitExplanationRequest,
    service: JobFitExplanationService = Depends(get_job_fit_explanation_service),
) -> JobFitExplanationResponse:
    try:
        return await service.enrich(payload)
    except JobFitExplanationProviderError as exc:
        raise http_error_from_job_fit_explanation_error(exc) from exc


@app.post("/v1/enrichment/application-coach", response_model=ApplicationCoachResponse)
@app.post("/api/v1/enrichment/application-coach", response_model=ApplicationCoachResponse)
async def enrich_application_coach(
    payload: ApplicationCoachRequest,
    service: ApplicationCoachService = Depends(get_application_coach_service),
) -> ApplicationCoachResponse:
    try:
        return await service.enrich(payload)
    except ApplicationCoachProviderError as exc:
        raise http_error_from_application_coach_error(exc) from exc


@app.post("/v1/enrichment/cover-letter-draft", response_model=CoverLetterDraftResponse)
@app.post("/api/v1/enrichment/cover-letter-draft", response_model=CoverLetterDraftResponse)
async def enrich_cover_letter_draft(
    payload: CoverLetterDraftRequest,
    service: CoverLetterDraftService = Depends(get_cover_letter_draft_service),
) -> CoverLetterDraftResponse:
    try:
        return await service.enrich(payload)
    except CoverLetterDraftProviderError as exc:
        raise http_error_from_cover_letter_draft_error(exc) from exc


@app.post("/v1/enrichment/interview-prep", response_model=InterviewPrepResponse)
@app.post("/api/v1/enrichment/interview-prep", response_model=InterviewPrepResponse)
async def enrich_interview_prep(
    payload: InterviewPrepRequest,
    service: InterviewPrepService = Depends(get_interview_prep_service),
) -> InterviewPrepResponse:
    try:
        return await service.enrich(payload)
    except InterviewPrepProviderError as exc:
        raise http_error_from_interview_prep_error(exc) from exc


@app.post("/v1/enrichment/weekly-guidance", response_model=WeeklyGuidanceResponse)
@app.post("/api/v1/enrichment/weekly-guidance", response_model=WeeklyGuidanceResponse)
async def enrich_weekly_guidance(
    payload: WeeklyGuidanceRequest,
    service: WeeklyGuidanceService = Depends(get_weekly_guidance_service),
) -> WeeklyGuidanceResponse:
    try:
        return await service.enrich(payload)
    except WeeklyGuidanceProviderError as exc:
        raise http_error_from_weekly_guidance_error(exc) from exc


@app.post("/api/v1/reranker/bootstrap", response_model=BootstrapResponse)
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
