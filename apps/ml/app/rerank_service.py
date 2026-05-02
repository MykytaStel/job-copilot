import asyncio
import logging
from dataclasses import dataclass
from time import monotonic, perf_counter

from app.api_models import RerankedJob, RerankRequest, RerankResponse
from app.engine_api_client import EngineApiClient
from app.scoring import score_job, unique_preserving_order

logger = logging.getLogger(__name__)

_CACHE_TTL = 300.0  # 5 minutes


@dataclass
class _CacheEntry:
    response: RerankResponse
    expires_at: float


class InvalidRerankRequestError(Exception):
    pass


class RerankService:
    def __init__(self, client_factory):
        self._client_factory = client_factory
        # {profile_id: {sorted_job_ids_tuple: _CacheEntry}}
        self._cache: dict[str, dict[tuple[str, ...], _CacheEntry]] = {}

    def invalidate(self, profile_id: str) -> None:
        self._cache.pop(profile_id, None)

    async def rerank(self, payload: RerankRequest) -> RerankResponse:
        started_at = perf_counter()
        unique_job_ids = unique_preserving_order(
            [job_id.strip() for job_id in payload.job_ids if job_id.strip()]
        )

        if not unique_job_ids:
            raise InvalidRerankRequestError("job_ids must contain at least one non-empty id")

        cache_key = tuple(sorted(unique_job_ids))

        if payload.cache_bust:
            self.invalidate(payload.profile_id)
        else:
            entry = self._cache.get(payload.profile_id, {}).get(cache_key)
            if entry is not None and monotonic() < entry.expires_at:
                return entry.response

        async with self._client_factory() as client_or_engine_api:
            engine_api = self._coerce_engine_api_client(client_or_engine_api)
            profile = await engine_api.fetch_profile(payload.profile_id)
            raw_results = await asyncio.gather(
                *(engine_api.fetch_job_lifecycle(job_id) for job_id in unique_job_ids),
                return_exceptions=True,
            )

        jobs = []
        for job_id, result in zip(unique_job_ids, raw_results):
            if isinstance(result, BaseException):
                logger.warning(
                    "failed to fetch job lifecycle",
                    extra={"job_id": job_id, "error": str(result)},
                )
            else:
                jobs.append(result)

        if not jobs:
            raise InvalidRerankRequestError("no jobs could be fetched")

        ranked_jobs: list[RerankedJob] = []

        for job in jobs:
            score, matched_terms, _, evidence = score_job(profile, job)
            ranked_jobs.append(
                RerankedJob(
                    job_id=job.id,
                    title=job.title,
                    company_name=job.company_name,
                    score=score,
                    matched_terms=matched_terms,
                    evidence=evidence,
                )
            )

        ranked_jobs.sort(key=lambda item: (-item.score, item.title.lower(), item.job_id))

        response = RerankResponse(profile_id=payload.profile_id, jobs=ranked_jobs)
        self._cache.setdefault(payload.profile_id, {})[cache_key] = _CacheEntry(
            response=response,
            expires_at=monotonic() + _CACHE_TTL,
        )

        logger.info(
            "rerank completed",
            extra={
                "profile_id": payload.profile_id,
                "requested_job_ids": len(payload.job_ids),
                "deduped_job_ids": len(unique_job_ids),
                "returned_jobs": len(ranked_jobs),
                "duration_ms": round((perf_counter() - started_at) * 1000, 2),
            },
        )

        return response

    @staticmethod
    def _coerce_engine_api_client(client_or_engine_api) -> EngineApiClient:
        if hasattr(client_or_engine_api, "fetch_profile") and hasattr(
            client_or_engine_api, "fetch_job_lifecycle"
        ):
            return client_or_engine_api
        return EngineApiClient(client_or_engine_api)
