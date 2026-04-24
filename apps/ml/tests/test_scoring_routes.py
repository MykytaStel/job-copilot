from fastapi.testclient import TestClient

from app.api import app
from app.api_models import BootstrapResponse, FitAnalyzeResponse, RerankResponse
from app.engine_api_client import EngineApiResponseError, EngineApiUnavailableError
from app.rerank_service import InvalidRerankRequestError
from app.reranker_bootstrap_service import (
    RerankerBootstrapUpstreamHttpError,
    RerankerBootstrapUpstreamUnavailableError,
)
from app.service_dependencies import (
    get_fit_analysis_service,
    get_rerank_service,
    get_reranker_bootstrap_service,
)
from app.trained_reranker import TrainingSummary

_BOOTSTRAP_PAYLOAD = {"profile_id": "profile-1", "min_examples": 30}


class StubFitAnalysisService:
    def __init__(self, response=None, error=None):
        self._response = response
        self._error = error

    async def analyze(self, payload):
        if self._error is not None:
            raise self._error
        return self._response


class StubRerankService:
    def __init__(self, response=None, error=None):
        self._response = response
        self._error = error

    async def rerank(self, payload):
        if self._error is not None:
            raise self._error
        return self._response


class StubRerankerBootstrapService:
    def __init__(self, response=None, error=None):
        self._response = response
        self._error = error

    async def bootstrap(self, payload, *, on_started=None):
        if on_started is not None:
            on_started()
        if self._error is not None:
            raise self._error
        return self._response


def test_fit_analyze_route_returns_service_response():
    app.dependency_overrides[get_fit_analysis_service] = lambda: StubFitAnalysisService(
        FitAnalyzeResponse(
            profile_id="profile-1",
            job_id="job-1",
            score=84,
            matched_terms=["rust"],
            missing_terms=["kafka"],
            evidence=["shared terms: rust"],
        )
    )

    with TestClient(app) as client:
        response = client.post(
            "/api/v1/fit/analyze",
            json={"profile_id": "profile-1", "job_id": "job-1"},
        )

    app.dependency_overrides.clear()

    assert response.status_code == 200
    assert response.json()["score"] == 84


def test_fit_analyze_route_maps_engine_api_unavailable_error():
    app.dependency_overrides[get_fit_analysis_service] = lambda: StubFitAnalysisService(
        error=EngineApiUnavailableError("engine-api request failed: timed out")
    )

    with TestClient(app) as client:
        response = client.post(
            "/api/v1/fit/analyze",
            json={"profile_id": "profile-1", "job_id": "job-1"},
        )

    app.dependency_overrides.clear()

    assert response.status_code == 503
    assert response.json()["detail"] == "engine-api request failed: timed out"


def test_rerank_route_maps_invalid_request_error_to_bad_request():
    app.dependency_overrides[get_rerank_service] = lambda: StubRerankService(
        error=InvalidRerankRequestError("job_ids must contain at least one non-empty id")
    )

    with TestClient(app) as client:
        response = client.post(
            "/api/v1/rerank",
            json={"profile_id": "profile-1", "job_ids": [" "]},
        )

    app.dependency_overrides.clear()

    assert response.status_code == 400
    assert response.json()["detail"] == "job_ids must contain at least one non-empty id"


def test_rerank_route_returns_service_response():
    app.dependency_overrides[get_rerank_service] = lambda: StubRerankService(
        RerankResponse(
            profile_id="profile-1",
            jobs=[
                {
                    "job_id": "job-1",
                    "title": "Senior Rust Engineer",
                    "company_name": "SignalHire",
                    "score": 90,
                    "matched_terms": ["rust"],
                    "evidence": ["shared terms: rust"],
                }
            ],
        )
    )

    with TestClient(app) as client:
        response = client.post(
            "/api/v1/rerank",
            json={"profile_id": "profile-1", "job_ids": ["job-1"]},
        )

    app.dependency_overrides.clear()

    assert response.status_code == 200
    assert response.json()["jobs"][0]["score"] == 90


def test_rerank_route_maps_engine_api_response_error():
    app.dependency_overrides[get_rerank_service] = lambda: StubRerankService(
        error=EngineApiResponseError(status_code=404, detail="profile not found")
    )

    with TestClient(app) as client:
        response = client.post(
            "/api/v1/rerank",
            json={"profile_id": "profile-1", "job_ids": ["job-1"]},
        )

    app.dependency_overrides.clear()

    assert response.status_code == 404
    assert response.json()["detail"] == "profile not found"


