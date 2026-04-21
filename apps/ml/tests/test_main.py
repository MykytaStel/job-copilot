from app.engine_api_client import (
    EngineJobLifecycle,
    EngineJobLifecycleVariant,
    EngineJobPresentation,
    EngineProfile,
)
from app.scoring import score_job
from app.text_normalization import normalize_text, tokenize


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


def test_score_job_prefers_frontend_react_overlap_over_generic_ui_overlap():
    profile = EngineProfile.model_validate(
        {
            "id": "profile-frontend-2",
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
    strong_job = EngineJobLifecycle.model_validate(
        {
            "id": "job-frontend-strong",
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
                    "source_job_id": "frontend-strong-001",
                    "source_url": "https://djinni.co/jobs/frontend-strong-001",
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
                    "outbound_url": "https://djinni.co/jobs/frontend-strong-001",
                    "salary_label": "5,000-6,500 USD",
                    "freshness_label": "Posted 2026-04-14",
                    "badges": ["Remote", "Senior"],
                }
            ),
        }
    )
    weak_job = EngineJobLifecycle.model_validate(
        {
            "id": "job-frontend-weak",
            "title": "Senior UI Engineer",
            "company_name": "SignalHire",
            "location": "Remote, Europe",
            "remote_type": "remote",
            "seniority": "senior",
            "description_text": "Improve shared product experiences and collaborate with design.",
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
                    "source_job_id": "frontend-weak-001",
                    "source_url": "https://djinni.co/jobs/frontend-weak-001",
                    "fetched_at": "2026-04-16T09:00:00Z",
                    "last_seen_at": "2026-04-16T09:00:00Z",
                    "is_active": True,
                    "inactivated_at": None,
                }
            ),
            "presentation": EngineJobPresentation.model_validate(
                {
                    "title": "Senior UI Engineer",
                    "company": "SignalHire",
                    "summary": "Improve shared product experiences and collaborate with design.",
                    "location_label": "Europe",
                    "work_mode_label": "Remote",
                    "source_label": "Djinni",
                    "outbound_url": "https://djinni.co/jobs/frontend-weak-001",
                    "salary_label": "5,000-6,500 USD",
                    "freshness_label": "Posted 2026-04-14",
                    "badges": ["Remote", "Senior"],
                }
            ),
        }
    )

    strong_score, strong_terms, _missing_terms, strong_evidence = score_job(profile, strong_job)
    weak_score, _weak_terms, _missing_terms, _weak_evidence = score_job(profile, weak_job)

    assert strong_score > weak_score
    assert "frontend" in strong_terms
    assert "react" in strong_terms
    assert any("title overlap: frontend, react" in line for line in strong_evidence)


def test_score_job_missing_terms_do_not_leak_profile_pii_when_analysis_exists():
    profile = EngineProfile.model_validate(
        {
            "id": "profile-pii-analysis",
            "name": "Mykyta Stelmashenko",
            "email": "mykytastelmashenko@gmail.com",
            "location": "Kyiv, Ukraine",
            "raw_text": (
                "Mykyta Stelmashenko\n"
                "Kyiv, Ukraine\n"
                "+380734784958\n"
                "Senior frontend developer with React Native experience"
            ),
            "analysis": {
                "summary": "Senior frontend developer",
                "primary_role": "frontend_developer",
                "seniority": "senior",
                "skills": ["React Native", "React"],
                "keywords": ["frontend"],
            },
            "created_at": "2026-04-14T08:00:00Z",
            "updated_at": "2026-04-14T08:00:00Z",
            "skills_updated_at": "2026-04-14T08:00:00Z",
        }
    )
    job = EngineJobLifecycle.model_validate(
        {
            "id": "job-pii-analysis",
            "title": "Senior Frontend React Developer",
            "company_name": "SignalHire",
            "location": "Remote, Europe",
            "remote_type": "remote",
            "seniority": "senior",
            "description_text": "Build frontend features with React.",
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
                    "source_job_id": "frontend-pii-analysis-001",
                    "source_url": "https://djinni.co/jobs/frontend-pii-analysis-001",
                    "fetched_at": "2026-04-16T09:00:00Z",
                    "last_seen_at": "2026-04-16T09:00:00Z",
                    "is_active": True,
                    "inactivated_at": None,
                }
            ),
            "presentation": EngineJobPresentation.model_validate(
                {
                    "title": "Senior Frontend React Developer",
                    "company": "SignalHire",
                    "summary": "Build frontend features with React.",
                    "location_label": "Europe",
                    "work_mode_label": "Remote",
                    "source_label": "Djinni",
                    "outbound_url": "https://djinni.co/jobs/frontend-pii-analysis-001",
                    "salary_label": "5,000-6,500 USD",
                    "freshness_label": "Posted 2026-04-14",
                    "badges": ["Remote", "Senior"],
                }
            ),
        }
    )

    _score, _matched_terms, missing_terms, _evidence = score_job(profile, job)

    assert "react native" in missing_terms
    assert "mykyta" not in missing_terms
    assert "stelmashenko" not in missing_terms
    assert "kyiv" not in missing_terms
    assert "ukraine" not in missing_terms
    assert "380734784958" not in missing_terms
    assert "mykytastelmashenko" not in missing_terms


def test_score_job_missing_terms_do_not_leak_profile_pii_without_analysis():
    profile = EngineProfile.model_validate(
        {
            "id": "profile-pii-fallback",
            "name": "Mykyta Stelmashenko",
            "email": "mykytastelmashenko@gmail.com",
            "location": "Kyiv, Ukraine",
            "raw_text": (
                "Mykyta Stelmashenko\n"
                "Kyiv, Ukraine\n"
                "Phone: +380734784958\n"
                "Senior frontend developer with React Native experience"
            ),
            "analysis": None,
            "created_at": "2026-04-14T08:00:00Z",
            "updated_at": "2026-04-14T08:00:00Z",
            "skills_updated_at": "2026-04-14T08:00:00Z",
        }
    )
    job = EngineJobLifecycle.model_validate(
        {
            "id": "job-pii-fallback",
            "title": "Senior Frontend React Developer",
            "company_name": "SignalHire",
            "location": "Remote, Europe",
            "remote_type": "remote",
            "seniority": "senior",
            "description_text": "Build frontend features with React.",
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
                    "source_job_id": "frontend-pii-fallback-001",
                    "source_url": "https://djinni.co/jobs/frontend-pii-fallback-001",
                    "fetched_at": "2026-04-16T09:00:00Z",
                    "last_seen_at": "2026-04-16T09:00:00Z",
                    "is_active": True,
                    "inactivated_at": None,
                }
            ),
            "presentation": EngineJobPresentation.model_validate(
                {
                    "title": "Senior Frontend React Developer",
                    "company": "SignalHire",
                    "summary": "Build frontend features with React.",
                    "location_label": "Europe",
                    "work_mode_label": "Remote",
                    "source_label": "Djinni",
                    "outbound_url": "https://djinni.co/jobs/frontend-pii-fallback-001",
                    "salary_label": "5,000-6,500 USD",
                    "freshness_label": "Posted 2026-04-14",
                    "badges": ["Remote", "Senior"],
                }
            ),
        }
    )

    _score, _matched_terms, missing_terms, _evidence = score_job(profile, job)

    assert "react native" in missing_terms
    assert "mykyta" not in missing_terms
    assert "stelmashenko" not in missing_terms
    assert "kyiv" not in missing_terms
    assert "ukraine" not in missing_terms
    assert "phone" not in missing_terms
    assert "380734784958" not in missing_terms
    assert "mykytastelmashenko" not in missing_terms
