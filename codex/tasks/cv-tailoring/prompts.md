# Block C — CV Tailoring (8 tasks)

Each task below is a self-contained Codex prompt. Implement in order C1 → C8.

---

## C1 — ML endpoint: POST /v1/cv-tailoring

```
You are working in the Job Copilot monorepo.
First read: CLAUDE.md, apps/ml/app/enrichment_routes.py,
apps/ml/app/services/ or apps/ml/app/ (enrichment service pattern),
apps/ml/app/models/ (request/response DTOs)

## Goal
Add a new FastAPI endpoint POST /v1/cv-tailoring (also aliased as /api/v1/cv-tailoring)
that accepts:
{
  "profile_id": "uuid",
  "job_id": "uuid",
  "profile_summary": "string",
  "candidate_skills": ["skill1", "skill2"],
  "job_title": "string",
  "job_description": "string",
  "job_required_skills": ["skill1", "skill2"],
  "job_nice_to_have_skills": ["skill1"],
  "candidate_cv_text": "string (optional)"
}
and returns:
{
  "suggestions": {
    "skills_to_highlight": ["skill1", "skill2"],
    "skills_to_mention": ["skill3"],
    "gaps_to_address": [{"skill": "skill4", "suggestion": "Consider adding..."}],
    "summary_rewrite": "Suggested summary paragraph...",
    "key_phrases": ["phrase1", "phrase2"]
  },
  "provider": "template|ollama|openai",
  "generated_at": "ISO8601"
}

Implement using the existing provider pattern (TemplateEnrichmentProvider by default).
Protect with ML_INTERNAL_TOKEN header check (same as other ML endpoints).

## Inspect first
- apps/ml/app/enrichment_routes.py — how existing enrichment endpoints are structured
- apps/ml/app/providers/ or wherever TemplateEnrichmentProvider lives
- apps/ml/app/models/ or apps/ml/app/api_models.py — DTO patterns
- apps/ml/app/auth.py or security (internal token check)

## Likely files to modify
- apps/ml/app/enrichment_routes.py (add new route)
- New file: apps/ml/app/services/cv_tailoring_service.py
- New file: apps/ml/app/templates/template_cv_tailoring.py
- apps/ml/app/models/ (add CvTailoringRequest, CvTailoringResponse DTOs)

## Rules
- Protect with the same internal token middleware as other enrichment endpoints.
- Default provider is template — no paid API calls.
- Never log candidate CV text or job description (PII risk).
- Match existing route registration pattern.

## Acceptance criteria
- [ ] POST /api/v1/cv-tailoring returns 200 with correct response shape
- [ ] 401 if internal token missing/invalid
- [ ] Template provider returns non-empty suggestions
- [ ] ML pytest passes

## Verification commands
cd apps/ml && python -m pytest tests/test_cv_tailoring.py -v
cd apps/ml && python -m pytest

## Final response format
## What changed
## Why
## Files changed
## Verification
## Notes / follow-ups
```

---

## C2 — CV tailoring service class

```
You are working in the Job Copilot monorepo.
First read: CLAUDE.md, apps/ml/app/services/ (existing service patterns),
apps/ml/app/providers/ (TemplateEnrichmentProvider pattern)

## Goal
Implement CvTailoringService class that:
1. Takes profile data + job data as structured input
2. Delegates to the configured LLM provider
3. Returns structured CvTailoring suggestions
4. Has methods: analyze_fit_gaps(), suggest_highlights(), suggest_summary_rewrite()

This is the service layer used by the C1 endpoint.

## Inspect first
- apps/ml/app/services/ — existing service classes (e.g. FitAnalyzerService or similar)
- apps/ml/app/providers/ — provider interface
- apps/ml/app/models/ or api_models.py — DTO definitions

## Likely files to modify
- apps/ml/app/services/cv_tailoring_service.py (create)

## Rules
- Dependency-injected provider (not hardcoded).
- Separate business logic from HTTP layer.
- Unit-testable without HTTP.

## Acceptance criteria
- [ ] CvTailoringService can be instantiated with any provider
- [ ] analyze_fit_gaps returns list of missing skills with suggestions
- [ ] suggest_highlights returns skills to emphasize
- [ ] suggest_summary_rewrite returns a string
- [ ] Unit tests via pytest

## Verification commands
cd apps/ml && python -m pytest tests/test_cv_tailoring_service.py -v

## Final response format
## What changed
## Why
## Files changed
## Verification
## Notes / follow-ups
```

---

## C3 — CV tailoring template provider

