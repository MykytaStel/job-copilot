import logging
from contextlib import asynccontextmanager

from fastapi import FastAPI, Request
from fastapi.middleware.cors import CORSMiddleware
from fastapi.responses import JSONResponse

from app.core.runtime import build_app_services, close_app_services
from app.enrichment_routes import register_enrichment_routes
from app.health_routes import register_health_routes
from app.middleware import register_runtime_middleware
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

    register_runtime_middleware(application, settings)

    application.add_middleware(
        CORSMiddleware,
        allow_origins=list(settings.cors_allowed_origins),
        allow_methods=["GET", "POST", "PUT", "PATCH", "DELETE", "OPTIONS"],
        allow_headers=["Authorization", "Content-Type", "Accept", "X-Internal-Token", "X-Request-ID"],
    )

    @application.exception_handler(Exception)
    async def unhandled_exception_handler(request: Request, exc: Exception) -> JSONResponse:
        logger.error("unhandled error: %s", type(exc).__name__, exc_info=True)
        return JSONResponse({"detail": "internal server error"}, status_code=500)

    register_health_routes(application, settings)
    register_scoring_routes(application)
    register_enrichment_routes(application)

    return application


app = create_app()
