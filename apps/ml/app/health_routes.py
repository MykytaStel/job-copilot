from __future__ import annotations

import asyncio
import logging

from fastapi import APIRouter, FastAPI, Request, Response
from fastapi.responses import PlainTextResponse

from app.api_models import (
    DatabaseComponent,
    HealthResponse,
    IngestionComponent,
    MlSidecarComponent,
    ReadyComponents,
    ReadyResponse,
)
from app.engine_api_client import engine_api_base_url
from app.service_dependencies import get_app_services
from app.settings import RuntimeSettings

logger = logging.getLogger(__name__)


def register_health_routes(application: FastAPI, settings: RuntimeSettings) -> None:
    router = APIRouter()

    @router.get("/health", response_model=HealthResponse)
    async def health() -> HealthResponse:
        return HealthResponse(
            status="ok",
            service="ml",
            engine_api_base_url=engine_api_base_url(),
            llm_provider=settings.llm_provider,
        )

    @router.get("/ready", response_model=ReadyResponse)
    async def ready(request: Request, http_response: Response) -> ReadyResponse:
        services = get_app_services(request)
        request_id = request.headers.get("x-request-id")
        timeout_seconds = min(settings.ready_timeout_seconds, 0.5)
        engine_ready_url = f"{engine_api_base_url()}/ready"
        headers = {"x-request-id": request_id} if request_id else None

        try:
            engine_response = await asyncio.wait_for(
                services.http_client.get(
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
        except TimeoutError:
            logger.warning(
                "engine-api readiness probe timed out",
                extra={"timeout_seconds": timeout_seconds, "request_id": request_id or "-"},
            )
            database = DatabaseComponent(status="error", latency_ms=0)
            ingestion = IngestionComponent(status="stale", last_run_at=None)
        except Exception as exc:
            logger.warning(
                "engine-api readiness probe failed",
                extra={
                    "error": type(exc).__name__,
                    "detail": str(exc),
                    "request_id": request_id or "-",
                },
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

    @router.get("/metrics", response_class=PlainTextResponse)
    async def metrics(request: Request) -> PlainTextResponse:
        services = get_app_services(request)
        task_counts = services.task_store.status_counts()
        provider_available = 1 if services.enrichment_provider_error is None else 0
        lines = [
            "# HELP job_copilot_ml_info Static ML sidecar info.",
            "# TYPE job_copilot_ml_info gauge",
            f'job_copilot_ml_info{{llm_provider="{settings.llm_provider}"}} 1',
            "# HELP job_copilot_ml_enrichment_provider_available Whether enrichment provider built at startup.",
            "# TYPE job_copilot_ml_enrichment_provider_available gauge",
            f"job_copilot_ml_enrichment_provider_available {provider_available}",
            "# HELP job_copilot_ml_bootstrap_tasks Current bootstrap task status counts.",
            "# TYPE job_copilot_ml_bootstrap_tasks gauge",
        ]
        lines.extend(
            f'job_copilot_ml_bootstrap_tasks{{status="{status}"}} {count}'
            for status, count in sorted(task_counts.items())
        )
        return PlainTextResponse("\n".join(lines) + "\n")

    application.include_router(router)
