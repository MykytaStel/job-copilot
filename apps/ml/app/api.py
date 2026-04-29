import asyncio
import logging
from contextlib import asynccontextmanager

from fastapi import FastAPI, Request, Response
from fastapi.middleware.cors import CORSMiddleware
from fastapi.responses import JSONResponse
from time import perf_counter

from app.api_models import (
    DatabaseComponent,
    HealthResponse,
    IngestionComponent,
    MlSidecarComponent,
    ReadyComponents,
    ReadyResponse,
)
from app.core.runtime import build_app_services, close_app_services
from app.engine_api_client import engine_api_base_url
from app.enrichment_routes import register_enrichment_routes
from app.scoring_routes import register_scoring_routes
from app.settings import configure_logging, get_runtime_settings

logger = logging.getLogger(__name__)


@asynccontextmanager
async def lifespan(application: FastAPI):
    settings = get_runtime_settings()
    settings.validate_startup_security()

    services = await build_app_services(settings)
    application.state.services = services

    logger.info("ML sidecar started")

    try:
        yield
    finally:
        await close_app_services(services)
        logger.info("ML sidecar shutdown")


def create_app() -> FastAPI:
    configure_logging()
    settings = get_runtime_settings()

    application = FastAPI(
        title="job-copilot-ml",
        version="0.1.0",
        description="Read-only ML sidecar over canonical engine-api data.",
        lifespan=lifespan,
    )
    application.state.runtime_settings = settings

    if settings.internal_token:
        @application.middleware("http")
        async def check_internal_token(request: Request, call_next):
            if request.url.path in {"/health", "/ready"}:
                return await call_next(request)
            token = request.headers.get("X-Internal-Token", "")
            if not settings.internal_token:
                  return JSONResponse({"detail": "internal token is not configured"}, status_code=503)
            if token != settings.internal_token:
                  return JSONResponse({"detail": "unauthorized"}, status_code=401)
            return await call_next(request)

    @application.middleware("http")
    async def record_request_duration(request: Request, call_next):
        started_at = perf_counter()
        request_id = request.headers.get("x-request-id", "-")
        response = await call_next(request)
        response.headers["x-request-id"] = request_id
        logger.info(
            "request completed",
            extra={
                "path": request.url.path,
                "method": request.method,
                "status_code": response.status_code,
                "duration_ms": round((perf_counter() - started_at) * 1000, 2),
                "request_id": request_id,
            },
        )
        return response

    application.add_middleware(
        CORSMiddleware,
        allow_origins=list(settings.cors_allowed_origins),
        allow_methods=["*"],
        allow_headers=["*"],
    )

    @application.exception_handler(Exception)
    async def unhandled_exception_handler(request: Request, exc: Exception) -> JSONResponse:
        logger.error("unhandled error: %s", type(exc).__name__, exc_info=True)
        return JSONResponse({"detail": "internal server error"}, status_code=500)

    @application.get("/health", response_model=HealthResponse)
    async def health() -> HealthResponse:
        return HealthResponse(
            status="ok",
            service="ml",
            engine_api_base_url=engine_api_base_url(),
            llm_provider=settings.llm_provider,
        )

    @application.get("/ready", response_model=ReadyResponse)
    async def ready(request: Request, http_response: Response) -> ReadyResponse:
        services = request.app.state.services
        request_id = request.headers.get("x-request-id")
        timeout_seconds = min(settings.ready_timeout_seconds, 0.5)
        engine_ready_url = f"{engine_api_base_url()}/ready"
        headers = {"x-request-id": request_id} if request_id else None

        try:
            engine_response = await asyncio.wait_for(
                services._http_client.get(
                    engine_ready_url,
                    headers=headers,
                    timeout=timeout_seconds,
                ),
                timeout=timeout_seconds,
            )
            payload = engine_response.json() if engine_response.content else {}
            engine_components = payload.get("components", {})
            database_payload = engine_components.get("database", {})
            ingestion_payload = engine_components.get("ingestion", {})
            database = DatabaseComponent(
                status=database_payload.get("status", "error"),
                latency_ms=database_payload.get("latency_ms", 0),
            )
            ingestion = IngestionComponent(
                status=ingestion_payload.get("status", "stale"),
                last_run_at=ingestion_payload.get("last_run_at"),
            )
        except asyncio.TimeoutError:
            logger.warning(
                "engine-api readiness probe timed out",
                extra={"timeout_seconds": timeout_seconds, "request_id": request_id or "-"},
            )
            database = DatabaseComponent(status="error", latency_ms=0)
            ingestion = IngestionComponent(status="stale", last_run_at=None)
        except Exception as exc:
            logger.warning(
                "engine-api readiness probe failed",
                extra={"error": type(exc).__name__, "detail": str(exc), "request_id": request_id or "-"},
            )
            database = DatabaseComponent(status="error", latency_ms=0)
            ingestion = IngestionComponent(status="stale", last_run_at=None)

        ml_sidecar = MlSidecarComponent(status="ok")
        status_value = (
            "not_ready"
            if database.status == "error"
            else "degraded"
            if ingestion.status != "ok"
            else "ready"
        )
        if status_value == "not_ready":
            http_response.status_code = 503
        return ReadyResponse(
            status=status_value,
            components=ReadyComponents(
                database=database,
                ml_sidecar=ml_sidecar,
                ingestion=ingestion,
            ),
        )

    register_scoring_routes(application)
    register_enrichment_routes(application)

    return application


app = create_app()
