import asyncio
import logging
from time import perf_counter

import httpx
from fastapi import APIRouter, FastAPI, HTTPException, status

from app.bootstrap_training import DEFAULT_MODEL_PATH, bootstrap_and_retrain
from app.engine_api_client import EngineApiClient, engine_api_timeout_seconds
from app.scoring import score_job, unique_preserving_order
from app.api_models import (
    BootstrapRequest,
    BootstrapResponse,
    FitAnalyzeRequest,
    FitAnalyzeResponse,
    RerankRequest,
    RerankResponse,
    RerankedJob,
)

router = APIRouter()
logger = logging.getLogger(__name__)


@router.post("/api/v1/fit/analyze", response_model=FitAnalyzeResponse)
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


@router.post("/api/v1/rerank", response_model=RerankResponse)
async def rerank_jobs(payload: RerankRequest) -> RerankResponse:
    started_at = perf_counter()
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

    logger.info(
        "rerank completed",
        extra={
            "profile_id": payload.profile_id,
            "requested_job_ids": len(payload.job_ids),
            "deduped_job_ids": len(unique_job_ids),
            "returned_jobs": len(ranked_jobs),
            "duration_ms": round((perf_counter() - started_at) * 1000, 2),
        },
    )

    return RerankResponse(profile_id=payload.profile_id, jobs=ranked_jobs)


@router.post("/api/v1/reranker/bootstrap", response_model=BootstrapResponse)
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


def register_scoring_routes(application: FastAPI) -> None:
    application.include_router(router)
