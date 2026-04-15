from fastapi.testclient import TestClient

from app.main import app, get_weekly_guidance_service
from app.weekly_guidance import (
    MalformedWeeklyGuidanceOutputError,
    WeeklyGuidanceRequest,
    WeeklyGuidanceResponse,
    build_weekly_guidance_prompt,
    parse_weekly_guidance_output,
)
from app.weekly_guidance_service import WeeklyGuidanceService


def sample_weekly_guidance_context() -> WeeklyGuidanceRequest:
    return WeeklyGuidanceRequest.model_validate(
        {
            "profile_id": "profile-1",
            "analytics_summary": {
                "feedback": {
                    "saved_jobs_count": 6,
                    "hidden_jobs_count": 2,
                    "bad_fit_jobs_count": 3,
                    "whitelisted_companies_count": 1,
                    "blacklisted_companies_count": 1,
                },
                "jobs_by_source": [
                    {"source": "djinni", "count": 40},
                    {"source": "work_ua", "count": 20},
                ],
                "jobs_by_lifecycle": {
                    "total": 120,
                    "active": 80,
                    "inactive": 30,
                    "reactivated": 10,
                },
                "top_matched_roles": ["backend_developer", "platform_engineer"],
                "top_matched_skills": ["Rust", "Postgres", "Docker"],
                "top_matched_keywords": ["backend", "platform", "distributed systems"],
            },
            "behavior_summary": {
                "search_run_count": 5,
                "top_positive_sources": [
                    {
                        "key": "djinni",
                        "save_count": 4,
                        "hide_count": 0,
                        "bad_fit_count": 0,
                        "application_created_count": 1,
                        "positive_count": 5,
                        "negative_count": 0,
                        "net_score": 6,
                    }
                ],
                "top_negative_sources": [
                    {
                        "key": "work_ua",
                        "save_count": 0,
                        "hide_count": 2,
                        "bad_fit_count": 1,
                        "application_created_count": 0,
                        "positive_count": 0,
                        "negative_count": 3,
                        "net_score": -3,
                    }
                ],
                "top_positive_role_families": [
                    {
                        "key": "engineering",
                        "save_count": 4,
                        "hide_count": 0,
                        "bad_fit_count": 0,
                        "application_created_count": 1,
                        "positive_count": 5,
                        "negative_count": 0,
                        "net_score": 6,
                    }
                ],
                "top_negative_role_families": [
                    {
                        "key": "support",
                        "save_count": 0,
                        "hide_count": 1,
                        "bad_fit_count": 2,
                        "application_created_count": 0,
                        "positive_count": 0,
                        "negative_count": 3,
                        "net_score": -3,
                    }
                ],
                "source_signal_counts": [],
                "role_family_signal_counts": [],
            },
            "funnel_summary": {
                "impression_count": 30,
                "open_count": 12,
                "save_count": 4,
                "hide_count": 2,
                "bad_fit_count": 3,
                "application_created_count": 1,
                "fit_explanation_requested_count": 4,
                "application_coach_requested_count": 2,
                "cover_letter_draft_requested_count": 1,
                "interview_prep_requested_count": 1,
                "conversion_rates": {
                    "open_rate_from_impressions": 0.4,
                    "save_rate_from_opens": 0.333,
                    "application_rate_from_saves": 0.25,
                },
                "impressions_by_source": [{"source": "djinni", "count": 20}],
                "opens_by_source": [{"source": "djinni", "count": 10}],
                "saves_by_source": [{"source": "djinni", "count": 4}],
                "applications_by_source": [{"source": "djinni", "count": 1}],
            },
            "llm_context": {
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
                "top_positive_evidence": [{"type": "saved_job", "label": "job-2"}],
                "top_negative_evidence": [{"type": "bad_fit_job", "label": "job-3"}],
            },
        }
    )


class StubProvider:
    def __init__(self, payload):
        self.payload = payload

    async def generate_weekly_guidance(self, context, prompt):
        return self.payload


def test_weekly_guidance_prompt_has_explicit_grounding_rules():
    prompt = build_weekly_guidance_prompt(sample_weekly_guidance_context())

    assert "Do not change or reinterpret ranking" in prompt.system_instructions
    assert "Do not invent trends" in prompt.system_instructions
    assert '"behavior_summary"' in prompt.context_payload
    assert '"funnel_summary"' in prompt.context_payload
    assert '"weekly_summary"' in prompt.output_schema_expectations
    assert '"next_week_plan"' in prompt.output_schema_expectations


