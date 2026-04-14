import json
from typing import Any

from fastapi import HTTPException, status
from pydantic import BaseModel, ConfigDict, Field, ValidationError, field_validator

from app.job_fit_explanation import (
    DeterministicFitContext,
    FeedbackStateContext,
    JobFitExplanationResponse,
    RankedJobContext,
    SearchProfileContext,
)
from app.profile_insights import MAX_LIST_ITEMS, LlmContextAnalyzedProfile, sanitize_text


class ApplicationCoachRequest(BaseModel):
    model_config = ConfigDict(extra="forbid")

    profile_id: str
    analyzed_profile: LlmContextAnalyzedProfile | None = None
    search_profile: SearchProfileContext | None = None
    ranked_job: RankedJobContext
    deterministic_fit: DeterministicFitContext
    job_fit_explanation: JobFitExplanationResponse | None = None
    feedback_state: FeedbackStateContext | None = None
    raw_profile_text: str | None = None

    @field_validator("profile_id", "raw_profile_text", mode="before")
    @classmethod
    def normalize_scalar(cls, value: Any) -> str:
        return sanitize_text(value)


class ApplicationCoachResponse(BaseModel):
    model_config = ConfigDict(extra="forbid")

    application_summary: str = ""
    resume_focus_points: list[str] = Field(default_factory=list)
    suggested_bullets: list[str] = Field(default_factory=list)
    cover_letter_angles: list[str] = Field(default_factory=list)
    interview_focus: list[str] = Field(default_factory=list)
    gaps_to_address: list[str] = Field(default_factory=list)
    red_flags: list[str] = Field(default_factory=list)

    @field_validator("application_summary", mode="before")
    @classmethod
    def normalize_summary(cls, value: Any) -> str:
        return sanitize_text(value)

    @field_validator(
        "resume_focus_points",
        "suggested_bullets",
        "cover_letter_angles",
        "interview_focus",
        "gaps_to_address",
        "red_flags",
        mode="before",
    )
    @classmethod
    def normalize_list_field(cls, value: Any) -> list[str]:
        if value is None:
            return []
        if not isinstance(value, list):
            raise TypeError("expected a list of strings")

        normalized: list[str] = []
        seen: set[str] = set()
        for item in value:
            if not isinstance(item, str):
                raise TypeError("expected a list of strings")
            cleaned = sanitize_text(item)
            if not cleaned:
                continue
            key = cleaned.casefold()
            if key in seen:
                continue
            seen.add(key)
            normalized.append(cleaned)
            if len(normalized) >= MAX_LIST_ITEMS:
                break
        return normalized


class ApplicationCoachPrompt(BaseModel):
    system_instructions: str
    context_payload: str
    output_schema_expectations: str
    output_schema: dict[str, Any]


class ApplicationCoachProviderError(Exception):
    pass


class MalformedApplicationCoachOutputError(ApplicationCoachProviderError):
    pass


def application_coach_schema() -> dict[str, Any]:
    schema = ApplicationCoachResponse.model_json_schema()
    schema["additionalProperties"] = False
    return schema


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


def parse_application_coach_output(raw_output: Any) -> ApplicationCoachResponse:
    payload = raw_output
    if isinstance(raw_output, str):
        try:
            payload = json.loads(raw_output)
        except json.JSONDecodeError as exc:
            raise MalformedApplicationCoachOutputError("provider returned non-JSON output") from exc

    if not isinstance(payload, dict):
        raise MalformedApplicationCoachOutputError("provider returned a non-object response")

    try:
        return ApplicationCoachResponse.model_validate(payload)
    except (TypeError, ValidationError, ValueError) as exc:
        raise MalformedApplicationCoachOutputError(
            "provider returned invalid application coaching"
        ) from exc


def http_error_from_application_coach_error(
    error: ApplicationCoachProviderError,
) -> HTTPException:
    if isinstance(error, MalformedApplicationCoachOutputError):
        return HTTPException(
            status_code=status.HTTP_502_BAD_GATEWAY,
            detail="Application coach provider returned malformed output.",
        )

    return HTTPException(
        status_code=status.HTTP_503_SERVICE_UNAVAILABLE,
        detail=str(error) or "Application coach provider failed.",
    )
