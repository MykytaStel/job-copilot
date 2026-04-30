import pytest
from fastapi.testclient import TestClient

from app.api import create_app
from app.application_coach import ApplicationCoachResponse
from app.cover_letter_draft import CoverLetterDraftResponse
from app.cv_tailoring import CvTailoringResponse
from app.interview_prep import InterviewPrepResponse
from app.job_fit_explanation import JobFitExplanationResponse
from app.profile_insights import ProfileInsightsResponse
from app.settings import get_runtime_settings
from app.weekly_guidance import WeeklyGuidanceResponse


@pytest.fixture()
def template_client(monkeypatch, tmp_path):
    monkeypatch.setenv("ML_LLM_PROVIDER", "template")
    monkeypatch.setenv("ML_BOOTSTRAP_TASKS_DIR", str(tmp_path / "bootstrap-tasks"))
    monkeypatch.delenv("ML_INTERNAL_TOKEN", raising=False)
    monkeypatch.delenv("OPENAI_API_KEY", raising=False)
    get_runtime_settings.cache_clear()

    application = create_app()
    with TestClient(application) as client:
        yield client

    get_runtime_settings.cache_clear()


def analyzed_profile() -> dict:
    return {
        "summary": "Senior backend engineer focused on Rust services and platform reliability.",
        "primary_role": "backend_developer",
        "seniority": "senior",
        "skills": ["Rust", "Postgres", "Docker", "Python"],
        "keywords": ["backend", "platform", "distributed systems"],
    }


def search_profile() -> dict:
    return {
        "primary_role": "backend_developer",
        "primary_role_confidence": 0.93,
        "target_roles": ["backend_developer", "platform_engineer"],
        "role_candidates": [{"role": "backend_developer", "confidence": 0.93}],
        "seniority": "senior",
        "target_regions": ["eu_remote", "ua"],
        "work_modes": ["remote"],
        "allowed_sources": ["djinni", "workua"],
        "profile_skills": ["Rust", "Postgres", "Docker", "Python"],
        "profile_keywords": ["backend", "platform", "distributed systems"],
        "search_terms": ["rust backend", "platform engineer"],
        "exclude_terms": ["php"],
    }


def ranked_job() -> dict:
    return {
        "id": "job-1",
        "title": "Senior Rust Backend Engineer",
        "company_name": "SignalHire",
        "description_text": (
            "Own backend services, API reliability, distributed workflows, and "
            "Postgres-backed platform features for a remote product team."
        ),
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
    }


def deterministic_fit() -> dict:
    return {
        "job_id": "job-1",
        "score": 86,
        "matched_roles": ["backend_developer"],
        "matched_skills": ["Rust", "Postgres", "Docker"],
        "matched_keywords": ["platform", "distributed systems"],
        "source_match": True,
        "work_mode_match": True,
        "region_match": True,
        "reasons": [
            "Strong role overlap with backend_developer target.",
            "Rust and Postgres match the candidate profile.",
        ],
    }


def feedback_summary() -> dict:
    return {
        "saved_jobs_count": 6,
        "hidden_jobs_count": 2,
        "bad_fit_jobs_count": 1,
        "whitelisted_companies_count": 1,
        "blacklisted_companies_count": 0,
    }


def feedback_state() -> dict:
    return {
        "summary": feedback_summary(),
        "top_positive_evidence": [
            {"type": "saved_job", "label": "Senior Platform Engineer at SignalHire"},
            {"type": "whitelisted_company", "label": "SignalHire"},
        ],
        "top_negative_evidence": [{"type": "bad_fit_job", "label": "Legacy PHP backend role"}],
        "current_job_feedback": {
            "saved": False,
            "hidden": False,
            "bad_fit": False,
            "company_status": "whitelist",
        },
    }


def job_fit_explanation() -> dict:
    return {
        "fit_summary": "Strong deterministic fit for backend platform work.",
        "why_it_matches": ["Role, Rust, Postgres, and remote mode all align."],
        "risks": ["Leadership scope is not explicit in the deterministic context."],
        "missing_signals": ["No direct Kafka evidence is present."],
        "recommended_next_step": "Tailor the opening resume bullets to Rust and Postgres.",
        "application_angle": "Lead with backend platform ownership and reliability work.",
    }


def application_coach() -> dict:
    return {
        "application_summary": "Tailor this application around proven backend platform evidence.",
        "resume_focus_points": ["Move Rust and Postgres near the top."],
        "suggested_bullets": ["Reframe existing Rust service ownership."],
        "cover_letter_angles": ["Connect platform reliability work to the job summary."],
        "interview_focus": ["Prepare Rust services and Postgres reliability examples."],
        "gaps_to_address": ["Kafka is not explicit in the candidate evidence."],
        "red_flags": ["Do not claim Kafka production ownership without evidence."],
    }