def test_weekly_guidance_output_normalizes_lists_and_truncates():
    parsed = parse_weekly_guidance_output(
        {
            "weekly_summary": "  `Signals are strongest around backend searches`  ",
            "what_is_working": ["- Djinni saves are concentrated", "", "- Djinni saves are concentrated"],
            "what_is_not_working": None,
            "recommended_search_adjustments": [
                "1. Narrow titles around backend",
                "* Remove broad support terms",
            ],
            "recommended_source_moves": [
                "Focus on djinni",
                "Reduce work_ua time",
                "Review source mix weekly",
                "Keep one experimental source",
                "Use saved-job sources first",
                "Check application conversion by source",
                "Should be truncated",
            ],
            "recommended_role_focus": ["- backend_developer", "- platform_engineer"],
            "funnel_bottlenecks": ["* Saves are not turning into many applications"],
            "next_week_plan": ["1. Run 3 focused searches", "1. Run 3 focused searches"],
        }
    )

    assert parsed.weekly_summary == "Signals are strongest around backend searches"
    assert parsed.what_is_working == ["Djinni saves are concentrated"]
    assert parsed.what_is_not_working == []
    assert parsed.recommended_search_adjustments == [
        "Narrow titles around backend",
        "Remove broad support terms",
    ]
    assert len(parsed.recommended_source_moves) == 6
    assert parsed.recommended_role_focus == ["backend_developer", "platform_engineer"]
    assert parsed.funnel_bottlenecks == ["Saves are not turning into many applications"]
    assert parsed.next_week_plan == ["Run 3 focused searches"]


def test_weekly_guidance_output_rejects_malformed_provider_payload():
    try:
        parse_weekly_guidance_output({"weekly_summary": "ok", "what_is_working": "bad"})
    except MalformedWeeklyGuidanceOutputError:
        pass
    else:  # pragma: no cover - defensive
        raise AssertionError("expected malformed provider output to raise")


def test_weekly_guidance_endpoint_returns_valid_structure():
    app.dependency_overrides[get_weekly_guidance_service] = lambda: WeeklyGuidanceService(
        StubProvider(
            {
                "weekly_summary": "```json Search activity is producing the clearest traction in backend-focused searches and Djinni-sourced jobs. ```",
                "what_is_working": ["- Djinni is producing the strongest positive signals"],
                "what_is_not_working": ["* Work.ua is producing more hides and bad fits than saves"],
                "recommended_search_adjustments": ["1. Keep search terms tighter around backend and platform roles"],
                "recommended_source_moves": ["Focus most weekly effort on Djinni before adding lower-signal sources"],
                "recommended_role_focus": ["Stay centered on backend_developer and adjacent platform work"],
                "funnel_bottlenecks": ["Applications remain limited relative to saves"],
                "next_week_plan": ["Run focused backend searches, review saved jobs, and convert the strongest matches into applications"],
            }
        )
    )

    with TestClient(app) as client:
        response = client.post(
            "/v1/enrichment/weekly-guidance",
            json=sample_weekly_guidance_context().model_dump(by_alias=True, exclude_none=True),
        )

    app.dependency_overrides.clear()

    assert response.status_code == 200
    payload = WeeklyGuidanceResponse.model_validate(response.json())
    assert payload.weekly_summary == (
        "Search activity is producing the clearest traction in backend-focused searches and Djinni-sourced jobs."
    )
    assert payload.what_is_working == ["Djinni is producing the strongest positive signals"]
    assert payload.what_is_not_working == ["Work.ua is producing more hides and bad fits than saves"]


def test_weekly_guidance_endpoint_handles_malformed_provider_output_gracefully():
    app.dependency_overrides[get_weekly_guidance_service] = lambda: WeeklyGuidanceService(
        StubProvider({"weekly_summary": "ok", "what_is_working": "bad"})
    )

    with TestClient(app) as client:
        response = client.post(
            "/v1/enrichment/weekly-guidance",
            json=sample_weekly_guidance_context().model_dump(by_alias=True, exclude_none=True),
        )

    app.dependency_overrides.clear()

    assert response.status_code == 502
    assert response.json()["detail"] == "Weekly guidance provider returned malformed output."
