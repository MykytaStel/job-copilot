import asyncio
import logging
from collections.abc import Awaitable, Callable
from pathlib import Path

from app.api_models import BootstrapRequest, BootstrapResponse
from app.bootstrap.locking import ProfileBootstrapAlreadyRunningError, ProfileBootstrapLock
from app.bootstrap_contract import BootstrapWorkflowResult
from app.engine_api_client import EngineApiResponseError, EngineApiUnavailableError
from app.trained_reranker_config import get_trained_reranker_model_path, profile_artifact_path

logger = logging.getLogger(__name__)


class RerankerBootstrapServiceError(Exception):
    pass


class RerankerBootstrapUpstreamHttpError(RerankerBootstrapServiceError):
    def __init__(self, status_code: int):
        super().__init__(f"engine-api error: {status_code}")
        self.status_code = status_code


class RerankerBootstrapUpstreamUnavailableError(RerankerBootstrapServiceError):
    def __init__(self, detail: str):
        super().__init__(detail)
        self.detail = detail


class RerankerBootstrapConflictError(RerankerBootstrapServiceError):
    pass


BootstrapWorkflow = Callable[[str, int, Path, Path], Awaitable[BootstrapWorkflowResult]]
StartedCallback = Callable[[], None]


class RerankerBootstrapService:
    def __init__(
        self,
        *,
        bootstrap_workflow: BootstrapWorkflow,
        lock_dir: Path | None = None,
        max_concurrent_jobs: int = 2,
    ) -> None:
        self._bootstrap_workflow = bootstrap_workflow
        self._lock_dir = lock_dir or Path(".bootstrap-locks")
        self._max_concurrent_jobs = max(1, max_concurrent_jobs)
        self._bootstrap_semaphore = asyncio.Semaphore(self._max_concurrent_jobs)
        self._active_jobs = 0

    async def bootstrap(
        self,
        payload: BootstrapRequest,
        *,
        on_started: StartedCallback | None = None,
    ) -> BootstrapResponse:
        lock = ProfileBootstrapLock(self._lock_dir / f"{payload.profile_id}.lock")
        try:
            await asyncio.to_thread(lock.acquire)
        except ProfileBootstrapAlreadyRunningError as exc:
            raise RerankerBootstrapConflictError(str(exc)) from exc

        profile_model_path = profile_artifact_path(payload.profile_id)
        runtime_model_path = get_trained_reranker_model_path()

        logger.info(
            "bootstrap starting",
            extra={
                "profile_id": payload.profile_id,
                "min_examples": payload.min_examples,
                "artifact_path": str(profile_model_path),
            },
        )

        try:
            async with self._bootstrap_semaphore:
                self._active_jobs += 1
                try:
                    if on_started is not None:
                        on_started()
                    result = await self._bootstrap_workflow(
                        payload.profile_id,
                        payload.min_examples,
                        artifact_path=profile_model_path,
                        compatibility_model_path=runtime_model_path,
                    )
                finally:
                    self._active_jobs -= 1
        except EngineApiResponseError as exc:
            raise RerankerBootstrapUpstreamHttpError(exc.status_code) from exc
        except EngineApiUnavailableError as exc:
            raise RerankerBootstrapUpstreamUnavailableError(
                f"engine-api unreachable: {exc.detail}"
            ) from exc
        finally:
            await asyncio.to_thread(lock.release)

        logger.info(
            "bootstrap completed",
            extra={
                "profile_id": payload.profile_id,
                "retrained": result.retrained,
                "reason": result.reason,
            },
        )

        return result.to_response()

    def runtime_snapshot(self) -> dict[str, int]:
        active_jobs = max(0, self._active_jobs)
        return {
            "active_jobs": active_jobs,
            "max_concurrent_jobs": self._max_concurrent_jobs,
            "available_slots": max(0, self._max_concurrent_jobs - active_jobs),
        }
