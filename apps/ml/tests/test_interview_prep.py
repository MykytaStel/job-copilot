from fastapi.testclient import TestClient

from app.interview_prep import (
    InterviewPrepRequest,
    InterviewPrepResponse,
    MalformedInterviewPrepOutputError,
    build_interview_prep_prompt,
    parse_interview_prep_output,
)
from app.api import app
from app.interview_prep_service import InterviewPrepService
from app.service_dependencies import get_interview_prep_service


def sample_interview_prep_context() -> InterviewPrepRequest:
    return InterviewPrepRequest.model_validate(
        {
            "profile_id": "profile-1",
            "analyzed_profile": {
                "summary": "Senior backend engineer focused on Rust services and platform reliability.",
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
            "application_coach": {
                "application_summary": "Tailor this application around proven backend platform evidence.",
                "resume_focus_points": ["Move Rust and Postgres near the top."],
                "suggested_bullets": ["Reframe existing Rust service work."],
                "cover_letter_angles": ["Connect platform reliability experience to the job summary."],
                "interview_focus": ["Prepare platform reliability examples."],
                "gaps_to_address": ["Leadership scope is not explicit in the deterministic context."],
                "red_flags": ["Do not claim architecture ownership without direct evidence."],
            },
            "cover_letter_draft": {
                "draft_summary": "Ground the letter in the explicit backend and Rust overlap already present.",
                "opening_paragraph": "Lead with grounded backend platform overlap.",
                "body_paragraphs": [
                    "The deterministic fit already highlights strong role overlap with backend_developer."
                ],
                "closing_paragraph": "Close with interest in discussing verified evidence in more detail.",
                "key_claims_used": [
                    "Senior backend engineer focused on Rust services and platform reliability."
                ],
                "evidence_gaps": ["Leadership evidence is not explicit in the deterministic context."],
                "tone_notes": ["Keep the tone direct and evidence-based."],
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

    async def generate_interview_prep(self, context, prompt):
        return self.payload


def test_interview_prep_prompt_has_explicit_sections():
    prompt = build_interview_prep_prompt(sample_interview_prep_context())

    assert "Do not invent experience, achievements, projects, metrics, employers" in prompt.system_instructions
    assert "place it in risk_areas instead of filling it in" in prompt.system_instructions
    assert '"application_coach"' in prompt.context_payload
    assert '"cover_letter_draft"' in prompt.context_payload
    assert '"prep_summary"' in prompt.output_schema_expectations
    assert '"follow_up_plan"' in prompt.output_schema_expectations


def test_interview_prep_output_normalizes_missing_values_and_bounds_lists():
    parsed = parse_interview_prep_output(
        {
            "prep_summary": "  `Prepare around explicit backend evidence`  ",
            "likely_topics": ["- Rust overlap", "", "- Rust overlap", "1. Platform scope"],
            "technical_focus": [
                "```json Prepare supported Rust examples ```",
                "* Prepare supported Postgres examples",
                "Review backend fit evidence",
                "Review platform alignment",
                "Review API ownership evidence",
                "Review remote collaboration fit",
                "Should be truncated",
            ],
            "behavioral_focus": None,
            "stories_to_prepare": ["1. One verified Rust example"],
            "questions_to_ask": ["* What outcomes matter most in the first few months?"],
            "risk_areas": ["- Leadership evidence is not explicit"],
            "follow_up_plan": ["* Bring only grounded examples"],
        }
    )

    assert parsed.prep_summary == "Prepare around explicit backend evidence"
    assert parsed.likely_topics == ["Rust overlap", "Platform scope"]
    assert len(parsed.technical_focus) == 6
    assert parsed.technical_focus[0] == "Prepare supported Rust examples"
    assert parsed.behavioral_focus == []
    assert parsed.stories_to_prepare == ["One verified Rust example"]
    assert parsed.questions_to_ask == ["What outcomes matter most in the first few months?"]
    assert parsed.risk_areas == ["Leadership evidence is not explicit"]
    assert parsed.follow_up_plan == ["Bring only grounded examples"]


def test_interview_prep_output_rejects_malformed_provider_payload():
    try:
        parse_interview_prep_output({"prep_summary": "ok", "likely_topics": "bad"})
    except MalformedInterviewPrepOutputError:
        pass
    else:  # pragma: no cover - defensive
        raise AssertionError("expected malformed provider output to raise")


def test_interview_prep_endpoint_returns_valid_enrichment():
    app.dependency_overrides[get_interview_prep_service] = lambda: InterviewPrepService(
        StubProvider(
            {
                "prep_summary": "```json Prepare around the explicit backend and Rust overlap already present in the deterministic context. ```",
                "likely_topics": [
                    "- How backend_developer experience maps to this role",
                    "* Rust and Postgres overlap in the posting",
                ],
                "technical_focus": [
                    "1. Prepare concrete Rust examples already supported by the profile.",
                    "* Prepare Postgres reliability examples already supported by the profile.",
                ],
                "behavioral_focus": ["Explain why this company is already on the positive target list."],
                "stories_to_prepare": ["Choose one verified backend platform example from the current profile context."],
                "questions_to_ask": ["What outcomes matter most for the Senior Rust Backend Engineer role in the first few months?"],
                "risk_areas": ["Leadership evidence is not explicit in the deterministic context."],
                "follow_up_plan": ["Bring two or three grounded questions so the conversation stays evidence-based."],
            }
        )
    )

    with TestClient(app) as client:
        response = client.post(
            "/v1/enrichment/interview-prep",
            json=sample_interview_prep_context().model_dump(by_alias=True),
        )

    app.dependency_overrides.clear()

    assert response.status_code == 200
    payload = InterviewPrepResponse.model_validate(response.json())
    assert payload.prep_summary == (
        "Prepare around the explicit backend and Rust overlap already present in the deterministic context."
    )
    assert payload.likely_topics == [
        "How backend_developer experience maps to this role",
        "Rust and Postgres overlap in the posting",
    ]
    assert payload.technical_focus == [
        "Prepare concrete Rust examples already supported by the profile.",
        "Prepare Postgres reliability examples already supported by the profile.",
    ]


def test_interview_prep_endpoint_handles_malformed_provider_output_gracefully():
    app.dependency_overrides[get_interview_prep_service] = lambda: InterviewPrepService(
        StubProvider({"prep_summary": "ok", "likely_topics": "bad"})
    )

    with TestClient(app) as client:
        response = client.post(
            "/v1/enrichment/interview-prep",
            json=sample_interview_prep_context().model_dump(by_alias=True),
        )

    app.dependency_overrides.clear()

    assert response.status_code == 502
    assert response.json()["detail"] == "Interview prep provider returned malformed output."
