# ml

Python ML/LLM service for:
- job extraction
- fit analysis
- reranking
- profile enrichment
- future adapter-based model integration

## Current slice

This service now exposes a read-only Phase 9 integration layer:
- fetch canonical profile data from `engine-api`
- fetch a dedicated lifecycle-aware job payload from `engine-api`
- compute heuristic fit analysis without writing to Postgres
- rerank a provided list of jobs for a persisted profile
- enrich deterministic analytics context with structured profile insights
- generate structured weekly guidance grounded in deterministic analytics, behavior, and funnel summaries
- generate structured application coaching for a deterministically ranked job
- generate structured first-pass cover letter drafts grounded in deterministic job/profile context
- generate structured interview prep packs grounded in deterministic job/profile context

## Runtime

Environment variables:
- `PORT` default `8000`
- `ENGINE_API_BASE_URL` default `http://localhost:8080`
- `ENGINE_API_TIMEOUT_SECONDS` default `10`
- `ML_LLM_PROVIDER` default `template`, or `openai` when `OPENAI_API_KEY` is present
- `OPENAI_API_KEY` required for the OpenAI provider
- `OPENAI_MODEL` default `gpt-5.4-mini`
- `OPENAI_BASE_URL` optional override

Install dependencies:

```bash
cd apps/ml
python3 -m venv .venv
source .venv/bin/activate
pip install -r requirements.txt
```

Run the service:

```bash
cd apps/ml
PORT=8000 ENGINE_API_BASE_URL=http://localhost:8080 \
  uvicorn app.main:app --host 0.0.0.0 --port ${PORT:-8000}
```

## Endpoints

Health:

```bash
curl http://localhost:8000/health
```

Fit analysis for persisted canonical entities:

```bash
curl \
  -X POST http://localhost:8000/api/v1/fit/analyze \
  -H "Content-Type: application/json" \
  -d '{
    "profile_id": "profile_test_001",
    "job_id": "job_backend_rust_001"
  }'
```

Rerank a list of jobs for a persisted profile:

```bash
curl \
  -X POST http://localhost:8000/api/v1/rerank \
  -H "Content-Type: application/json" \
  -d '{
    "profile_id": "profile_test_001",
    "job_ids": [
      "job_backend_rust_001",
      "job_frontend_react_001"
    ]
  }'
```

Generate additive profile insights from deterministic analytics context:

```bash
curl \
  -X POST http://localhost:8000/v1/enrichment/profile-insights \
  -H "Content-Type: application/json" \
  -d '{
    "profile_id": "profile_test_001",
    "analyzed_profile": {
      "summary": "Senior backend engineer with Rust experience",
      "primary_role": "backend_developer",
      "seniority": "senior",
      "skills": ["rust", "postgres"],
      "keywords": ["backend", "distributed systems"]
    },
    "profile_skills": ["rust", "postgres"],
    "profile_keywords": ["backend", "distributed systems"],
    "jobs_feed_summary": {
      "total": 120,
      "active": 90,
      "inactive": 20,
      "reactivated": 10
    },
    "feedback_summary": {
      "saved_jobs_count": 6,
      "hidden_jobs_count": 2,
      "bad_fit_jobs_count": 1,
      "whitelisted_companies_count": 1,
      "blacklisted_companies_count": 0
    },
    "top_positive_evidence": [
      { "type": "saved_job", "label": "job_backend_rust_001" }
    ],
    "top_negative_evidence": []
  }'
```

Generate additive weekly guidance from deterministic analytics, behavior, and funnel summaries:

