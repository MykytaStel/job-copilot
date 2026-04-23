import json

from app.enrichment.profile_insights.contract import (
    LlmContextRequest,
    ProfileInsightsPrompt,
    profile_insights_schema,
)


def build_profile_insights_prompt(context: LlmContextRequest) -> ProfileInsightsPrompt:
    prompt_context = context.model_dump(exclude={"profile_id"}, by_alias=True)
    output_schema = profile_insights_schema()

    return ProfileInsightsPrompt(
        system_instructions=(
            "You generate additive profile enrichment for a job search copilot. "
            "Use only the deterministic context provided. "
            "Do not change ranking, do not invent facts, and do not create canonical role IDs, source IDs, or new entities. "
            "Keep all output grounded in the provided profile analysis, feedback summary, and evidence. "
            "Return plain JSON only. Do not use markdown, bullets, headings, or code fences."
        ),
        context_payload=json.dumps(prompt_context, ensure_ascii=True, indent=2, sort_keys=True),
        output_schema_expectations=json.dumps(
            output_schema, ensure_ascii=True, indent=2, sort_keys=True
        ),
        output_schema=output_schema,
    )
