import asyncio
import logging
from collections.abc import AsyncIterator
from contextlib import asynccontextmanager
from time import perf_counter
from typing import Any

import httpx
from pydantic import BaseModel, field_validator

from app.dataset import OutcomeDataset
from app.settings import RuntimeSettings, get_runtime_settings

logger = logging.getLogger(__name__)


class EngineApiError(BaseModel):
    code: str | None = None
    message: str | None = None
    details: dict[str, Any] | None = None


class EngineApiClientError(Exception):
    pass


class EngineApiUnavailableError(EngineApiClientError):
    def __init__(self, detail: str):
        super().__init__(detail)
        self.detail = detail


class EngineApiResponseError(EngineApiClientError):
    def __init__(self, *, status_code: int, detail: str):
        super().__init__(detail)
        self.status_code = status_code
        self.detail = detail


class EngineProfileAnalysis(BaseModel):
    summary: str
    primary_role: str
    seniority: str
    skills: list[str]
    keywords: list[str]

    @field_validator("seniority", mode="before")
    @classmethod
    def normalize_seniority(cls, value: Any) -> str:
        if not isinstance(value, str):
            return ""
        cleaned = value.strip()
        return "" if cleaned.lower() == "unknown" else cleaned


class EngineProfile(BaseModel):
    id: str
    name: str
    email: str
    location: str | None = None
    raw_text: str
    analysis: EngineProfileAnalysis | None = None
    created_at: str
    updated_at: str
    skills_updated_at: str | None = None


class EngineJobLifecycleVariant(BaseModel):
    source: str
    source_job_id: str
    source_url: str
    fetched_at: str
    last_seen_at: str
    is_active: bool
    inactivated_at: str | None = None


class EngineJobPresentation(BaseModel):
    title: str
    company: str
    summary: str | None = None
    location_label: str | None = None
    work_mode_label: str | None = None
    source_label: str | None = None
    outbound_url: str | None = None
    salary_label: str | None = None
    freshness_label: str | None = None
    badges: list[str]


class EngineJobLifecycle(BaseModel):
    id: str
    title: str
    company_name: str
    location: str | None = None
    remote_type: str | None = None
    seniority: str | None = None
    description_text: str
    salary_min: int | None = None
    salary_max: int | None = None
    salary_currency: str | None = None
    posted_at: str | None = None
    first_seen_at: str
    last_seen_at: str
    is_active: bool
    inactivated_at: str | None = None
    reactivated_at: str | None = None
    lifecycle_stage: str
    primary_variant: EngineJobLifecycleVariant | None = None
    presentation: EngineJobPresentation

    @field_validator("seniority", mode="before")
    @classmethod
    def normalize_job_seniority(cls, value: Any) -> str | None:
        if value is None:
            return None
        if not isinstance(value, str):
            return None
        cleaned = value.strip()
        if not cleaned or cleaned.lower() == "unknown":
            return None
        return cleaned


def engine_api_base_url() -> str:
    return get_runtime_settings().engine_api_base_url


def engine_api_timeout_seconds() -> float:
    return get_runtime_settings().engine_api_timeout_seconds


def build_shared_http_client(settings: RuntimeSettings) -> httpx.AsyncClient:
    headers: dict[str, str] = {}
    if settings.engine_api_internal_token:
        headers["X-Internal-Token"] = settings.engine_api_internal_token
    kwargs = {
        "timeout": httpx.Timeout(settings.engine_api_timeout_seconds),
        "limits": httpx.Limits(max_connections=100, max_keepalive_connections=20),
        "headers": headers or None,
    }
    try:
        return httpx.AsyncClient(**kwargs)  # type: ignore[arg-type]
    except TypeError:
        return httpx.AsyncClient(timeout=kwargs["timeout"])  # type: ignore[arg-type]


_shared_client: httpx.AsyncClient | None = None


def configure_shared_client(client: httpx.AsyncClient | None) -> None:
    global _shared_client
    _shared_client = client


class EngineApiClient:
    def __init__(
        self,
        client: httpx.AsyncClient,
        *,
        base_url: str | None = None,
        request_id: str | None = None,
    ):
        self._client = client
        self._base_url = base_url.rstrip("/") if base_url else None
        self._request_id = request_id

    async def fetch_profile(self, profile_id: str) -> EngineProfile:
        payload = await self._fetch_json(f"/api/v1/profiles/{profile_id}")
        return EngineProfile.model_validate(payload)

    async def fetch_job_lifecycle(self, job_id: str) -> EngineJobLifecycle:
        payload = await self._fetch_json(f"/api/v1/ml/jobs/{job_id}/lifecycle")
        return EngineJobLifecycle.model_validate(payload)

    async def fetch_reranker_dataset(self, profile_id: str) -> OutcomeDataset:
        payload = await self._fetch_json(f"/api/v1/profiles/{profile_id}/reranker-dataset")
        return OutcomeDataset.model_validate(payload)

    async def probe_health(self) -> dict[str, Any]:
        return await self._fetch_json("/health")

    async def _fetch_json(self, path: str) -> dict[str, Any]:
        url = self._build_url(path)
        response = await self._get_with_retry(url)

        if response.status_code >= 400:
            try:
                payload = response.json() if response.content else {}
            except ValueError:
                payload = {}
            error = EngineApiError.model_validate(payload)
            detail = error.message or error.code or f"engine-api returned {response.status_code}"
            raise EngineApiResponseError(status_code=response.status_code, detail=detail)

        return response.json()

    async def _get_with_retry(self, url: str) -> httpx.Response:
        retryable_errors = (
            httpx.ConnectError,
            httpx.PoolTimeout,
            httpx.ReadTimeout,
            httpx.RemoteProtocolError,
            httpx.WriteError,
            httpx.WriteTimeout,
        )
        retryable_status_codes = {502, 503, 504}
        last_error: Exception | None = None
        extra_headers: dict[str, str] = {}
        if self._request_id:
            extra_headers["x-request-id"] = self._request_id

        for attempt in range(3):
            started = perf_counter()
            try:
                response = await self._client.get(url, headers=extra_headers or None)
            except retryable_errors as exc:
                last_error = exc
                if attempt == 2:
                    break
                await asyncio.sleep(0.2 * (attempt + 1))
                continue
            except httpx.HTTPError as exc:
                raise EngineApiUnavailableError(f"engine-api request failed: {exc}") from exc

            duration_ms = round((perf_counter() - started) * 1000, 2)
            log_extra = {
                "url": url,
                "status_code": response.status_code,
                "duration_ms": duration_ms,
                "request_id": self._request_id or "-",
            }
            if response.status_code >= 400:
                logger.warning("engine-api request returned error", extra=log_extra)
            else:
                logger.debug("engine-api request completed", extra=log_extra)

            if response.status_code in retryable_status_codes and attempt < 2:
                await asyncio.sleep(0.2 * (attempt + 1))
                continue
            return response

        detail = f"engine-api request failed: {last_error}" if last_error else "engine-api unavailable"
        raise EngineApiUnavailableError(detail)

    def _build_url(self, path: str) -> str:
        return f"{self._base_url or engine_api_base_url()}{path}"


@asynccontextmanager
async def engine_api_client_context(
    *,
    base_url: str | None = None,
    request_id: str | None = None,
) -> AsyncIterator[EngineApiClient]:
    if _shared_client is not None:
        yield EngineApiClient(_shared_client, base_url=base_url, request_id=request_id)
        return
    async with build_shared_http_client(get_runtime_settings()) as client:
        yield EngineApiClient(client, base_url=base_url, request_id=request_id)
