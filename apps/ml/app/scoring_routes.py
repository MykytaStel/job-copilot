import logging
import uuid
from pathlib import Path

from fastapi import APIRouter, BackgroundTasks, Depends, FastAPI, HTTPException, Request, status

from app.api_models import (
    BootstrapRequest,
    BootstrapTaskAccepted,
    BootstrapTaskStatus,
    FitAnalyzeRequest,
    FitAnalyzeResponse,
    RerankInvalidateRequest,
    RerankRequest,
    RerankResponse,
    RerankerStatusResponse,
)
from app.engine_api_client import EngineApiResponseError, EngineApiUnavailableError
from app.rerank_service import InvalidRerankRequestError
from app.reranker_bootstrap_service import (
    RerankerBootstrapConflictError,
    RerankerBootstrapServiceError,
    RerankerBootstrapUpstreamHttpError,
    RerankerBootstrapUpstreamUnavailableError,
)
from app.reranker_status import reranker_status
from app.service_dependencies import (
    get_app_services,
    get_fit_analysis_service,
    get_rerank_service,
    get_reranker_bootstrap_service,
)
from app.trained_reranker_config import profile_artifact_path

router = APIRouter()

logger = logging.getLogger(__name__)


def http_error_from_engine_api_client_error(error: Exception) -> HTTPException:
    if isinstance(error, EngineApiResponseError):
        if error.status_code >= 500:
            return HTTPException(status_code=502, detail="upstream error")
        return HTTPException(status_code=error.status_code, detail=error.detail)
    if isinstance(error, EngineApiUnavailableError):
        return HTTPException(status_code=status.HTTP_503_SERVICE_UNAVAILABLE, detail=error.detail)
    logger.error("unhandled engine-api client error type: %s", type(error).__name__)
    return HTTPException(status_code=500, detail="internal server error")


def http_error_from_reranker_bootstrap_error(
    error: RerankerBootstrapServiceError,
) -> HTTPException:
    if isinstance(error, RerankerBootstrapUpstreamHttpError):
        return HTTPException(
            status_code=status.HTTP_502_BAD_GATEWAY,
            detail=f"engine-api error: {error.status_code}",
        )
    if isinstance(error, RerankerBootstrapUpstreamUnavailableError):
        return HTTPException(
            status_code=status.HTTP_503_SERVICE_UNAVAILABLE,
            detail=error.detail,
        )
    if isinstance(error, RerankerBootstrapConflictError):
        return HTTPException(status_code=status.HTTP_409_CONFLICT, detail=str(error))
    logger.error("unhandled bootstrap error type: %s", type(error).__name__)
    return HTTPException(status_code=500, detail="internal server error")


@router.post("/api/v1/fit/analyze", response_model=FitAnalyzeResponse)
async def analyze_fit(
    payload: FitAnalyzeRequest,
    service=Depends(get_fit_analysis_service),
) -> FitAnalyzeResponse:
    try:
        return await service.analyze(payload)
    except (EngineApiResponseError, EngineApiUnavailableError) as exc:
        raise http_error_from_engine_api_client_error(exc) from exc


@router.post("/api/v1/rerank", response_model=RerankResponse)
async def rerank_jobs(
    payload: RerankRequest,
    service=Depends(get_rerank_service),
) -> RerankResponse:
    try:
        return await service.rerank(payload)
    except InvalidRerankRequestError as exc:
        raise HTTPException(
            status_code=status.HTTP_400_BAD_REQUEST,
            detail=str(exc),
        ) from exc
    except (EngineApiResponseError, EngineApiUnavailableError) as exc:
        raise http_error_from_engine_api_client_error(exc) from exc


@router.post("/api/v1/rerank/invalidate", status_code=204)
async def invalidate_rerank_cache(
    payload: RerankInvalidateRequest,
    service=Depends(get_rerank_service),
) -> None:
    service.invalidate(payload.profile_id)


@router.post("/api/v1/reranker/bootstrap", response_model=BootstrapTaskAccepted, status_code=202)
async def bootstrap_reranker(
    request: Request,
    payload: BootstrapRequest,
    background_tasks: BackgroundTasks,
    service=Depends(get_reranker_bootstrap_service),
) -> BootstrapTaskAccepted:
    task_id = str(uuid.uuid4())
    task_store = get_app_services(request).task_store
    task_store.create(task_id=task_id, profile_id=payload.profile_id)

    async def _run_bootstrap() -> None:
        running_status = None

        def _mark_running() -> None:
            nonlocal running_status
            if running_status is None:
                running_status = task_store.mark_running(task_id)

        try:
            result = await service.bootstrap(payload, on_started=_mark_running)
            task_store.mark_completed(task_id, result)
        except RerankerBootstrapServiceError as exc:
            http_exc = http_error_from_reranker_bootstrap_error(exc)
            task_store.mark_failed(
                task_id,
                profile_id=payload.profile_id,
                error=str(http_exc.detail),
                artifact_path=str(profile_artifact_path(payload.profile_id)),
                started_at=running_status.started_at if running_status is not None else None,
            )
        except Exception as exc:
            logger.error("bootstrap task failed unexpectedly: %s", exc, exc_info=True)
            task_store.mark_failed(
                task_id,
                profile_id=payload.profile_id,
                error="internal error",
                artifact_path=str(profile_artifact_path(payload.profile_id)),
                started_at=running_status.started_at if running_status is not None else None,
            )

    background_tasks.add_task(_run_bootstrap)
    return BootstrapTaskAccepted(task_id=task_id)
@router.get("/api/v1/reranker/status", response_model=RerankerStatusResponse)
async def get_reranker_status() -> RerankerStatusResponse:
    return RerankerStatusResponse(**reranker_status())


@router.get("/api/v1/reranker/bootstrap/{task_id}", response_model=BootstrapTaskStatus)
async def get_bootstrap_status(task_id: str, request: Request) -> BootstrapTaskStatus:
    task_store = get_app_services(request).task_store
    task: BootstrapTaskStatus | None = task_store.get(task_id)
    if task is None:
        raise HTTPException(status_code=404, detail="task not found")
    return task


def register_scoring_routes(application: FastAPI) -> None:
    application.include_router(router)
