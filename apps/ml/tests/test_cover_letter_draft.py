from fastapi.testclient import TestClient

from app.cover_letter_draft import (
    MAX_BODY_PARAGRAPHS,
    MAX_PARAGRAPH_LENGTH,
    CoverLetterDraftRequest,
    CoverLetterDraftResponse,
    MalformedCoverLetterDraftOutputError,
    build_cover_letter_draft_prompt,
    parse_cover_letter_draft_output,
)
from app.cover_letter_draft_service import CoverLetterDraftService
from app.main import app, get_cover_letter_draft_service


def sample_cover_letter_context() -> CoverLetterDraftRequest:
    return CoverLetterDraftRequest.model_validate(
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

    async def generate_cover_letter_draft(self, context, prompt):
        return self.payload


def test_cover_letter_prompt_has_explicit_sections():
    prompt = build_cover_letter_draft_prompt(sample_cover_letter_context())

    assert "Do not invent employers" in prompt.system_instructions
    assert "evidence_gaps" in prompt.system_instructions
    assert '"application_coach"' in prompt.context_payload
    assert '"job_fit_explanation"' in prompt.context_payload
    assert '"draft_summary"' in prompt.output_schema_expectations
    assert '"tone_notes"' in prompt.output_schema_expectations


def test_cover_letter_output_normalizes_and_bounds_fields():
    long_paragraph = " ".join(["supported evidence"] * 80)
    parsed = parse_cover_letter_draft_output(
        {
            "draft_summary": "  `Ground the letter in real backend evidence`  ",
            "opening_paragraph": "```json Lead with the role and supported Rust overlap ```",
            "body_paragraphs": [
                "- Connect Rust and Postgres evidence to the job summary",
                "",
                "1. Connect Rust and Postgres evidence to the job summary",
                long_paragraph,
                "Use platform reliability examples",
                "This entry should be truncated by max items",
            ],
            "closing_paragraph": "* Close with measured interest and no unsupported claims",
            "key_claims_used": ["- Senior backend engineer", "- Senior backend engineer"],
            "evidence_gaps": None,
            "tone_notes": ["1. Keep the tone direct"],
        }
    )

    assert parsed.draft_summary == "Ground the letter in real backend evidence"
    assert parsed.opening_paragraph == "Lead with the role and supported Rust overlap"
    assert parsed.body_paragraphs[0] == "Connect Rust and Postgres evidence to the job summary"
    assert len(parsed.body_paragraphs) == MAX_BODY_PARAGRAPHS
    assert len(parsed.body_paragraphs[1]) <= MAX_PARAGRAPH_LENGTH
    assert parsed.closing_paragraph == "Close with measured interest and no unsupported claims"
    assert parsed.key_claims_used == ["Senior backend engineer"]
    assert parsed.evidence_gaps == []
    assert parsed.tone_notes == ["Keep the tone direct"]


def test_cover_letter_output_rejects_malformed_provider_payload():
    try:
        parse_cover_letter_draft_output({"draft_summary": "ok", "body_paragraphs": "bad"})
    except MalformedCoverLetterDraftOutputError:
        pass
    else:  # pragma: no cover - defensive
        raise AssertionError("expected malformed provider output to raise")


def test_cover_letter_endpoint_returns_valid_enrichment():
    app.dependency_overrides[get_cover_letter_draft_service] = lambda: CoverLetterDraftService(
        StubProvider(
            {
                "draft_summary": "```json Ground the draft in the explicit backend and Rust overlap already present in the profile and fit evidence. ```",
                "opening_paragraph": "- I am applying for the Senior Rust Backend Engineer role at SignalHire because the provided profile evidence already shows backend platform experience.",
                "body_paragraphs": [
                    "1. The deterministic fit already highlights strong role overlap with backend_developer and matched Rust plus Postgres skills.",
                    "* The available profile and job context both point to platform-focused backend work without requiring any invented claims.",
                ],
                "closing_paragraph": "Close with interest in discussing the verified backend platform evidence in more detail.",
                "key_claims_used": ["Senior backend engineer focused on Rust services and platform reliability."],
                "evidence_gaps": ["Leadership evidence is not explicit in the deterministic context."],
                "tone_notes": ["Keep the tone direct and evidence-based."],
            }
        )
    )

    with TestClient(app) as client:
        response = client.post(
            "/v1/enrichment/cover-letter-draft",
            json=sample_cover_letter_context().model_dump(by_alias=True),
        )

    app.dependency_overrides.clear()

    assert response.status_code == 200
    payload = CoverLetterDraftResponse.model_validate(response.json())
    assert payload.draft_summary == (
        "Ground the draft in the explicit backend and Rust overlap already present in the profile and fit evidence."
    )
    assert payload.body_paragraphs == [
        "The deterministic fit already highlights strong role overlap with backend_developer and matched Rust plus Postgres skills.",
        "The available profile and job context both point to platform-focused backend work without requiring any invented claims.",
    ]
    assert payload.tone_notes == ["Keep the tone direct and evidence-based."]


def test_cover_letter_endpoint_handles_malformed_provider_output_gracefully():
    app.dependency_overrides[get_cover_letter_draft_service] = lambda: CoverLetterDraftService(
        StubProvider({"draft_summary": "ok", "body_paragraphs": "bad"})
    )

    with TestClient(app) as client:
        response = client.post(
            "/v1/enrichment/cover-letter-draft",
            json=sample_cover_letter_context().model_dump(by_alias=True),
        )

    app.dependency_overrides.clear()

    assert response.status_code == 502
    assert response.json()["detail"] == "Cover letter draft provider returned malformed output."