def cover_letter_draft() -> dict:
    return {
        "draft_summary": "Evidence-based draft for the Rust backend role.",
        "opening_paragraph": "I am interested in the Senior Rust Backend Engineer role at SignalHire.",
        "body_paragraphs": [
            "My recent backend work maps to the Rust, Postgres, and platform reliability needs in the role.",
            "I would keep the application grounded in verified service ownership and delivery evidence.",
        ],
        "closing_paragraph": "I would welcome a conversation about the platform work your team is hiring for.",
        "key_claims_used": ["Senior backend engineer focused on Rust services."],
        "evidence_gaps": ["Kafka is not explicit in the supplied candidate profile."],
        "tone_notes": ["Keep the tone direct and evidence-based."],
    }


def job_payload(*, include_raw_profile_text: bool = False) -> dict:
    payload = {
        "profile_id": "profile-1",
        "analyzed_profile": analyzed_profile(),
        "search_profile": search_profile(),
        "ranked_job": ranked_job(),
        "deterministic_fit": deterministic_fit(),
        "feedback_state": feedback_state(),
    }
    if include_raw_profile_text:
        payload["raw_profile_text"] = (
            "Senior backend engineer with Rust, Postgres, Docker, Python, and "
            "platform delivery experience."
        )
    return payload


def profile_insights_payload() -> dict:
    return {
        "profile_id": "profile-1",
        "analyzed_profile": analyzed_profile(),
        "profile_skills": ["Rust", "Postgres", "Docker", "Python"],
        "profile_keywords": ["backend", "platform", "distributed systems"],
        "jobs_feed_summary": {
            "total": 120,
            "active": 86,
            "inactive": 26,
            "reactivated": 8,
        },
        "feedback_summary": feedback_summary(),
        "top_positive_evidence": [
            {"type": "saved_job", "label": "Senior Platform Engineer at SignalHire"},
            {"type": "whitelisted_company", "label": "SignalHire"},
        ],
        "top_negative_evidence": [{"type": "bad_fit_job", "label": "Legacy PHP backend role"}],
    }


def weekly_guidance_payload() -> dict:
    return {
        "profile_id": "profile-1",
        "analytics_summary": {
            "feedback": feedback_summary(),
            "jobs_by_source": [{"source": "djinni", "count": 44}, {"source": "workua", "count": 32}],
            "jobs_by_lifecycle": {
                "total": 120,
                "active": 86,
                "inactive": 26,
                "reactivated": 8,
            },
            "top_matched_roles": ["backend_developer", "platform_engineer"],
            "top_matched_skills": ["Rust", "Postgres", "Docker"],
            "top_matched_keywords": ["platform", "distributed systems"],
        },
        "behavior_summary": {
            "search_run_count": 7,
            "top_positive_sources": [signal_count("djinni", positive=5, negative=1)],
            "top_negative_sources": [signal_count("workua", positive=1, negative=3)],
            "top_positive_role_families": [signal_count("backend_developer", positive=5, negative=1)],
            "top_negative_role_families": [signal_count("legacy_backend", positive=0, negative=2)],
            "source_signal_counts": [signal_count("djinni", positive=5, negative=1)],
            "role_family_signal_counts": [signal_count("backend_developer", positive=5, negative=1)],
        },
        "funnel_summary": {
            "impression_count": 80,
            "open_count": 32,
            "save_count": 8,
            "hide_count": 3,
            "bad_fit_count": 2,
            "application_created_count": 2,
            "fit_explanation_requested_count": 5,
            "application_coach_requested_count": 2,
            "cover_letter_draft_requested_count": 1,
            "interview_prep_requested_count": 1,
            "conversion_rates": {
                "open_rate_from_impressions": 0.4,
                "save_rate_from_opens": 0.25,
                "application_rate_from_saves": 0.25,
            },
            "impressions_by_source": [{"source": "djinni", "count": 44}],
            "opens_by_source": [{"source": "djinni", "count": 20}],
            "saves_by_source": [{"source": "djinni", "count": 6}],
            "applications_by_source": [{"source": "djinni", "count": 2}],
        },
        "llm_context": {
            "analyzed_profile": analyzed_profile(),
            "profile_skills": ["Rust", "Postgres", "Docker", "Python"],
            "profile_keywords": ["backend", "platform", "distributed systems"],
            "jobs_feed_summary": {
                "total": 120,
                "active": 86,
                "inactive": 26,
                "reactivated": 8,
            },
            "feedback_summary": feedback_summary(),
            "top_positive_evidence": [
                {"type": "saved_job", "label": "Senior Platform Engineer at SignalHire"}
            ],
            "top_negative_evidence": [{"type": "bad_fit_job", "label": "Legacy PHP backend role"}],
            "top_model_signals": {"deterministic_score": 0.62, "matched_skill_count": 0.24},
        },
        "recent_search_summary": {
            "target_roles": ["backend_developer"],
            "search_terms": ["rust backend", "platform engineer"],
            "exclude_terms": ["php"],
            "allowed_sources": ["djinni", "workua"],
            "target_regions": ["eu_remote", "ua"],
            "work_modes": ["remote"],
        },
        "recent_feedback_summary": {
            "summary": feedback_summary(),
            "top_positive_evidence": [
                {"type": "saved_job", "label": "Senior Platform Engineer at SignalHire"}
            ],
            "top_negative_evidence": [{"type": "bad_fit_job", "label": "Legacy PHP backend role"}],
        },
    }