```bash
curl \
  -X POST http://localhost:8000/v1/enrichment/weekly-guidance \
  -H "Content-Type: application/json" \
  -d '{
    "profile_id": "profile_test_001",
    "analytics_summary": {
      "feedback": {
        "saved_jobs_count": 6,
        "hidden_jobs_count": 2,
        "bad_fit_jobs_count": 1,
        "whitelisted_companies_count": 1,
        "blacklisted_companies_count": 0
      },
      "jobs_by_source": [
        { "source": "djinni", "count": 42 }
      ],
      "jobs_by_lifecycle": {
        "total": 120,
        "active": 90,
        "inactive": 20,
        "reactivated": 10
      },
      "top_matched_roles": ["backend_developer"],
      "top_matched_skills": ["rust", "postgres"],
      "top_matched_keywords": ["backend", "distributed systems"]
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
          "net_score": 6
        }
      ],
      "top_negative_sources": [],
      "top_positive_role_families": [],
      "top_negative_role_families": [],
      "source_signal_counts": [],
      "role_family_signal_counts": []
    },
    "funnel_summary": {
      "impression_count": 30,
      "open_count": 12,
      "save_count": 4,
      "hide_count": 2,
      "bad_fit_count": 1,
      "application_created_count": 1,
      "fit_explanation_requested_count": 3,
      "application_coach_requested_count": 2,
      "cover_letter_draft_requested_count": 1,
      "interview_prep_requested_count": 1,
      "conversion_rates": {
        "open_rate_from_impressions": 0.4,
        "save_rate_from_opens": 0.333,
        "application_rate_from_saves": 0.25
      },
      "impressions_by_source": [{ "source": "djinni", "count": 20 }],
      "opens_by_source": [{ "source": "djinni", "count": 10 }],
      "saves_by_source": [{ "source": "djinni", "count": 4 }],
      "applications_by_source": [{ "source": "djinni", "count": 1 }]
    },
    "llm_context": {
      "analyzed_profile": {
        "summary": "Senior backend engineer with Rust experience",
        "primary_role": "backend_developer",
        "seniority": "senior",
        "skills": ["rust", "postgres"],
        "keywords": ["backend", "distributed systems"]
      },
      "profile_skills": ["rust", "postgres"],
      "profile_keywords": ["backend", "distributed systems"],
      "jobs_feed_summary": {
        "total": 120,
        "active": 90,
        "inactive": 20,
        "reactivated": 10
      },
      "feedback_summary": {
        "saved_jobs_count": 6,
        "hidden_jobs_count": 2,
        "bad_fit_jobs_count": 1,
        "whitelisted_companies_count": 1,
        "blacklisted_companies_count": 0
      },
      "top_positive_evidence": [
        { "type": "saved_job", "label": "job_backend_rust_001" }
      ],
      "top_negative_evidence": []
    }
  }'
```

Generate additive fit explanations for a deterministically ranked job:

```bash
curl \
  -X POST http://localhost:8000/v1/enrichment/job-fit-explanation \
  -H "Content-Type: application/json" \
  -d '{
    "profile_id": "profile_test_001",
    "analyzed_profile": {
      "summary": "Senior backend engineer with Rust experience",
      "primary_role": "backend_developer",
      "seniority": "senior",
      "skills": ["rust", "postgres"],
      "keywords": ["backend", "distributed systems"]
    },
    "search_profile": {
      "primary_role": "backend_developer",
      "primary_role_confidence": 0.92,
      "target_roles": ["backend_developer", "platform_engineer"],
      "role_candidates": [{ "role": "backend_developer", "confidence": 0.92 }],
      "seniority": "senior",
      "target_regions": ["eu_remote"],
      "work_modes": ["remote"],
      "allowed_sources": ["djinni"],
      "profile_skills": ["rust", "postgres"],
      "profile_keywords": ["backend", "distributed systems"],
      "search_terms": ["rust backend", "platform engineer"],
      "exclude_terms": ["php"]
    },
    "ranked_job": {
      "id": "job_backend_rust_001",
      "title": "Senior Rust Backend Engineer",
      "company_name": "Example",
      "description_text": "Own APIs and platform services.",
      "summary": "Remote backend role with Rust and Postgres.",
      "source": "djinni",
      "source_job_id": "source_123",
      "source_url": "https://example.com/job/123",
      "remote_type": "remote",
      "seniority": "senior",
      "salary_label": "$4,000 - $5,000",
      "location_label": "Remote EU",
      "work_mode_label": "Remote",
      "freshness_label": "Seen today",
      "badges": ["remote", "active"]
    },
    "deterministic_fit": {
      "job_id": "job_backend_rust_001",
      "score": 82,
      "matched_roles": ["backend_developer"],
      "matched_skills": ["rust", "postgres"],
      "matched_keywords": ["backend"],
      "source_match": true,
      "work_mode_match": true,
      "region_match": true,
      "reasons": ["Strong role overlap with backend_developer target."]
    },
    "feedback_state": {
      "summary": {
        "saved_jobs_count": 6,
        "hidden_jobs_count": 2,
        "bad_fit_jobs_count": 1,
        "whitelisted_companies_count": 1,
        "blacklisted_companies_count": 0
      },
      "top_positive_evidence": [{ "type": "saved_job", "label": "job_backend_rust_001" }],
      "top_negative_evidence": [],
      "current_job_feedback": {
        "saved": false,
        "hidden": false,
        "bad_fit": false,
        "company_status": "whitelist"
      }
    }
  }'
```

