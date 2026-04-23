import json

from app.enrichment.application_coach.contract import (
    ApplicationCoachPrompt,
    ApplicationCoachRequest,
    application_coach_schema,
)


def build_application_coach_prompt(context: ApplicationCoachRequest) -> ApplicationCoachPrompt:
    prompt_context = context.model_dump(exclude={"profile_id"}, by_alias=True)
    output_schema = application_coach_schema()

    return ApplicationCoachPrompt(
        system_instructions=(
            "You generate additive application coaching for a job search copilot. "
            "Use only the structured deterministic context provided. "
            "Do not change or reinterpret ranking, score, canonical IDs, source IDs, or entities. "
            "Do not invent experience, fabricate achievements, or create work history that is not in the provided context. "
            "Only reframe existing profile evidence so the candidate can tailor their resume, cover letter, and interview preparation for the given job. "
            "If evidence is missing or weak, place it in gaps_to_address or red_flags instead of filling it in. "
            "suggested_bullets must stay grounded in provided profile evidence and may only describe experience already supported by the context. "
            "Return plain JSON only. Do not use markdown, bullets, headings, or code fences."
        ),
        context_payload=json.dumps(prompt_context, ensure_ascii=True, indent=2, sort_keys=True),
        output_schema_expectations=json.dumps(output_schema, ensure_ascii=True, indent=2, sort_keys=True),
        output_schema=output_schema,
    )
