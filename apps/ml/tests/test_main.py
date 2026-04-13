from app.engine_api_client import EngineJobLifecycle, EngineJobLifecycleVariant, EngineProfile
from app.main import score_job


def test_score_job_uses_lifecycle_payload():
    profile = EngineProfile.model_validate(
        {
            "id": "profile-1",
            "name": "Test User",
            "email": "test@example.com",
            "raw_text": "Senior platform engineer with Rust and Postgres experience",
            "analysis": {
                "summary": "Senior platform engineer",
                "primary_role": "platform engineer",
                "seniority": "senior",
                "skills": ["Rust", "Postgres"],
                "keywords": ["platform", "backend"],
            },
            "created_at": "2026-04-14T08:00:00Z",
            "updated_at": "2026-04-14T08:00:00Z",
            "skills_updated_at": "2026-04-14T08:00:00Z",
        }
    )
    job = EngineJobLifecycle.model_validate(
        {
            "id": "job-1",
            "title": "Senior Platform Engineer",
            "company_name": "SignalHire",
            "remote_type": "remote",
            "seniority": "senior",
            "description_text": "Build Rust services and Postgres-backed ingestion flows.",
            "salary_min": 5000,
            "salary_max": 6500,
            "salary_currency": "USD",
            "posted_at": "2026-04-14T08:00:00Z",
            "first_seen_at": "2026-04-14T08:00:00Z",
            "last_seen_at": "2026-04-16T09:00:00Z",
            "is_active": True,
            "inactivated_at": None,
            "reactivated_at": "2026-04-16T09:00:00Z",
            "lifecycle_stage": "reactivated",
            "primary_variant": EngineJobLifecycleVariant.model_validate(
                {
                    "source": "mock_source",
                    "source_job_id": "platform-001",
                    "source_url": "https://mock-source.example/jobs/platform-001",
                    "fetched_at": "2026-04-16T09:00:00Z",
                    "last_seen_at": "2026-04-16T09:00:00Z",
                    "is_active": True,
                    "inactivated_at": None,
                }
            ),
        }
    )

    score, matched_terms, _missing_terms, evidence = score_job(profile, job)

    assert score > 0
    assert "rust" in matched_terms
    assert "lifecycle: reactivated" in evidence