Generate additive application coaching for a deterministically ranked job:

```bash
curl \
  -X POST http://localhost:8000/v1/enrichment/application-coach \
  -H "Content-Type: application/json" \
  -d '{
    "profile_id": "profile_test_001",
    "analyzed_profile": {
      "summary": "Senior backend engineer with Rust experience",
      "primary_role": "backend_developer",
      "seniority": "senior",
      "skills": ["rust", "postgres"],
      "keywords": ["backend", "distributed systems"]
    },
    "search_profile": {
      "primary_role": "backend_developer",
      "primary_role_confidence": 0.92,
      "target_roles": ["backend_developer", "platform_engineer"],
      "role_candidates": [{ "role": "backend_developer", "confidence": 0.92 }],
      "seniority": "senior",
      "target_regions": ["eu_remote"],
      "work_modes": ["remote"],
      "allowed_sources": ["djinni"],
      "profile_skills": ["rust", "postgres"],
      "profile_keywords": ["backend", "distributed systems"],
      "search_terms": ["rust backend", "platform engineer"],
      "exclude_terms": ["php"]
    },
    "ranked_job": {
      "id": "job_backend_rust_001",
      "title": "Senior Rust Backend Engineer",
      "company_name": "Example",
      "description_text": "Own APIs and platform services.",
      "summary": "Remote backend role with Rust and Postgres.",
      "source": "djinni",
      "source_job_id": "source_123",
      "source_url": "https://example.com/job/123",
      "remote_type": "remote",
      "seniority": "senior",
      "salary_label": "$4,000 - $5,000",
      "location_label": "Remote EU",
      "work_mode_label": "Remote",
      "freshness_label": "Seen today",
      "badges": ["remote", "active"]
    },
    "deterministic_fit": {
      "job_id": "job_backend_rust_001",
      "score": 82,
      "matched_roles": ["backend_developer"],
      "matched_skills": ["rust", "postgres"],
      "matched_keywords": ["backend"],
      "source_match": true,
      "work_mode_match": true,
      "region_match": true,
      "reasons": ["Strong role overlap with backend_developer target."]
    },
    "job_fit_explanation": {
      "fit_summary": "Strong deterministic fit for backend platform work.",
      "why_it_matches": ["Role and skill overlap are both explicit."],
      "risks": ["Keyword depth is narrower than the full job scope."],
      "missing_signals": ["Leadership evidence is not explicit in the deterministic context."],
      "recommended_next_step": "Tailor the opening bullets to Rust and Postgres work.",
      "application_angle": "Lead with backend platform ownership already evidenced in the profile."
    },
    "feedback_state": {
      "summary": {
        "saved_jobs_count": 6,
        "hidden_jobs_count": 2,
        "bad_fit_jobs_count": 1,
        "whitelisted_companies_count": 1,
        "blacklisted_companies_count": 0
      },
      "top_positive_evidence": [{ "type": "saved_job", "label": "job_backend_rust_001" }],
      "top_negative_evidence": [],
      "current_job_feedback": {
        "saved": false,
        "hidden": false,
        "bad_fit": false,
        "company_status": "whitelist"
      }
    },
    "raw_profile_text": "Senior backend engineer with Rust, Postgres, and platform delivery experience."
  }'
```

Generate an additive cover letter draft for a deterministically ranked job:

