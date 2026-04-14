from fastapi.testclient import TestClient

from app.job_fit_explanation import (
    JobFitExplanationRequest,
    JobFitExplanationResponse,
    MalformedJobFitExplanationOutputError,
    build_job_fit_explanation_prompt,
    parse_job_fit_explanation_output,
)
from app.job_fit_explanation_service import JobFitExplanationService
from app.main import app, get_job_fit_explanation_service


def sample_job_fit_context() -> JobFitExplanationRequest:
    return JobFitExplanationRequest.model_validate(
        {
            "profile_id": "profile-1",
            "analyzed_profile": {
                "summary": "Senior backend engineer focused on Rust services and platform work.",
                "primary_role": "backend_developer",
                "seniority": "senior",
                "skills": ["Rust", "Postgres", "Docker"],
                "keywords": ["backend", "platform", "distributed systems"],
            },
            "search_profile": {
                "primary_role": "backend_developer",
                "primary_role_confidence": 0.93,
                "target_roles": ["backend_developer", "platform_engineer"],
                "role_candidates": [{"role": "backend_developer", "confidence": 0.93}],
                "seniority": "senior",
                "target_regions": ["eu_remote", "ua"],
                "work_modes": ["remote"],
                "allowed_sources": ["djinni", "workua"],
                "profile_skills": ["Rust", "Postgres", "Docker"],
                "profile_keywords": ["backend", "platform", "distributed systems"],
                "search_terms": ["rust backend", "platform engineer"],
                "exclude_terms": ["php"],
            },
            "ranked_job": {
                "id": "job-1",
                "title": "Senior Rust Backend Engineer",
                "company_name": "SignalHire",
                "description_text": "Own backend services, APIs, and platform reliability.",
                "summary": "Platform-focused backend role with Rust and Postgres.",
                "source": "djinni",
                "source_job_id": "dj-1",
                "source_url": "https://example.com/jobs/1",
                "remote_type": "remote",
                "seniority": "senior",
                "salary_label": "$4,000 - $5,000",
                "location_label": "Remote EU",
                "work_mode_label": "Remote",
                "freshness_label": "Seen today",
                "badges": ["remote", "active"],
            },
            "deterministic_fit": {
                "job_id": "job-1",
                "score": 82,
                "matched_roles": ["backend_developer"],
                "matched_skills": ["Rust", "Postgres"],
                "matched_keywords": ["platform"],
                "source_match": True,
                "work_mode_match": True,
                "region_match": True,
                "reasons": ["Strong role overlap with backend_developer target."],
            },
            "feedback_state": {
                "summary": {
                    "saved_jobs_count": 6,
                    "hidden_jobs_count": 2,
                    "bad_fit_jobs_count": 1,
                    "whitelisted_companies_count": 1,
                    "blacklisted_companies_count": 0,
                },
                "top_positive_evidence": [
                    {"type": "saved_job", "label": "job-2"},
                    {"type": "whitelisted_company", "label": "SignalHire"},
                ],
                "top_negative_evidence": [{"type": "bad_fit_job", "label": "job-3"}],
                "current_job_feedback": {
                    "saved": False,
                    "hidden": False,
                    "bad_fit": False,
                    "company_status": "whitelist",
                },
            },
        }
    )


class StubProvider:
    def __init__(self, payload):
        self.payload = payload

    async def generate_job_fit_explanation(self, context, prompt):
        return self.payload


def test_job_fit_prompt_has_explicit_sections():
    prompt = build_job_fit_explanation_prompt(sample_job_fit_context())

    assert "Do not change or reinterpret ranking" in prompt.system_instructions
    assert "missing_signals" in prompt.system_instructions
    assert '"ranked_job"' in prompt.context_payload
    assert '"deterministic_fit"' in prompt.context_payload
    assert '"fit_summary"' in prompt.output_schema_expectations
    assert '"application_angle"' in prompt.output_schema_expectations


def test_job_fit_output_normalizes_missing_values_and_markdown():
    parsed = parse_job_fit_explanation_output(
        {
            "fit_summary": "  `Strong fit for backend work`  ",
            "why_it_matches": ["- Strong Rust overlap", "", "- Strong Rust overlap"],
            "risks": None,
            "missing_signals": ["1. No salary match evidence"],
            "recommended_next_step": "```json Tailor the first CV bullets ```",
            "application_angle": "* Lead with platform ownership",
        }
    )

    assert parsed.fit_summary == "Strong fit for backend work"
    assert parsed.why_it_matches == ["Strong Rust overlap"]
    assert parsed.risks == []
    assert parsed.missing_signals == ["No salary match evidence"]
    assert parsed.recommended_next_step == "Tailor the first CV bullets"
    assert parsed.application_angle == "Lead with platform ownership"


def test_job_fit_output_rejects_malformed_provider_payload():
    try:
        parse_job_fit_explanation_output({"why_it_matches": "not-a-list"})
    except MalformedJobFitExplanationOutputError:
        pass
    else:  # pragma: no cover - defensive
        raise AssertionError("expected malformed provider output to raise")


def test_job_fit_endpoint_returns_valid_enrichment():
    app.dependency_overrides[get_job_fit_explanation_service] = lambda: JobFitExplanationService(
        StubProvider(
            {
                "fit_summary": "```json Strong deterministic match for backend work. ```",
                "why_it_matches": ["- Strong Rust overlap", "- Seniority and remote mode align"],
                "risks": ["Limited keyword depth beyond backend platform signals."],
                "missing_signals": ["Direct compensation overlap is not explicit."],
                "recommended_next_step": "1. Tailor the opening resume bullets to Rust and Postgres.",
                "application_angle": "* Lead with backend platform ownership and delivery evidence.",
            }
        )
    )

    with TestClient(app) as client:
        response = client.post(
            "/v1/enrichment/job-fit-explanation",
            json=sample_job_fit_context().model_dump(by_alias=True),
        )

    app.dependency_overrides.clear()

    assert response.status_code == 200
    payload = JobFitExplanationResponse.model_validate(response.json())
    assert payload.fit_summary == "Strong deterministic match for backend work."
    assert payload.why_it_matches == ["Strong Rust overlap", "Seniority and remote mode align"]
    assert payload.application_angle == "Lead with backend platform ownership and delivery evidence."


def test_job_fit_endpoint_handles_malformed_provider_output_gracefully():
    app.dependency_overrides[get_job_fit_explanation_service] = lambda: JobFitExplanationService(
        StubProvider({"fit_summary": "ok", "why_it_matches": "bad"})
    )

    with TestClient(app) as client:
        response = client.post(
            "/v1/enrichment/job-fit-explanation",
            json=sample_job_fit_context().model_dump(by_alias=True),
        )

    app.dependency_overrides.clear()

    assert response.status_code == 502
    assert response.json()["detail"] == "Job fit explanation provider returned malformed output."
