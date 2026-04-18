from app.engine_api_client import (
    EngineJobLifecycle,
    EngineJobLifecycleVariant,
    EngineJobPresentation,
    EngineProfile,
)
from app.main import normalize_text, score_job, tokenize


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
            "location": "Remote, Europe",
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
            "presentation": EngineJobPresentation.model_validate(
                {
                    "title": "Senior Platform Engineer",
                    "company": "SignalHire",
                    "summary": "Build Rust services and Postgres-backed ingestion flows.",
                    "location_label": "Europe",
                    "work_mode_label": "Remote",
                    "source_label": "Mock source",
                    "outbound_url": "https://mock-source.example/jobs/platform-001",
                    "salary_label": "5,000-6,500 USD",
                    "freshness_label": "Posted 2026-04-14",
                    "badges": ["Remote", "Senior", "Reactivated"],
                }
            ),
        }
    )

    score, matched_terms, _missing_terms, evidence = score_job(profile, job)

    assert score > 0
    assert "rust" in matched_terms
    assert "lifecycle: reactivated" in evidence


def test_normalize_text_canonicalizes_compound_terms_and_aliases():
    assert normalize_text("Front-end React Developer") == "frontend react developer"
    assert normalize_text("Front end React Developer") == "frontend react developer"
    assert normalize_text("Back-end Node.js Engineer") == "backend nodejs engineer"
    assert normalize_text("Full stack C# / C++") == "fullstack csharp cpp"
    assert normalize_text("React Native distributed systems") == "react_native distributed_systems"


def test_tokenize_preserves_meaningful_phrases_without_fragmenting_frontend():
    terms = tokenize(
        "Senior Front-end React Developer with React Native and distributed systems, quality assurance, and Google Ads work"
    )

    assert "frontend" in terms
    assert "react native" in terms
    assert "distributed systems" in terms
    assert "quality assurance" in terms
    assert "google ads" in terms
    assert "front" not in terms
    assert "end" not in terms


def test_score_job_evidence_uses_canonical_frontend_terms():
    profile = EngineProfile.model_validate(
        {
            "id": "profile-frontend-1",
            "name": "Frontend User",
            "email": "frontend@example.com",
            "raw_text": "Senior front-end developer with React, TypeScript and design system experience",
            "analysis": {
                "summary": "Senior frontend engineer",
                "primary_role": "frontend_developer",
                "seniority": "senior",
                "skills": ["React", "TypeScript"],
                "keywords": ["frontend", "design system"],
            },
            "created_at": "2026-04-14T08:00:00Z",
            "updated_at": "2026-04-14T08:00:00Z",
            "skills_updated_at": "2026-04-14T08:00:00Z",
        }
    )
    job = EngineJobLifecycle.model_validate(
        {
            "id": "job-frontend-1",
            "title": "Senior Front-end React Developer",
            "company_name": "SignalHire",
            "location": "Remote, Europe",
            "remote_type": "remote",
            "seniority": "senior",
            "description_text": "Ship frontend design system features with React and TypeScript.",
            "salary_min": 5000,
            "salary_max": 6500,
            "salary_currency": "USD",
            "posted_at": "2026-04-14T08:00:00Z",
            "first_seen_at": "2026-04-14T08:00:00Z",
            "last_seen_at": "2026-04-16T09:00:00Z",
            "is_active": True,
            "inactivated_at": None,
            "reactivated_at": None,
            "lifecycle_stage": "active",
            "primary_variant": EngineJobLifecycleVariant.model_validate(
                {
                    "source": "djinni",
                    "source_job_id": "frontend-001",
                    "source_url": "https://djinni.co/jobs/frontend-001",
                    "fetched_at": "2026-04-16T09:00:00Z",
                    "last_seen_at": "2026-04-16T09:00:00Z",
                    "is_active": True,
                    "inactivated_at": None,
                }
            ),
            "presentation": EngineJobPresentation.model_validate(
                {
                    "title": "Senior Front-end React Developer",
                    "company": "SignalHire",
                    "summary": "Ship frontend design system features with React and TypeScript.",
                    "location_label": "Europe",
                    "work_mode_label": "Remote",
                    "source_label": "Djinni",
                    "outbound_url": "https://djinni.co/jobs/frontend-001",
                    "salary_label": "5,000-6,500 USD",
                    "freshness_label": "Posted 2026-04-14",
                    "badges": ["Remote", "Senior"],
                }
            ),
        }
    )

    score, matched_terms, _missing_terms, evidence = score_job(profile, job)

    assert score > 0
    assert "frontend" in matched_terms
    assert "front" not in matched_terms
    assert "end" not in matched_terms
    assert any("title overlap: frontend" in line for line in evidence)
    assert all("front, end" not in line for line in evidence)