```bash
curl \
  -X POST http://localhost:8000/v1/enrichment/cover-letter-draft \
  -H "Content-Type: application/json" \
  -d '{
    "profile_id": "profile_test_001",
    "analyzed_profile": {
      "summary": "Senior backend engineer with Rust experience",
      "primary_role": "backend_developer",
      "seniority": "senior",
      "skills": ["rust", "postgres"],
      "keywords": ["backend", "distributed systems"]
    },
    "search_profile": {
      "primary_role": "backend_developer",
      "primary_role_confidence": 0.92,
      "target_roles": ["backend_developer", "platform_engineer"],
      "role_candidates": [{ "role": "backend_developer", "confidence": 0.92 }],
      "seniority": "senior",
      "target_regions": ["eu_remote"],
      "work_modes": ["remote"],
      "allowed_sources": ["djinni"],
      "profile_skills": ["rust", "postgres"],
      "profile_keywords": ["backend", "distributed systems"],
      "search_terms": ["rust backend", "platform engineer"],
      "exclude_terms": ["php"]
    },
    "ranked_job": {
      "id": "job_backend_rust_001",
      "title": "Senior Rust Backend Engineer",
      "company_name": "Example",
      "description_text": "Own APIs and platform services.",
      "summary": "Remote backend role with Rust and Postgres.",
      "source": "djinni",
      "source_job_id": "source_123",
      "source_url": "https://example.com/job/123",
      "remote_type": "remote",
      "seniority": "senior",
      "salary_label": "$4,000 - $5,000",
      "location_label": "Remote EU",
      "work_mode_label": "Remote",
      "freshness_label": "Seen today",
      "badges": ["remote", "active"]
    },
    "deterministic_fit": {
      "job_id": "job_backend_rust_001",
      "score": 82,
      "matched_roles": ["backend_developer"],
      "matched_skills": ["rust", "postgres"],
      "matched_keywords": ["backend"],
      "source_match": true,
      "work_mode_match": true,
      "region_match": true,
      "reasons": ["Strong role overlap with backend_developer target."]
    },
    "job_fit_explanation": {
      "fit_summary": "Strong deterministic fit for backend platform work.",
      "why_it_matches": ["Role and skill overlap are both explicit."],
      "risks": ["Keyword depth is narrower than the full job scope."],
      "missing_signals": ["Leadership evidence is not explicit in the deterministic context."],
      "recommended_next_step": "Tailor the opening bullets to Rust and Postgres work.",
      "application_angle": "Lead with backend platform ownership already evidenced in the profile."
    },
    "application_coach": {
      "application_summary": "Tailor this application around the existing Rust backend evidence.",
      "resume_focus_points": ["Move Rust and Postgres near the top."],
      "suggested_bullets": ["Reframe existing backend platform work."],
      "cover_letter_angles": ["Connect platform ownership to the job summary."],
      "interview_focus": ["Prepare Rust service examples."],
      "gaps_to_address": ["Leadership evidence is not explicit in the deterministic context."],
      "red_flags": ["Do not claim unsupported achievements."]
    },
    "feedback_state": {
      "summary": {
        "saved_jobs_count": 6,
        "hidden_jobs_count": 2,
        "bad_fit_jobs_count": 1,
        "whitelisted_companies_count": 1,
        "blacklisted_companies_count": 0
      },
      "top_positive_evidence": [{ "type": "saved_job", "label": "job_backend_rust_001" }],
      "top_negative_evidence": [],
      "current_job_feedback": {
        "saved": false,
        "hidden": false,
        "bad_fit": false,
        "company_status": "whitelist"
      }
    },
    "raw_profile_text": "Senior backend engineer with Rust, Postgres, and platform delivery experience."
  }'
```

Generate an additive interview prep pack for a deterministically ranked job:

