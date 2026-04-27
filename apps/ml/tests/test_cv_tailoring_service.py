import asyncio
from typing import Any

from app.cv_tailoring import (
    CvTailoringRequest,
    MalformedCvTailoringOutputError,
)
from app.cv_tailoring_service import CvTailoringService
from app.llm_provider_template import TemplateEnrichmentProvider


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


def test_service_enriches_with_template_provider_without_mocks() -> None:
    service = CvTailoringService(TemplateEnrichmentProvider())

    response = asyncio.run(service.enrich(sample_request()))

    assert response.provider == "template"
    assert "Rust" in response.suggestions.skills_to_highlight
    assert "Docker" in response.suggestions.skills_to_mention
    assert response.suggestions.summary_rewrite
    assert response.generated_at


def test_service_enriches_with_mock_provider() -> None:
    service = CvTailoringService(
        StubCvTailoringProvider(
            {
                "skills_to_highlight": ["Rust", "Postgres"],
                "skills_to_mention": ["Docker"],
                "gaps_to_address": [
                    {
                        "skill": "Distributed Systems",
                        "suggestion": "Add examples of distributed system ownership.",
                    }
                ],
                "summary_rewrite": (
                    "Senior Rust backend engineer focused on platform systems."
                ),
                "key_phrases": ["Rust", "Postgres", "Distributed Systems"],
            }
        )
    )

    response = asyncio.run(service.enrich(sample_request()))

    assert response.provider == "unknown"
    assert response.suggestions.skills_to_highlight == ["Rust", "Postgres"]
    assert response.suggestions.skills_to_mention == ["Docker"]
    assert response.suggestions.gaps_to_address[0].skill == "Distributed Systems"
    assert response.suggestions.summary_rewrite.startswith("Senior Rust")


def test_service_raises_safe_error_for_malformed_provider_output() -> None:
    service = CvTailoringService(StubCvTailoringProvider("not json {{{"))

    try:
        asyncio.run(service.enrich(sample_request()))
    except MalformedCvTailoringOutputError:
        pass
    else:  # pragma: no cover
        raise AssertionError("expected MalformedCvTailoringOutputError")