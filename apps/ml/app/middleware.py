from __future__ import annotations

import logging
from time import perf_counter

from fastapi import FastAPI, Request
from fastapi.responses import JSONResponse

from app.settings import RuntimeSettings

logger = logging.getLogger(__name__)


def register_runtime_middleware(application: FastAPI, settings: RuntimeSettings) -> None:
    if not settings.internal_token:
        logger.warning(
            "ML_INTERNAL_TOKEN is not set — all API endpoints are unauthenticated; "
            "set ML_INTERNAL_TOKEN before deploying"
        )

    @application.middleware("http")
    async def check_internal_token(request: Request, call_next):
        if request.url.path in {"/health", "/ready", "/metrics"}:
            return await call_next(request)
        if settings.internal_token:
            token = request.headers.get("X-Internal-Token", "")
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