def test_reranker_bootstrap_route_returns_202_with_task_id():
    app.dependency_overrides[get_reranker_bootstrap_service] = (
        lambda: StubRerankerBootstrapService(
            BootstrapResponse(retrained=False, example_count=12, reason="need at least 30 examples, got 12")
        )
    )

    with TestClient(app) as client:
        response = client.post("/api/v1/reranker/bootstrap", json=_BOOTSTRAP_PAYLOAD)

    app.dependency_overrides.clear()

    assert response.status_code == 202
    body = response.json()
    assert "task_id" in body
    assert body["status"] == "accepted"


def test_reranker_bootstrap_route_background_task_stores_completed_result():
    expected = BootstrapResponse(
        retrained=False,
        example_count=12,
        reason="need at least 30 examples, got 12",
    )
    app.dependency_overrides[get_reranker_bootstrap_service] = (
        lambda: StubRerankerBootstrapService(expected)
    )

    with TestClient(app) as client:
        post_resp = client.post("/api/v1/reranker/bootstrap", json=_BOOTSTRAP_PAYLOAD)
        assert post_resp.status_code == 202
        task_id = post_resp.json()["task_id"]

        status_resp = client.get(f"/api/v1/reranker/bootstrap/{task_id}")

    app.dependency_overrides.clear()

    assert status_resp.status_code == 200
    data = status_resp.json()
    assert data["status"] == "completed"
    assert data["result"]["retrained"] is False
    assert data["result"]["example_count"] == 12


def test_reranker_bootstrap_route_background_task_stores_trained_payload_shape():
    app.dependency_overrides[get_reranker_bootstrap_service] = (
        lambda: StubRerankerBootstrapService(
            BootstrapResponse(
                retrained=True,
                example_count=30,
                model_path="/tmp/trained-reranker-v3.json",
                training=TrainingSummary(
                    example_count=30,
                    positive_count=10,
                    medium_count=10,
                    negative_count=10,
                    saved_only_count=4,
                    viewed_only_count=3,
                    medium_default_count=0,
                    epochs=500,
                    learning_rate=0.08,
                    l2=0.01,
                    loss=0.123,
                ),
            )
        )
    )

    with TestClient(app) as client:
        post_resp = client.post("/api/v1/reranker/bootstrap", json=_BOOTSTRAP_PAYLOAD)
        assert post_resp.status_code == 202
        task_id = post_resp.json()["task_id"]

        status_resp = client.get(f"/api/v1/reranker/bootstrap/{task_id}")

    app.dependency_overrides.clear()

    assert status_resp.status_code == 200
    data = status_resp.json()
    assert data["status"] == "completed"
    assert data["result"]["retrained"] is True
    assert data["result"]["model_path"] == "/tmp/trained-reranker-v3.json"
    assert data["result"]["training"]["loss"] == 0.123


def test_reranker_bootstrap_route_background_task_stores_upstream_http_error():
    app.dependency_overrides[get_reranker_bootstrap_service] = (
        lambda: StubRerankerBootstrapService(error=RerankerBootstrapUpstreamHttpError(404))
    )

    with TestClient(app) as client:
        post_resp = client.post("/api/v1/reranker/bootstrap", json=_BOOTSTRAP_PAYLOAD)
        assert post_resp.status_code == 202
        task_id = post_resp.json()["task_id"]

        status_resp = client.get(f"/api/v1/reranker/bootstrap/{task_id}")

    app.dependency_overrides.clear()

    assert status_resp.status_code == 200
    data = status_resp.json()
    assert data["status"] == "failed"
    assert data["error"] == "engine-api error: 404"


def test_reranker_bootstrap_route_background_task_stores_upstream_unavailable_error():
    app.dependency_overrides[get_reranker_bootstrap_service] = (
        lambda: StubRerankerBootstrapService(
            error=RerankerBootstrapUpstreamUnavailableError("engine-api unreachable: boom")
        )
    )

    with TestClient(app) as client:
        post_resp = client.post("/api/v1/reranker/bootstrap", json=_BOOTSTRAP_PAYLOAD)
        assert post_resp.status_code == 202
        task_id = post_resp.json()["task_id"]

        status_resp = client.get(f"/api/v1/reranker/bootstrap/{task_id}")

    app.dependency_overrides.clear()

    assert status_resp.status_code == 200
    data = status_resp.json()
    assert data["status"] == "failed"
    assert data["error"] == "engine-api unreachable: boom"


def test_bootstrap_task_status_returns_404_for_unknown_task_id():
    with TestClient(app) as client:
        response = client.get("/api/v1/reranker/bootstrap/nonexistent-task-id")

    assert response.status_code == 404
    assert response.json()["detail"] == "task not found"