def signal_count(key: str, *, positive: int, negative: int) -> dict:
    return {
        "key": key,
        "save_count": positive,
        "hide_count": max(negative - 1, 0),
        "bad_fit_count": 1 if negative else 0,
        "application_created_count": 1 if positive >= 5 else 0,
        "positive_count": positive,
        "negative_count": negative,
        "net_score": positive - negative,
    }


@pytest.mark.parametrize(
    ("path", "payload", "response_model", "expected_keys"),
    [
        (
            "/api/v1/enrichment/profile-insights",
            profile_insights_payload(),
            ProfileInsightsResponse,
            {
                "profile_summary",
                "search_strategy_summary",
                "strengths",
                "risks",
                "recommended_actions",
                "top_focus_areas",
                "search_term_suggestions",
                "application_strategy",
            },
        ),
        (
            "/api/v1/enrichment/job-fit-explanation",
            job_payload(),
            JobFitExplanationResponse,
            {
                "fit_summary",
                "why_it_matches",
                "risks",
                "missing_signals",
                "recommended_next_step",
                "application_angle",
            },
        ),
        (
            "/api/v1/enrichment/application-coach",
            {
                **job_payload(include_raw_profile_text=True),
                "job_fit_explanation": job_fit_explanation(),
            },
            ApplicationCoachResponse,
            {
                "application_summary",
                "resume_focus_points",
                "suggested_bullets",
                "cover_letter_angles",
                "interview_focus",
                "gaps_to_address",
                "red_flags",
            },
        ),
        (
            "/api/v1/enrichment/cover-letter-draft",
            {
                **job_payload(),
                "raw_profile_text": job_payload(include_raw_profile_text=True)["raw_profile_text"],
                "job_fit_explanation": job_fit_explanation(),
                "application_coach": application_coach(),
            },
            CoverLetterDraftResponse,
            {
                "draft_summary",
                "opening_paragraph",
                "body_paragraphs",
                "closing_paragraph",
                "key_claims_used",
                "evidence_gaps",
                "tone_notes",
            },
        ),
        (
            "/api/v1/enrichment/interview-prep",
            {
                **job_payload(),
                "raw_profile_text": job_payload(include_raw_profile_text=True)["raw_profile_text"],
                "job_fit_explanation": job_fit_explanation(),
                "application_coach": application_coach(),
                "cover_letter_draft": cover_letter_draft(),
            },
            InterviewPrepResponse,
            {
                "prep_summary",
                "likely_topics",
                "technical_focus",
                "behavioral_focus",
                "stories_to_prepare",
                "questions_to_ask",
                "risk_areas",
                "follow_up_plan",
            },
        ),
        (
            "/api/v1/enrichment/weekly-guidance",
            weekly_guidance_payload(),
            WeeklyGuidanceResponse,
            {
                "weekly_summary",
                "what_is_working",
                "what_is_not_working",
                "recommended_search_adjustments",
                "recommended_source_moves",
                "recommended_role_focus",
                "funnel_bottlenecks",
                "next_week_plan",
            },
        ),
    ],
)
def test_template_enrichment_endpoints_return_contract_shapes(
    template_client,
    path: str,
    payload: dict,
    response_model,
    expected_keys: set[str],
) -> None:
    response = template_client.post(path, json=payload)

    assert response.status_code == 200, response.text
    body = response.json()
    assert set(body) == expected_keys

    parsed = response_model.model_validate(body)
    assert parsed.model_dump() == body
    assert any(value for value in body.values())


def test_template_cv_tailoring_endpoint_returns_contract_shape(template_client) -> None:
    response = template_client.post(
        "/api/v1/cv-tailoring",
        json={
            "profile_id": "profile-1",
            "job_id": "job-1",
            "profile_summary": analyzed_profile()["summary"],
            "candidate_skills": ["Rust", "Postgres", "Docker", "Python"],
            "job_title": "Senior Rust Backend Engineer",
            "job_description": ranked_job()["description_text"],
            "job_required_skills": ["Rust", "Postgres", "Distributed Systems"],
            "job_nice_to_have_skills": ["Docker", "Kafka"],
            "candidate_cv_text": (
                "Led development of Rust microservices and Postgres-backed internal APIs."
            ),
        },
    )

    assert response.status_code == 200, response.text
    body = response.json()
    assert set(body) == {"suggestions", "provider", "generated_at"}

    parsed = CvTailoringResponse.model_validate(body)
    assert parsed.provider == "template"
    assert parsed.generated_at
    assert parsed.suggestions.skills_to_highlight
    assert parsed.suggestions.summary_rewrite
