import asyncio
from collections.abc import Awaitable, Callable
import inspect
from pathlib import Path

from app.api_models import BootstrapRequest, BootstrapResponse
from app.bootstrap.locking import ProfileBootstrapAlreadyRunningError, ProfileBootstrapLock
from app.bootstrap_contract import BootstrapWorkflowResult
from app.engine_api_client import EngineApiResponseError, EngineApiUnavailableError
from app.trained_reranker_config import profile_artifact_path
from app.trained_reranker_config import get_trained_reranker_model_path


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


class RerankerBootstrapService:
    def __init__(
        self,
        *,
        bootstrap_workflow: BootstrapWorkflow,
        lock_dir: Path | None = None,
        model_path: Path | None = None,
    ) -> None:
        self._bootstrap_workflow = bootstrap_workflow
        self._lock_dir = lock_dir or (model_path.parent / ".bootstrap-locks" if model_path else Path(".bootstrap-locks"))
        self._legacy_model_path = model_path

    async def bootstrap(self, payload: BootstrapRequest) -> BootstrapResponse:
        lock = ProfileBootstrapLock(self._lock_dir / f"{payload.profile_id}.lock")
        try:
            await asyncio.to_thread(lock.acquire)
        except ProfileBootstrapAlreadyRunningError as exc:
            raise RerankerBootstrapConflictError(str(exc)) from exc

        profile_model_path = profile_artifact_path(payload.profile_id)
        runtime_model_path = get_trained_reranker_model_path()

        try:
            workflow_params = len(inspect.signature(self._bootstrap_workflow).parameters)
            if workflow_params <= 3 and self._legacy_model_path is not None:
                result = await self._bootstrap_workflow(
                    payload.profile_id,
                    payload.min_examples,
                    self._legacy_model_path,
                )
            else:
                result = await self._bootstrap_workflow(
                    payload.profile_id,
                    payload.min_examples,
                    profile_model_path,
                    runtime_model_path,
                )
        except EngineApiResponseError as exc:
            raise RerankerBootstrapUpstreamHttpError(exc.status_code) from exc
        except EngineApiUnavailableError as exc:
            raise RerankerBootstrapUpstreamUnavailableError(
                f"engine-api unreachable: {exc.detail}"
            ) from exc
        finally:
            await asyncio.to_thread(lock.release)

        return result.to_response()
