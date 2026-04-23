import asyncio
import logging

import pytest

from app.profile_insights import LlmContextRequest, ProfileInsightsProviderError
from app.profile_insights_service import ProfileInsightsService


def sample_context() -> LlmContextRequest:
    return LlmContextRequest.model_validate(
        {
            "profile_id": "profile-1",
            "analyzed_profile": {
                "summary": "Senior backend engineer with Rust and internal platform experience",
                "primary_role": "backend_developer",
                "seniority": "senior",
                "skills": ["Rust", "Postgres"],
                "keywords": ["backend", "platform"],
            },
            "profile_skills": ["Rust", "Postgres"],
            "profile_keywords": ["backend", "platform"],
            "jobs_feed_summary": {
                "total": 10,
                "active": 8,
                "inactive": 1,
                "reactivated": 1,
            },
            "feedback_summary": {
                "saved_jobs_count": 2,
                "hidden_jobs_count": 0,
                "bad_fit_jobs_count": 0,
                "whitelisted_companies_count": 0,
                "blacklisted_companies_count": 0,
            },
        }
    )


class SuccessfulProvider:
    async def generate_profile_insights(self, context, prompt):
        return {
            "profile_summary": "Strong backend profile",
            "search_strategy_summary": "Stay close to backend roles",
            "strengths": ["Rust depth"],
            "risks": [],
            "recommended_actions": ["Tighten search terms"],
            "top_focus_areas": ["Rust"],
            "search_term_suggestions": ["backend engineer"],
            "application_strategy": ["Apply where Rust appears early"],
        }


class FailingProvider:
    async def generate_profile_insights(self, context, prompt):
        raise ProfileInsightsProviderError(
            "Sensitive prompt leak: Senior backend engineer with Rust and internal platform experience"
        )


def test_enrichment_success_logs_metadata_without_payload(caplog):
    caplog.set_level(logging.INFO)

    result = asyncio.run(ProfileInsightsService(SuccessfulProvider()).enrich(sample_context()))

    assert result.profile_summary == "Strong backend profile"
    records = [record for record in caplog.records if record.message == "enrichment call completed"]
    assert records
    record = records[-1]
    assert record.flow == "profile_insights"
    assert record.provider == "SuccessfulProvider"
    assert record.profile_id == "profile-1"
    assert record.success is True
    assert record.context_bytes > 0
    assert "Senior backend engineer with Rust and internal platform experience" not in caplog.text
    assert '"analyzed_profile"' not in caplog.text


def test_enrichment_failure_logs_error_type_without_sensitive_text(caplog):
    caplog.set_level(logging.WARNING)

    with pytest.raises(ProfileInsightsProviderError):
        asyncio.run(ProfileInsightsService(FailingProvider()).enrich(sample_context()))

    records = [record for record in caplog.records if record.message == "enrichment call failed"]
    assert records
    record = records[-1]
    assert record.flow == "profile_insights"
    assert record.provider == "FailingProvider"
    assert record.profile_id == "profile-1"
    assert record.success is False
    assert record.error_type == "ProfileInsightsProviderError"
    assert "Sensitive prompt leak" not in caplog.text
    assert "Senior backend engineer with Rust and internal platform experience" not in caplog.text
