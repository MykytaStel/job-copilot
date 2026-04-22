from collections.abc import AsyncIterator
from contextlib import asynccontextmanager
from typing import Any

import httpx
from pydantic import BaseModel, field_validator

from app.dataset import OutcomeDataset
from app.settings import get_runtime_settings


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


class EngineApiClient:
    def __init__(self, client: httpx.AsyncClient, *, base_url: str | None = None):
        self._client = client
        self._base_url = base_url.rstrip("/") if base_url else None

    async def fetch_profile(self, profile_id: str) -> EngineProfile:
        payload = await self._fetch_json(f"/api/v1/profiles/{profile_id}")
        return EngineProfile.model_validate(payload)

    async def fetch_job_lifecycle(self, job_id: str) -> EngineJobLifecycle:
        payload = await self._fetch_json(f"/api/v1/ml/jobs/{job_id}/lifecycle")
        return EngineJobLifecycle.model_validate(payload)

    async def fetch_reranker_dataset(self, profile_id: str) -> OutcomeDataset:
        payload = await self._fetch_json(f"/api/v1/profiles/{profile_id}/reranker-dataset")
        return OutcomeDataset.model_validate(payload)

    async def _fetch_json(self, path: str) -> dict[str, Any]:
        url = self._build_url(path)
        try:
            response = await self._client.get(url)
        except httpx.HTTPError as exc:
            raise EngineApiUnavailableError(f"engine-api request failed: {exc}") from exc

        if response.status_code >= 400:
            try:
                payload = response.json() if response.content else {}
            except ValueError:
                payload = {}
            error = EngineApiError.model_validate(payload)
            detail = error.message or error.code or f"engine-api returned {response.status_code}"
            raise EngineApiResponseError(status_code=response.status_code, detail=detail)

        return response.json()

    def _build_url(self, path: str) -> str:
        return f"{self._base_url or engine_api_base_url()}{path}"


@asynccontextmanager
async def engine_api_client_context(
    *,
    base_url: str | None = None,
) -> AsyncIterator[EngineApiClient]:
    timeout = httpx.Timeout(engine_api_timeout_seconds())
    async with httpx.AsyncClient(timeout=timeout) as client:
        yield EngineApiClient(client, base_url=base_url)
