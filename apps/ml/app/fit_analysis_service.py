from app.api_models import FitAnalyzeRequest, FitAnalyzeResponse
from app.engine_api_client import EngineApiClient
from app.scoring import score_job


class FitAnalysisService:
    def __init__(self, client_factory):
        self._client_factory = client_factory

    async def analyze(self, payload: FitAnalyzeRequest) -> FitAnalyzeResponse:
        async with self._client_factory() as client_or_engine_api:
            engine_api = self._coerce_engine_api_client(client_or_engine_api)
            profile = await engine_api.fetch_profile(payload.profile_id)
            job = await engine_api.fetch_job_lifecycle(payload.job_id)

        score, matched_terms, missing_terms, evidence = score_job(profile, job)

        return FitAnalyzeResponse(
            profile_id=payload.profile_id,
            job_id=payload.job_id,
            score=score,
            matched_terms=matched_terms,
            missing_terms=missing_terms,
            evidence=evidence,
        )

    @staticmethod
    def _coerce_engine_api_client(client_or_engine_api) -> EngineApiClient:
        if hasattr(client_or_engine_api, "fetch_profile") and hasattr(
            client_or_engine_api, "fetch_job_lifecycle"
        ):
            return client_or_engine_api
        return EngineApiClient(client_or_engine_api)
