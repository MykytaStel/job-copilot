from fastapi.testclient import TestClient

from app.api import app
from app.cv_tailoring import (
    CvTailoringRequest,
    CvTailoringResponse,
    MalformedCvTailoringOutputError,
    parse_cv_tailoring_suggestions,
)
from app.cv_tailoring_service import CvTailoringService
from app.service_dependencies import get_cv_tailoring_service
from app.template_cv_tailoring import build_cv_tailoring


def sample_request() -> CvTailoringRequest:
    return CvTailoringRequest.model_validate(
        {
            "profile_id": "profile-1",
            "job_id": "job-1",
            "profile_summary": "Senior backend engineer with Rust and Postgres experience.",
            "candidate_skills": ["Rust", "Postgres", "Docker", "Python"],
            "job_title": "Senior Rust Backend Engineer",
            "job_description": "Build and maintain distributed backend systems using Rust and Postgres.",
            "job_required_skills": ["Rust", "Postgres", "Distributed Systems"],
            "job_nice_to_have_skills": ["Docker", "Kafka"],
            "candidate_cv_text": "Led development of Rust microservices for a fintech platform.",
        }
    )


class StubTemplateProvider:
    def __init__(self, payload):
        self.payload = payload

    async def generate_cv_tailoring(self, context, prompt):
        return self.payload


StubProvider = StubTemplateProvider


def test_template_builder_highlights_matched_required_skills():
    ctx = sample_request()
    result = build_cv_tailoring(ctx)

    assert "Rust" in result["skills_to_highlight"]
    assert "Postgres" in result["skills_to_highlight"]


def test_template_builder_mentions_nice_to_have_candidate_skills():
    ctx = sample_request()
    result = build_cv_tailoring(ctx)

    assert "Docker" in result["skills_to_mention"]


def test_template_builder_reports_gaps_for_missing_required_skills():
    ctx = sample_request()
    result = build_cv_tailoring(ctx)

    gap_skills = [g["skill"] for g in result["gaps_to_address"]]
    assert "Distributed Systems" in gap_skills


def test_template_builder_produces_non_empty_summary_rewrite():
    ctx = sample_request()
    result = build_cv_tailoring(ctx)

    assert result["summary_rewrite"]
    assert "Senior Rust Backend Engineer" in result["summary_rewrite"] or "Rust" in result["summary_rewrite"]


def test_template_builder_produces_key_phrases():
    ctx = sample_request()
    result = build_cv_tailoring(ctx)

    assert len(result["key_phrases"]) > 0


def test_parse_cv_tailoring_suggestions_normalizes_and_dedupes_lists():
    parsed = parse_cv_tailoring_suggestions(
        {
            "skills_to_highlight": ["- Rust", "  rust  ", "Postgres"],
            "skills_to_mention": ["1. Docker"],
            "gaps_to_address": [{"skill": "Kafka", "suggestion": "Add Kafka examples."}],
            "summary_rewrite": "```json Senior Rust engineer. ```",
            "key_phrases": ["Rust", "rust"],
        }
    )

    assert parsed.skills_to_highlight == ["Rust", "Postgres"]
    assert parsed.skills_to_mention == ["Docker"]
    assert parsed.gaps_to_address[0].skill == "Kafka"
    assert parsed.summary_rewrite == "Senior Rust engineer."
    assert parsed.key_phrases == ["Rust"]


def test_parse_cv_tailoring_suggestions_rejects_non_json_string():
    try:
        parse_cv_tailoring_suggestions("not json {{{")
    except MalformedCvTailoringOutputError:
        pass
    else:  # pragma: no cover
        raise AssertionError("expected MalformedCvTailoringOutputError")


def test_parse_cv_tailoring_suggestions_rejects_wrong_gap_type():
    try:
        parse_cv_tailoring_suggestions(
            {
                "skills_to_highlight": [],
                "skills_to_mention": [],
                "gaps_to_address": [{"skill": 123, "suggestion": "bad"}],
                "summary_rewrite": "",
                "key_phrases": [],
            }
        )
    except MalformedCvTailoringOutputError:
        pass
    else:  # pragma: no cover
        raise AssertionError("expected MalformedCvTailoringOutputError")


def test_cv_tailoring_endpoint_returns_valid_response():
    app.dependency_overrides[get_cv_tailoring_service] = lambda: CvTailoringService(
        StubProvider(
            {
                "skills_to_highlight": ["Rust", "Postgres"],
                "skills_to_mention": ["Docker"],
                "gaps_to_address": [
                    {"skill": "Distributed Systems", "suggestion": "Add distributed systems examples."}
                ],
                "summary_rewrite": "Senior Rust backend engineer targeting platform roles.",
                "key_phrases": ["Rust", "Postgres", "Distributed Systems"],
            }
        )
    )

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
    assert payload.provider == "template"
    assert payload.generated_at


def test_cv_tailoring_endpoint_alias_path_also_works():
    app.dependency_overrides[get_cv_tailoring_service] = lambda: CvTailoringService(
        StubProvider(
            {
                "skills_to_highlight": ["Rust"],
                "skills_to_mention": [],
                "gaps_to_address": [],
                "summary_rewrite": "Rust engineer.",
                "key_phrases": ["Rust"],
            }
        )
    )

    with TestClient(app) as client:
        response = client.post(
            "/api/v1/cv-tailoring",
            json=sample_request().model_dump(by_alias=True),
        )

    app.dependency_overrides.clear()

    assert response.status_code == 200


def test_cv_tailoring_endpoint_returns_502_on_malformed_provider_output():
    app.dependency_overrides[get_cv_tailoring_service] = lambda: CvTailoringService(
        StubProvider("not valid json {{{")
    )

    with TestClient(app) as client:
        response = client.post(
            "/v1/cv-tailoring",
            json=sample_request().model_dump(by_alias=True),
        )

    app.dependency_overrides.clear()

    assert response.status_code == 502
    assert response.json()["detail"] == "CV tailoring provider returned malformed output."
