from fastapi import APIRouter, Depends, FastAPI, HTTPException, status

from app.api_models import (
    BootstrapRequest,
    BootstrapResponse,
    FitAnalyzeRequest,
    FitAnalyzeResponse,
    RerankRequest,
    RerankResponse,
)
from app.engine_api_client import EngineApiResponseError, EngineApiUnavailableError
from app.rerank_service import InvalidRerankRequestError
from app.reranker_bootstrap_service import (
    RerankerBootstrapServiceError,
    RerankerBootstrapUpstreamHttpError,
    RerankerBootstrapUpstreamUnavailableError,
)
from app.service_dependencies import (
    get_fit_analysis_service,
    get_rerank_service,
    get_reranker_bootstrap_service,
)

router = APIRouter()


def http_error_from_engine_api_client_error(error: Exception) -> HTTPException:
    if isinstance(error, EngineApiResponseError):
        return HTTPException(status_code=error.status_code, detail=error.detail)
    if isinstance(error, EngineApiUnavailableError):
        return HTTPException(status_code=status.HTTP_503_SERVICE_UNAVAILABLE, detail=error.detail)
    raise TypeError("unsupported engine-api client error")


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
    raise TypeError("unsupported reranker bootstrap error")


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


@router.post("/api/v1/reranker/bootstrap", response_model=BootstrapResponse)
async def bootstrap_reranker(
    payload: BootstrapRequest,
    service=Depends(get_reranker_bootstrap_service),
) -> BootstrapResponse:
    try:
        return await service.bootstrap(payload)
    except RerankerBootstrapServiceError as exc:
        raise http_error_from_reranker_bootstrap_error(exc) from exc


def register_scoring_routes(application: FastAPI) -> None:
    application.include_router(router)
