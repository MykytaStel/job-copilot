from collections.abc import Awaitable, Callable
from pathlib import Path

from app.api_models import BootstrapRequest, BootstrapResponse
from app.bootstrap_contract import BootstrapWorkflowResult
from app.engine_api_client import EngineApiResponseError, EngineApiUnavailableError


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


BootstrapWorkflow = Callable[[str, int, Path], Awaitable[BootstrapWorkflowResult]]


class RerankerBootstrapService:
    def __init__(
        self,
        *,
        bootstrap_workflow: BootstrapWorkflow,
        model_path: Path,
    ) -> None:
        self._bootstrap_workflow = bootstrap_workflow
        self._model_path = model_path

    async def bootstrap(self, payload: BootstrapRequest) -> BootstrapResponse:
        try:
            result = await self._bootstrap_workflow(
                payload.profile_id,
                payload.min_examples,
                self._model_path,
            )
        except EngineApiResponseError as exc:
            raise RerankerBootstrapUpstreamHttpError(exc.status_code) from exc
        except EngineApiUnavailableError as exc:
            raise RerankerBootstrapUpstreamUnavailableError(
                f"engine-api unreachable: {exc.detail}"
            ) from exc

        return result.to_response()
