import json

from app.enrichment.interview_prep.contract import (
    InterviewPrepPrompt,
    InterviewPrepRequest,
    interview_prep_schema,
)


def build_interview_prep_prompt(context: InterviewPrepRequest) -> InterviewPrepPrompt:
    prompt_context = context.model_dump(exclude={"profile_id"}, by_alias=True)
    output_schema = interview_prep_schema()

    return InterviewPrepPrompt(
        system_instructions=(
            "You generate additive interview preparation for a job search copilot. "
            "Use only the provided deterministic context and optional additive enrichments. "
            "Do not change or reinterpret ranking, score, canonical IDs, source IDs, or entities. "
            "Do not invent experience, achievements, projects, metrics, employers, timelines, responsibilities, or technologies that are not present in the provided context. "
            "Every item must stay grounded in the analyzed profile, search profile, ranked job payload, deterministic fit, optional job-fit explanation, optional application coaching, optional cover letter draft, optional feedback state, or raw profile text. "
            "stories_to_prepare may only describe categories of evidence or examples the candidate should prepare from existing context, not fabricated story details. "
            "questions_to_ask should be grounded in explicit job scope, matched evidence, or missing signals from the provided context. "
            "If evidence is missing, ambiguous, or weak, place it in risk_areas instead of filling it in. "
            "Return plain JSON only. Do not use markdown, bullets, headings, or code fences."
        ),
        context_payload=json.dumps(prompt_context, ensure_ascii=True, indent=2, sort_keys=True),
        output_schema_expectations=json.dumps(output_schema, ensure_ascii=True, indent=2, sort_keys=True),
        output_schema=output_schema,
    )
