from typing import Any

from fastapi.testclient import TestClient

from app.api import app, create_app
from app.cv_tailoring import CvTailoringRequest, CvTailoringResponse
from app.cv_tailoring_service import CvTailoringService
from app.service_dependencies import get_cv_tailoring_service
from app.settings import get_runtime_settings


def sample_request() -> CvTailoringRequest:
    return CvTailoringRequest.model_validate(
        {
            "profile_id": "profile-1",
            "job_id": "job-1",
            "profile_summary": "Senior backend engineer with Rust and Postgres experience.",
            "candidate_skills": ["Rust", "Postgres", "Docker", "Python"],
            "job_title": "Senior Rust Backend Engineer",
            "job_description": (
                "Build and maintain distributed backend systems using Rust and Postgres."
            ),
            "job_required_skills": ["Rust", "Postgres", "Distributed Systems"],
            "job_nice_to_have_skills": ["Docker", "Kafka"],
            "candidate_cv_text": (
                "Led development of Rust microservices for a fintech platform."
            ),
        }
    )


class StubCvTailoringProvider:
    def __init__(self, payload: Any) -> None:
        self.payload = payload

    async def generate_cv_tailoring(self, context, prompt):
        return self.payload


def successful_service() -> CvTailoringService:
    return CvTailoringService(
        StubCvTailoringProvider(
            {
                "skills_to_highlight": ["Rust", "Postgres"],
                "skills_to_mention": ["Docker"],
                "gaps_to_address": [
                    {
                        "skill": "Distributed Systems",
                        "suggestion": "Add distributed systems examples.",
                    }
                ],
                "summary_rewrite": (
                    "Senior Rust backend engineer targeting platform roles."
                ),
                "key_phrases": ["Rust", "Postgres", "Distributed Systems"],
            }
        )
    )


def test_cv_tailoring_endpoint_returns_valid_response() -> None:
    app.dependency_overrides[get_cv_tailoring_service] = successful_service

    with TestClient(app) as client:
        response = client.post(
            "/v1/cv-tailoring",
            json=sample_request().model_dump(by_alias=True),
        )

    app.dependency_overrides.clear()

    assert response.status_code == 200

    payload = CvTailoringResponse.model_validate(response.json())

    assert payload.suggestions.skills_to_highlight == ["Rust", "Postgres"]
    assert payload.suggestions.skills_to_mention == ["Docker"]
    assert payload.suggestions.gaps_to_address[0].skill == "Distributed Systems"
    assert payload.provider == "unknown"
    assert payload.generated_at


def test_cv_tailoring_endpoint_alias_path_also_works() -> None:
    app.dependency_overrides[get_cv_tailoring_service] = successful_service

    with TestClient(app) as client:
        response = client.post(
            "/api/v1/cv-tailoring",
            json=sample_request().model_dump(by_alias=True),
        )

    app.dependency_overrides.clear()

    assert response.status_code == 200


def test_cv_tailoring_endpoint_returns_502_on_malformed_provider_output() -> None:
    app.dependency_overrides[get_cv_tailoring_service] = lambda: CvTailoringService(
        StubCvTailoringProvider("not valid json {{{")
    )

    with TestClient(app) as client:
        response = client.post(
            "/v1/cv-tailoring",
            json=sample_request().model_dump(by_alias=True),
        )

    app.dependency_overrides.clear()

    assert response.status_code == 502
    assert response.json()["detail"] == "CV tailoring provider returned malformed output."


def test_cv_tailoring_endpoint_returns_422_on_invalid_body() -> None:
    app.dependency_overrides[get_cv_tailoring_service] = successful_service

    with TestClient(app) as client:
        response = client.post(
            "/v1/cv-tailoring",
            json={
                "profile_id": "profile-1",
            },
        )

    app.dependency_overrides.clear()

    assert response.status_code == 422


def test_cv_tailoring_endpoint_returns_200_with_internal_token(monkeypatch) -> None:
    monkeypatch.setenv("ML_INTERNAL_TOKEN", "test-internal-token")
    get_runtime_settings.cache_clear()

    application = create_app()
    application.dependency_overrides[get_cv_tailoring_service] = successful_service

    with TestClient(application) as client:
        response = client.post(
            "/api/v1/cv-tailoring",
            json=sample_request().model_dump(by_alias=True),
            headers={"X-Internal-Token": "test-internal-token"},
        )

    application.dependency_overrides.clear()
    get_runtime_settings.cache_clear()

    assert response.status_code == 200


def test_cv_tailoring_endpoint_returns_401_without_internal_token(monkeypatch) -> None:
    monkeypatch.setenv("ML_INTERNAL_TOKEN", "test-internal-token")
    get_runtime_settings.cache_clear()

    application = create_app()
    application.dependency_overrides[get_cv_tailoring_service] = successful_service

    with TestClient(application) as client:
        response = client.post(
            "/api/v1/cv-tailoring",
            json=sample_request().model_dump(by_alias=True),
        )

    application.dependency_overrides.clear()
    get_runtime_settings.cache_clear()

    assert response.status_code == 401
    assert response.json()["detail"] == "unauthorized"