```bash
curl \
  -X POST http://localhost:8000/v1/enrichment/interview-prep \
  -H "Content-Type: application/json" \
  -d '{
    "profile_id": "profile_test_001",
    "analyzed_profile": {
      "summary": "Senior backend engineer with Rust experience",
      "primary_role": "backend_developer",
      "seniority": "senior",
      "skills": ["rust", "postgres"],
      "keywords": ["backend", "distributed systems"]
    },
    "search_profile": {
      "primary_role": "backend_developer",
      "primary_role_confidence": 0.92,
      "target_roles": ["backend_developer", "platform_engineer"],
      "role_candidates": [{ "role": "backend_developer", "confidence": 0.92 }],
      "seniority": "senior",
      "target_regions": ["eu_remote"],
      "work_modes": ["remote"],
      "allowed_sources": ["djinni"],
      "profile_skills": ["rust", "postgres"],
      "profile_keywords": ["backend", "distributed systems"],
      "search_terms": ["rust backend", "platform engineer"],
      "exclude_terms": ["php"]
    },
    "ranked_job": {
      "id": "job_backend_rust_001",
      "title": "Senior Rust Backend Engineer",
      "company_name": "Example",
      "description_text": "Own APIs and platform services.",
      "summary": "Remote backend role with Rust and Postgres.",
      "source": "djinni",
      "source_job_id": "source_123",
      "source_url": "https://example.com/job/123",
      "remote_type": "remote",
      "seniority": "senior",
      "salary_label": "$4,000 - $5,000",
      "location_label": "Remote EU",
      "work_mode_label": "Remote",
      "freshness_label": "Seen today",
      "badges": ["remote", "active"]
    },
    "deterministic_fit": {
      "job_id": "job_backend_rust_001",
      "score": 82,
      "matched_roles": ["backend_developer"],
      "matched_skills": ["rust", "postgres"],
      "matched_keywords": ["backend"],
      "source_match": true,
      "work_mode_match": true,
      "region_match": true,
      "reasons": ["Strong role overlap with backend_developer target."]
    },
    "job_fit_explanation": {
      "fit_summary": "Strong deterministic fit for backend platform work.",
      "why_it_matches": ["Role and skill overlap are both explicit."],
      "risks": ["Keyword depth is narrower than the full job scope."],
      "missing_signals": ["Leadership evidence is not explicit in the deterministic context."],
      "recommended_next_step": "Tailor the opening bullets to Rust and Postgres work.",
      "application_angle": "Lead with backend platform ownership already evidenced in the profile."
    },
    "application_coach": {
      "application_summary": "Tailor this application around the existing Rust backend evidence.",
      "resume_focus_points": ["Move Rust and Postgres near the top."],
      "suggested_bullets": ["Reframe existing backend platform work."],
      "cover_letter_angles": ["Connect platform ownership to the job summary."],
      "interview_focus": ["Prepare Rust service examples."],
      "gaps_to_address": ["Leadership evidence is not explicit in the deterministic context."],
      "red_flags": ["Do not claim unsupported achievements."]
    },
    "cover_letter_draft": {
      "draft_summary": "Ground the letter in explicit backend overlap already present in the profile context.",
      "opening_paragraph": "Lead with grounded Rust backend overlap.",
      "body_paragraphs": ["The deterministic fit already highlights strong role overlap and matched skills."],
      "closing_paragraph": "Close with interest in discussing verified backend evidence.",
      "key_claims_used": ["Senior backend engineer with Rust experience"],
      "evidence_gaps": ["Leadership evidence is not explicit in the deterministic context."],
      "tone_notes": ["Keep the tone direct and evidence-based."]
    },
    "feedback_state": {
      "summary": {
        "saved_jobs_count": 6,
        "hidden_jobs_count": 2,
        "bad_fit_jobs_count": 1,
        "whitelisted_companies_count": 1,
        "blacklisted_companies_count": 0
      },
      "top_positive_evidence": [{ "type": "saved_job", "label": "job_backend_rust_001" }],
      "top_negative_evidence": [],
      "current_job_feedback": {
        "saved": false,
        "hidden": false,
        "bad_fit": false,
        "company_status": "whitelist"
      }
    },
    "raw_profile_text": "Senior backend engineer with Rust, Postgres, and platform delivery experience."
  }'
```

## Rules

- `ml` does not write canonical job, profile, or application data
- `engine-api` remains the only write authority
- this service consumes `engine-api` over HTTP as a sidecar
- `app/engine_api_client.py` is the only place that knows the ML read-only engine-api surface
- structured enrichment is additive only and must not invent canonical IDs
