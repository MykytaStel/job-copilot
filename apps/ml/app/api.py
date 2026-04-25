import asyncio
import logging
from contextlib import asynccontextmanager
import token

from fastapi import FastAPI, Request
from fastapi.middleware.cors import CORSMiddleware
from fastapi.responses import JSONResponse
from time import perf_counter

from app.api_models import HealthResponse, ReadyCheck, ReadyResponse
from app.core.runtime import build_app_services, close_app_services
from app.engine_api_client import EngineApiClient, engine_api_base_url
from app.enrichment_routes import register_enrichment_routes
from app.scoring_routes import register_scoring_routes
from app.trained_reranker_config import get_profile_artifacts_dir, get_trained_reranker_model_path
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
    async def ready(request: Request) -> ReadyResponse:
        services = request.app.state.services
        checks: list[ReadyCheck] = []
        request_id = request.headers.get("x-request-id")

        try:
            engine_api = EngineApiClient(services._http_client, request_id=request_id)
            await asyncio.wait_for(
                engine_api.probe_health(),
                timeout=settings.ready_timeout_seconds,
            )
            checks.append(ReadyCheck(name="engine_api", status="ok"))
        except asyncio.TimeoutError:
            logger.warning(
                "engine-api health probe timed out",
                extra={"timeout_seconds": settings.ready_timeout_seconds, "request_id": request_id or "-"},
            )
            checks.append(
                ReadyCheck(name="engine_api", status="degraded", detail="engine-api health probe timed out")
            )
        except Exception as exc:
            logger.warning(
                "engine-api health probe failed",
                extra={"error": type(exc).__name__, "detail": str(exc), "request_id": request_id or "-"},
            )
            checks.append(
                ReadyCheck(name="engine_api", status="degraded", detail=f"engine-api unavailable: {type(exc).__name__}")
            )

        if services.enrichment_provider_error:
            checks.append(
                ReadyCheck(
                    name="enrichment_provider",
                    status="degraded",
                    detail="provider unavailable",
                )
            )
        else:
            checks.append(ReadyCheck(name="enrichment_provider", status="ok"))

        artifact_path = get_trained_reranker_model_path()
        if artifact_path.exists():
            checks.append(ReadyCheck(name="runtime_artifact", status="ok"))
        else:
            checks.append(
                ReadyCheck(
                    name="runtime_artifact",
                    status="degraded",
                    detail="no promoted runtime artifact",
                )
            )

        artifacts_dir = get_profile_artifacts_dir()
        if artifacts_dir.exists():
            checks.append(ReadyCheck(name="profile_artifacts_dir", status="ok"))
        else:
            checks.append(
                ReadyCheck(
                    name="profile_artifacts_dir",
                    status="degraded",
                    detail="profile artifact directory missing",
                )
            )

        bootstrap_runtime = services.reranker_bootstrap_service.runtime_snapshot()
        task_counts = services.task_store.status_counts()
        queued_jobs = task_counts["accepted"] + max(
            0,
            task_counts["running"] - bootstrap_runtime["active_jobs"],
        )
        bootstrap_saturated = (
            bootstrap_runtime["active_jobs"] >= bootstrap_runtime["max_concurrent_jobs"]
            and queued_jobs > 0
        )
        checks.append(
            ReadyCheck(
                name="bootstrap_runtime",
                status="degraded" if bootstrap_saturated else "ok",
                detail=(
                    "active="
                    f"{bootstrap_runtime['active_jobs']}/{bootstrap_runtime['max_concurrent_jobs']}, "
                    f"available_slots={bootstrap_runtime['available_slots']}, "
                    f"queued={queued_jobs}, "
                    f"accepted={task_counts['accepted']}, running={task_counts['running']}, "
                    f"completed={task_counts['completed']}, failed={task_counts['failed']}"
                ),
            )
        )

        status_value = "ok" if all(check.status == "ok" for check in checks) else "degraded"
        return ReadyResponse(status=status_value, service="ml", checks=checks)

    register_scoring_routes(application)
    register_enrichment_routes(application)

    return application


app = create_app()