```
You are working in the Job Copilot monorepo.
First read: CLAUDE.md, apps/ml/app/templates/ (existing template files),
apps/ml/app/providers/template_enrichment_provider.py (or similar)

## Goal
Implement template_cv_tailoring.py that provides deterministic (non-LLM) CV tailoring
suggestions based on rule-based skill matching:

1. skills_to_highlight: intersection of candidate_skills and job_required_skills
2. skills_to_mention: intersection of candidate_skills and job_nice_to_have_skills
3. gaps_to_address: job_required_skills minus candidate_skills, each with a standard
   suggestion ("Consider adding X to your CV if you have experience with it")
4. summary_rewrite: template string that inserts top 3 matching skills and job title
5. key_phrases: extract nouns/tech terms from job description (simple word frequency)

## Inspect first
- apps/ml/app/templates/ — existing template files (e.g. template_job_fit_explanation.py)
- apps/ml/app/providers/template_enrichment_provider.py — how templates are called

## Likely files to modify
- apps/ml/app/templates/template_cv_tailoring.py (create)
- apps/ml/app/providers/template_enrichment_provider.py (register new template)

## Rules
- No external NLP library — use stdlib string operations.
- Output should be useful even with minimal input (graceful degradation).
- Do not include generic filler — if no skill match, say "No strong matches found".

## Acceptance criteria
- [ ] skills_to_highlight is correct intersection
- [ ] gaps_to_address has a suggestion string per missing required skill
- [ ] summary_rewrite is a complete sentence referencing top skills + role
- [ ] key_phrases returns 5-10 terms
- [ ] Unit tests for all methods

## Verification commands
cd apps/ml && python -m pytest tests/test_cv_tailoring*.py -v

## Final response format
## What changed
## Why
## Files changed
## Verification
## Notes / follow-ups
```

---

## C4 — Engine-API proxy route for CV tailoring

```
You are working in the Job Copilot monorepo.
First read: CLAUDE.md, apps/engine-api/src/api/routes/ (existing proxy patterns),
apps/engine-api/src/services/ (ML client service),
apps/engine-api/src/api/routes/jobs.rs (how job data is fetched and enriched)

## Goal
Add POST /api/v1/cv/tailor to engine-api. This route:
1. Requires JWT auth (AuthUser)
2. Accepts { job_id: Uuid } in request body
3. Fetches the job details from DB
4. Fetches the candidate profile from DB
5. Calls ML sidecar POST /api/v1/cv-tailoring with assembled payload
6. Returns the ML response to the client

This keeps ML internal — the web never calls ML directly.

## Inspect first
- apps/engine-api/src/api/routes/ — find an existing ML proxy route pattern
  (look for where engine calls ML sidecar, e.g. fit analyze or rerank)
- apps/engine-api/src/services/ — ML client (reqwest HTTP calls to ML)
- apps/engine-api/src/api/routes/mod.rs — route registration

## Likely files to modify
- New file: apps/engine-api/src/api/routes/cv_tailoring.rs
- apps/engine-api/src/api/routes/mod.rs (register route)
- apps/engine-api/src/services/ml_client.rs or similar (add cv_tailor method)

## Rules
- Never expose ML internal token to the web client.
- 404 if job_id not found.
- 403 if job is not accessible to this profile.
- Propagate ML errors as 502 with a safe message.
- Add route-level test.

## Acceptance criteria
- [ ] POST /api/v1/cv/tailor requires valid JWT
- [ ] Returns ML suggestions for a valid job_id
- [ ] 404 for unknown job_id
- [ ] 502 if ML is unreachable with descriptive error
- [ ] Cargo tests pass

## Verification commands
cargo test --manifest-path apps/engine-api/Cargo.toml
cargo check --manifest-path apps/engine-api/Cargo.toml

## Final response format
## What changed
## Why
## Files changed
## Verification
## Notes / follow-ups
```

---

## C5 — Web: CV Tailoring modal in Job Detail

```
You are working in the Job Copilot monorepo.
First read: CLAUDE.md, apps/web/src/pages/JobDetail.tsx,
apps/web/src/components/job-detail/ (tabs, panels),
apps/web/src/api/ (engine client)

## Goal
Add a "Tailor CV" button in the Job Detail page. On click, open a modal/panel that
calls POST /api/v1/cv/tailor and shows the suggestions (C6 handles the display).

The button should be in the job detail header or action bar alongside "Save", "Apply" etc.
While loading, show a spinner inside the modal. On error, show an error message.

## Inspect first
- apps/web/src/pages/JobDetail.tsx — current action buttons
- apps/web/src/components/job-detail/ — tab components
- apps/web/src/api/jobs.ts or similar — how job detail is fetched

## Likely files to modify
- apps/web/src/pages/JobDetail.tsx or relevant action bar component
- New file: apps/web/src/components/job-detail/CvTailoringModal.tsx
- apps/web/src/api/ (add cv tailoring client function)

## Rules
- Use existing modal/dialog pattern from the codebase.
- Lazy load the CV tailoring call (only when button clicked, not on page load).
- Match dark operator-focused style.

## Acceptance criteria
- [ ] "Tailor CV" button visible in Job Detail
- [ ] Click opens modal with loading state
- [ ] Modal shows skeleton while API call in progress
- [ ] Error message if API fails
- [ ] Modal closeable with Escape or X button

## Verification commands
pnpm --dir apps/web typecheck
pnpm guard:web-api-imports

## Final response format
## What changed
## Why
## Files changed
## Verification
## Notes / follow-ups
```

---

## C6 — Web: CV suggestions display

