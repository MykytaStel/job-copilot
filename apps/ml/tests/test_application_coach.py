from fastapi.testclient import TestClient

from app.application_coach import (
    ApplicationCoachRequest,
    ApplicationCoachResponse,
    MalformedApplicationCoachOutputError,
    build_application_coach_prompt,
    parse_application_coach_output,
)
from app.api import app
from app.application_coach_service import ApplicationCoachService
from app.service_dependencies import get_application_coach_service


def sample_application_coach_context() -> ApplicationCoachRequest:
    return ApplicationCoachRequest.model_validate(
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
            "job_fit_explanation": {
                "fit_summary": "Strong deterministic fit for backend platform work.",
                "why_it_matches": ["Role and skill overlap are both explicit."],
                "risks": ["Keyword depth is narrower than the full job scope."],
                "missing_signals": ["Leadership evidence is not explicit in the deterministic context."],
                "recommended_next_step": "Tailor the opening bullets to Rust and Postgres work.",
                "application_angle": "Lead with backend platform ownership already evidenced in the profile.",
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
            "raw_profile_text": "Senior backend engineer with Rust, Postgres, Docker, and platform delivery experience.",
        }
    )


class StubProvider:
    def __init__(self, payload):
        self.payload = payload

    async def generate_application_coach(self, context, prompt):
        return self.payload


def test_application_coach_prompt_has_explicit_sections():
    prompt = build_application_coach_prompt(sample_application_coach_context())

    assert "Do not invent experience" in prompt.system_instructions
    assert "Only reframe existing profile evidence" in prompt.system_instructions
    assert '"search_profile"' in prompt.context_payload
    assert '"job_fit_explanation"' in prompt.context_payload
    assert '"application_summary"' in prompt.output_schema_expectations
    assert '"red_flags"' in prompt.output_schema_expectations


def test_application_coach_output_normalizes_missing_values_and_bounds_lists():
    parsed = parse_application_coach_output(
        {
            "application_summary": "  `Tailor around real Rust + backend evidence`  ",
            "resume_focus_points": ["- Rust depth", "", "- Rust depth", "1. Postgres ownership"],
            "suggested_bullets": [
                "```json Highlight backend platform work ```",
                "* Reframe existing Rust service work",
                "Use Postgres reliability work",
                "Use Docker deployment work",
                "Use API ownership evidence",
                "Use cross-team platform examples",
                "Should be truncated",
            ],
            "cover_letter_angles": None,
            "interview_focus": ["1. Prepare Rust examples"],
            "gaps_to_address": ["- Leadership scope is not explicit"],
            "red_flags": ["* Do not claim architecture ownership without evidence"],
        }
    )

    assert parsed.application_summary == "Tailor around real Rust + backend evidence"
    assert parsed.resume_focus_points == ["Rust depth", "Postgres ownership"]
    assert len(parsed.suggested_bullets) == 6
    assert parsed.suggested_bullets[0] == "Highlight backend platform work"
    assert parsed.cover_letter_angles == []
    assert parsed.interview_focus == ["Prepare Rust examples"]
    assert parsed.gaps_to_address == ["Leadership scope is not explicit"]
    assert parsed.red_flags == ["Do not claim architecture ownership without evidence"]


def test_application_coach_output_rejects_malformed_provider_payload():
    try:
        parse_application_coach_output({"application_summary": "ok", "resume_focus_points": "bad"})
    except MalformedApplicationCoachOutputError:
        pass
    else:  # pragma: no cover - defensive
        raise AssertionError("expected malformed provider output to raise")


def test_application_coach_endpoint_returns_valid_enrichment():
    app.dependency_overrides[get_application_coach_service] = lambda: ApplicationCoachService(
        StubProvider(
            {
                "application_summary": "```json Tailor this application around the proven Rust backend evidence already present in the profile. ```",
                "resume_focus_points": ["- Put Rust and Postgres near the top", "- Keep backend platform framing explicit"],
                "suggested_bullets": ["1. Reframe existing Rust service ownership", "* Reframe API and Postgres delivery evidence"],
                "cover_letter_angles": ["Connect proven backend platform work to the role summary."],
                "interview_focus": ["Prepare examples around Rust services and Postgres reliability."],
                "gaps_to_address": ["Leadership scope is not explicit in the deterministic context."],
                "red_flags": ["Do not claim achievements beyond the provided profile evidence."],
            }
        )
    )

    with TestClient(app) as client:
        response = client.post(
            "/v1/enrichment/application-coach",
            json=sample_application_coach_context().model_dump(by_alias=True),
        )

    app.dependency_overrides.clear()

    assert response.status_code == 200
    payload = ApplicationCoachResponse.model_validate(response.json())
    assert payload.application_summary == (
        "Tailor this application around the proven Rust backend evidence already present in the profile."
    )
    assert payload.resume_focus_points == [
        "Put Rust and Postgres near the top",
        "Keep backend platform framing explicit",
    ]
    assert payload.suggested_bullets == [
        "Reframe existing Rust service ownership",
        "Reframe API and Postgres delivery evidence",
    ]


def test_application_coach_endpoint_handles_malformed_provider_output_gracefully():
    app.dependency_overrides[get_application_coach_service] = lambda: ApplicationCoachService(
        StubProvider({"application_summary": "ok", "resume_focus_points": "bad"})
    )

    with TestClient(app) as client:
        response = client.post(
            "/v1/enrichment/application-coach",
            json=sample_application_coach_context().model_dump(by_alias=True),
        )

    app.dependency_overrides.clear()

    assert response.status_code == 502
    assert response.json()["detail"] == "Application coach provider returned malformed output."
