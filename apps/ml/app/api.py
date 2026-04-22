from fastapi import FastAPI
from fastapi.middleware.cors import CORSMiddleware

from app.engine_api_client import engine_api_base_url
from app.enrichment_routes import register_enrichment_routes
from app.api_models import HealthResponse
from app.scoring_routes import register_scoring_routes
from app.settings import configure_logging, get_runtime_settings


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

    register_scoring_routes(application)
    register_enrichment_routes(application)

    return application


app = create_app()
