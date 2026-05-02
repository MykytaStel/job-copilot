from fastapi.testclient import TestClient

from app.api import app
from app.profile_insights import (
    LlmContextRequest,
    MalformedProviderOutputError,
    ProfileInsightsResponse,
    build_profile_insights_prompt,
    parse_profile_insights_output,
)
from app.profile_insights_service import ProfileInsightsService
from app.service_dependencies import get_profile_insights_service


def sample_context() -> LlmContextRequest:
    return LlmContextRequest.model_validate(
        {
            "profile_id": "profile-1",
            "analyzed_profile": {
                "summary": "Senior backend engineer focused on Rust services and platform work.",
                "primary_role": "backend_developer",
                "seniority": "senior",
                "skills": ["Rust", "Postgres", "Docker"],
                "keywords": ["backend", "platform", "distributed systems"],
            },
            "profile_skills": ["Rust", "Postgres", "Docker"],
            "profile_keywords": ["backend", "platform", "distributed systems"],
            "jobs_feed_summary": {
                "total": 120,
                "active": 80,
                "inactive": 30,
                "reactivated": 10,
            },
            "feedback_summary": {
                "saved_jobs_count": 6,
                "hidden_jobs_count": 2,
                "bad_fit_jobs_count": 3,
                "whitelisted_companies_count": 1,
                "blacklisted_companies_count": 1,
            },
            "top_positive_evidence": [
                {"type": "saved_job", "label": "job-1"},
                {"type": "whitelisted_company", "label": "SignalHire"},
            ],
            "top_negative_evidence": [
                {"type": "bad_fit_job", "label": "job-2"},
                {"type": "blacklisted_company", "label": "BadCorp"},
            ],
        }
    )


class StubProvider:
    def __init__(self, payload):
        self.payload = payload

    async def generate_profile_insights(self, context, prompt):
        return self.payload


def test_profile_insights_prompt_has_explicit_sections():
    prompt = build_profile_insights_prompt(sample_context())

    assert "Do not change ranking" in prompt.system_instructions
    assert "canonical role IDs" in prompt.system_instructions
    assert '"analyzed_profile"' in prompt.context_payload
    assert '"feedback_summary"' in prompt.context_payload
    assert '"profile_summary"' in prompt.output_schema_expectations
    assert '"search_term_suggestions"' in prompt.output_schema_expectations


def test_profile_insights_output_normalizes_missing_values_and_markdown():
    parsed = parse_profile_insights_output(
        {
            "profile_summary": "  `Strong backend profile`  ",
            "search_strategy_summary": "",
            "strengths": ["- Strong Rust depth", "", "- Strong Rust depth", "  "],
            "risks": None,
            "recommended_actions": ["1. Tighten titles", "* Reuse saved-job patterns"],
        }
    )

    assert parsed.profile_summary == "Strong backend profile"
    assert parsed.search_strategy_summary == ""
    assert parsed.strengths == ["Strong Rust depth"]
    assert parsed.risks == []
    assert parsed.recommended_actions == ["Tighten titles", "Reuse saved-job patterns"]
    assert parsed.search_term_suggestions == []


def test_profile_insights_output_rejects_malformed_provider_payload():
    try:
        parse_profile_insights_output({"strengths": "not-a-list"})
    except MalformedProviderOutputError:
        pass
    else:  # pragma: no cover - defensive
        raise AssertionError("expected malformed provider output to raise")


def test_profile_insights_endpoint_returns_valid_enrichment():
    app.dependency_overrides[get_profile_insights_service] = lambda: ProfileInsightsService(
        StubProvider(
            {
                "profile_summary": "```json Senior backend engineer with strong Rust evidence. ```",
                "search_strategy_summary": "Stay close to backend platform roles.",
                "strengths": ["- Strong Rust depth", "- Clear backend positioning"],
                "risks": ["Broader searches may drift into unrelated jobs."],
                "recommended_actions": ["1. Focus on backend platform titles"],
                "top_focus_areas": ["Rust", "Postgres"],
                "search_term_suggestions": ["backend engineer", "rust", "platform"],
                "application_strategy": ["Apply first where Rust is mentioned in the title or first requirements."],
            }
        )
    )

    with TestClient(app) as client:
        response = client.post("/v1/enrichment/profile-insights", json=sample_context().model_dump(by_alias=True))

    app.dependency_overrides.clear()

    assert response.status_code == 200
    payload = ProfileInsightsResponse.model_validate(response.json())
    assert payload.profile_summary == "Senior backend engineer with strong Rust evidence."
    assert payload.strengths == ["Strong Rust depth", "Clear backend positioning"]
    assert payload.search_term_suggestions == ["backend engineer", "rust", "platform"]


def test_profile_insights_endpoint_handles_malformed_provider_output_gracefully():
    app.dependency_overrides[get_profile_insights_service] = lambda: ProfileInsightsService(
        StubProvider({"profile_summary": "ok", "strengths": "bad"})
    )

    with TestClient(app) as client:
        response = client.post("/v1/enrichment/profile-insights", json=sample_context().model_dump(by_alias=True))

    app.dependency_overrides.clear()

    assert response.status_code == 502
    assert response.json()["detail"] == "Profile insights provider returned malformed output."