```
You are working in the Job Copilot monorepo.
First read: CLAUDE.md, apps/web/src/components/job-detail/CvTailoringModal.tsx (C5 output),
packages/contracts/src/ (check if cv-tailoring types should be added)

## Goal
Implement the suggestions display inside the CV Tailoring modal:
- Section "Skills to Highlight" — green chips for each skill
- Section "Skills to Mention" — blue chips
- Section "Skill Gaps" — orange chips with tooltip showing the suggestion text
- Section "Suggested Summary" — text block with monospace/pre formatting
- Section "Key Phrases" — small grey tags

Each section shows only if it has content. Empty sections are hidden.

## Inspect first
- apps/web/src/components/job-detail/CvTailoringModal.tsx — current modal content
- apps/web/src/components/ — existing chip/tag/badge components
- packages/contracts/src/ — where to add CvTailoringResponse type

## Likely files to modify
- apps/web/src/components/job-detail/CvTailoringModal.tsx (add suggestion sections)
- packages/contracts/src/cv-tailoring.ts (create — add response types)

## Rules
- Do not render raw API field names — use human-readable section labels.
- Skill chips should be small, not overwhelming.
- Suggested Summary section must be selectable text (easy to copy manually).

## Acceptance criteria
- [ ] All 5 sections render correctly when data present
- [ ] Sections with no content are hidden
- [ ] Skill chips have correct color per section
- [ ] Suggested summary is readable, selectable text
- [ ] TypeScript types match API response

## Verification commands
pnpm --dir apps/web typecheck
pnpm --dir apps/web lint

## Final response format
## What changed
## Why
## Files changed
## Verification
## Notes / follow-ups
```

---

## C7 — Web: copy sections to clipboard

```
You are working in the Job Copilot monorepo.
First read: CLAUDE.md, apps/web/src/components/job-detail/CvTailoringModal.tsx (C6 output)

## Goal
Add copy-to-clipboard buttons to each section of the CV tailoring suggestions:
- "Copy skills list" button next to Skills to Highlight → copies comma-separated skills
- "Copy summary" button next to Suggested Summary → copies the summary text
- "Copy all" button at the bottom → copies a formatted text block with all sections

Use the navigator.clipboard API. Show a brief "Copied!" toast or button state change
after copy.

## Inspect first
- apps/web/src/components/job-detail/CvTailoringModal.tsx — current section layout
- apps/web/src/context/ToastContext.tsx (from A17) — use toast for copy confirmation

## Likely files to modify
- apps/web/src/components/job-detail/CvTailoringModal.tsx

## Rules
- navigator.clipboard requires HTTPS or localhost — add graceful fallback for HTTP.
- "Copy all" output should be clean plain text, not HTML.
- Button state: normal → "Copied!" (2s) → normal.

## Acceptance criteria
- [ ] "Copy skills" copies comma-separated skill list
- [ ] "Copy summary" copies summary paragraph
- [ ] "Copy all" copies formatted text block
- [ ] "Copied!" feedback shown after each copy
- [ ] Typecheck passes

## Verification commands
pnpm --dir apps/web typecheck

## Final response format
## What changed
## Why
## Files changed
## Verification
## Notes / follow-ups
```

---

## C8 — CV Tailoring tests (ML + integration)

```
You are working in the Job Copilot monorepo.
First read: CLAUDE.md, apps/ml/tests/ (existing test patterns),
apps/ml/app/enrichment_routes.py,
apps/ml/app/services/cv_tailoring_service.py,
apps/ml/app/templates/template_cv_tailoring.py

## Goal
Write comprehensive pytest tests for the CV tailoring feature:
1. test_cv_tailoring_template.py — unit tests for template provider:
   - skills_to_highlight correct with overlap
   - gaps_to_address correct with missing skills
   - summary_rewrite is non-empty and contains job title
   - empty input returns safe defaults
2. test_cv_tailoring_service.py — unit tests for service:
   - test with template provider (no mocks)
   - test with mock provider
3. test_cv_tailoring_route.py — integration tests for HTTP endpoint:
   - 200 with valid request + internal token
   - 401 without token
   - 422 on invalid request body

## Inspect first
- apps/ml/tests/test_job_fit_explanation.py — reference test structure
- apps/ml/app/auth.py — how to bypass auth in tests
- apps/ml/tests/conftest.py — shared fixtures

## Likely files to modify
- apps/ml/tests/test_cv_tailoring_template.py (create)
- apps/ml/tests/test_cv_tailoring_service.py (create)
- apps/ml/tests/test_cv_tailoring_route.py (create)

## Rules
- Tests must not make external API calls.
- Use the template provider — no mocking needed for basic tests.
- Cover edge cases: empty skills list, missing optional fields.

## Acceptance criteria
- [ ] All 3 test files exist and pass
- [ ] At least 10 test cases total
- [ ] No external network calls in tests
- [ ] cd apps/ml && python -m pytest passes

## Verification commands
cd apps/ml && python -m pytest tests/test_cv_tailoring*.py -v
cd apps/ml && python -m pytest

## Final response format
## What changed
## Why
## Files changed
## Verification
## Notes / follow-ups
```
