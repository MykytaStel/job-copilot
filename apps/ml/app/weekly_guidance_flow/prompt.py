import json
from typing import Any

from pydantic import BaseModel

from app.weekly_guidance_flow.contract import WeeklyGuidanceRequest, WeeklyGuidanceResponse


class WeeklyGuidancePrompt(BaseModel):
    system_instructions: str
    context_payload: str
    output_schema_expectations: str
    output_schema: dict[str, Any]


def weekly_guidance_schema() -> dict[str, Any]:
    schema = WeeklyGuidanceResponse.model_json_schema()
    schema["additionalProperties"] = False
    return schema


def build_weekly_guidance_prompt(context: WeeklyGuidanceRequest) -> WeeklyGuidancePrompt:
    prompt_context = context.model_dump(exclude={"profile_id"}, by_alias=True, exclude_none=True)
    output_schema = weekly_guidance_schema()

    return WeeklyGuidancePrompt(
        system_instructions=(
            "You generate additive weekly job-search guidance for a job search copilot. "
            "Use only the structured deterministic analytics, behavior, funnel, and LLM context provided. "
            "Do not change or reinterpret ranking, feedback state, canonical IDs, source IDs, event history, or entities. "
            "Do not invent trends, causal explanations, or facts that are not directly supported by the provided summaries. "
            "If evidence is weak or mixed, say so conservatively and avoid strong recommendations. "
            "Keep each list item short, concrete, and grounded in the supplied metrics or evidence. "
            "Return strict JSON only. Do not use markdown, bullets, headings, or code fences."
        ),
        context_payload=json.dumps(prompt_context, ensure_ascii=True, indent=2, sort_keys=True),
        output_schema_expectations=json.dumps(output_schema, ensure_ascii=True, indent=2, sort_keys=True),
        output_schema=output_schema,
    )
