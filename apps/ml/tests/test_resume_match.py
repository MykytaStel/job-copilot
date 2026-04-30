from fastapi.testclient import TestClient

from app.api import app
from app.resume_match import ResumeMatchRequest, ResumeMatchResponse, ResumeMatchService


def sample_request() -> dict[str, str]:
    return {
        "resume_text": (
            "Senior backend engineer building Rust services with Postgres, Docker, and Python."
        ),
        "jd_text": (
            "We need a Senior Rust Backend Engineer for distributed systems using Rust, "
            "Postgres, Kafka, and Kubernetes."
        ),
    }


def test_resume_match_service_returns_keyword_coverage_and_gaps() -> None:
    response = ResumeMatchService().analyze(ResumeMatchRequest.model_validate(sample_request()))

    assert response.keyword_coverage_percent > 0
    assert "rust" in response.matched_keywords
    assert "kafka" in response.missing_keywords
    assert response.gap_summary


def test_resume_match_endpoint_alias_path_returns_valid_response() -> None:
    with TestClient(app) as client:
        response = client.post("/api/v1/enrichment/resume-match", json=sample_request())

    assert response.status_code == 200
    payload = ResumeMatchResponse.model_validate(response.json())
    assert payload.keyword_coverage_percent > 0
    assert payload.missing_keywords


def test_resume_match_endpoint_returns_422_on_invalid_body() -> None:
    with TestClient(app) as client:
        response = client.post(
            "/api/v1/enrichment/resume-match",
            json={"resume_text": "Rust and Postgres"},
        )

    assert response.status_code == 422

