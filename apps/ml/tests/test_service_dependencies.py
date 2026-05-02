import asyncio
from contextlib import asynccontextmanager
from types import SimpleNamespace

from app import service_dependencies
from app.api_models import FitAnalyzeRequest, RerankRequest
from app.fit_analysis_service import FitAnalysisService
from app.rerank_service import RerankService


def test_engine_api_client_factory_reuses_shared_engine_api_context(monkeypatch):
    captured: dict[str, object] = {}
    fake_engine_api = object()

    @asynccontextmanager
    async def fake_engine_api_client_context(*, base_url=None):
        captured["base_url"] = base_url
        yield fake_engine_api

    monkeypatch.setattr(
        service_dependencies,
        "engine_api_client_context",
        fake_engine_api_client_context,
    )

    async def open_factory() -> None:
        async with service_dependencies.engine_api_client_factory() as engine_api:
            captured["engine_api"] = engine_api

    asyncio.run(open_factory())

    assert captured["base_url"] is None
    assert captured["engine_api"] is fake_engine_api


def test_fit_analysis_service_accepts_shared_engine_api_factory(monkeypatch):
    captured: dict[str, object] = {}

    class FakeEngineApiClient:
        async def fetch_profile(self, profile_id: str):
            captured["profile_id"] = profile_id
            return {"profile": profile_id}

        async def fetch_job_lifecycle(self, job_id: str):
            captured["job_id"] = job_id
            return {"job": job_id}

    @asynccontextmanager
    async def fake_client_factory():
        yield FakeEngineApiClient()

    monkeypatch.setattr(
        "app.fit_analysis_service.score_job",
        lambda profile, job: (84, ["rust"], ["kafka"], ["shared terms: rust"]),
    )

    response = asyncio.run(
        FitAnalysisService(fake_client_factory).analyze(
            FitAnalyzeRequest(profile_id="profile-1", job_id="job-1")
        )
    )

    assert captured == {"profile_id": "profile-1", "job_id": "job-1"}
    assert response.model_dump() == {
        "profile_id": "profile-1",
        "job_id": "job-1",
        "score": 84,
        "matched_terms": ["rust"],
        "missing_terms": ["kafka"],
        "evidence": ["shared terms: rust"],
    }


def test_rerank_service_accepts_shared_engine_api_factory(monkeypatch):
    captured: dict[str, object] = {"job_ids": []}

    class FakeEngineApiClient:
        async def fetch_profile(self, profile_id: str):
            captured["profile_id"] = profile_id
            return {"profile": profile_id}

        async def fetch_job_lifecycle(self, job_id: str):
            captured["job_ids"].append(job_id)
            return SimpleNamespace(
                id=job_id,
                title=f"Title {job_id[-1]}",
                company_name=f"Company {job_id[-1]}",
            )

    @asynccontextmanager
    async def fake_client_factory():
        yield FakeEngineApiClient()

    def fake_score_job(profile, job):
        score = {"job-1": 90, "job-2": 70}[job.id]
        matched_terms = ["rust"] if job.id == "job-1" else ["python"]
        evidence = [f"score for {job.id}"]
        return score, matched_terms, [], evidence

    monkeypatch.setattr("app.rerank_service.score_job", fake_score_job)

    response = asyncio.run(
        RerankService(fake_client_factory).rerank(
            RerankRequest(profile_id="profile-1", job_ids=[" job-2 ", "job-1", "job-2"])
        )
    )

    assert captured == {"profile_id": "profile-1", "job_ids": ["job-2", "job-1"]}
    assert response.model_dump() == {
        "profile_id": "profile-1",
        "jobs": [
            {
                "job_id": "job-1",
                "title": "Title 1",
                "company_name": "Company 1",
                "score": 90,
                "matched_terms": ["rust"],
                "evidence": ["score for job-1"],
            },
            {
                "job_id": "job-2",
                "title": "Title 2",
                "company_name": "Company 2",
                "score": 70,
                "matched_terms": ["python"],
                "evidence": ["score for job-2"],
            },
        ],
    }


def test_build_cached_reranker_bootstrap_service_injects_workflow(monkeypatch):
    captured: dict[str, object] = {}

    service_dependencies.build_cached_reranker_bootstrap_service.cache_clear()

    def fake_service(*, bootstrap_workflow):
        captured["bootstrap_workflow"] = bootstrap_workflow
        return object()

    monkeypatch.setattr(service_dependencies, "RerankerBootstrapService", fake_service)

    service = service_dependencies.build_cached_reranker_bootstrap_service()

    assert service is not None
    assert captured["bootstrap_workflow"] is service_dependencies.bootstrap_workflow.bootstrap_and_retrain

    service_dependencies.build_cached_reranker_bootstrap_service.cache_clear()
