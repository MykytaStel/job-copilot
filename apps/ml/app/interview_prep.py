import json
from typing import Any

from fastapi import HTTPException, status
from pydantic import BaseModel, ConfigDict, Field, ValidationError, field_validator

from app.application_coach import ApplicationCoachResponse
from app.cover_letter_draft import CoverLetterDraftResponse
from app.job_fit_explanation import (
    DeterministicFitContext,
    FeedbackStateContext,
    JobFitExplanationResponse,
    RankedJobContext,
    SearchProfileContext,
)
from app.profile_insights import MAX_LIST_ITEMS, LlmContextAnalyzedProfile, sanitize_text

MAX_SUMMARY_LENGTH = 320
MAX_LIST_TEXT_LENGTH = 200


def _truncate_text(value: str, max_length: int) -> str:
    if len(value) <= max_length:
        return value

    candidate = value[: max_length - 1].rstrip()
    boundary = candidate.rfind(" ")
    if boundary >= max_length // 2:
        candidate = candidate[:boundary].rstrip()
    return f"{candidate}…"


def _normalize_bounded_text(value: Any, max_length: int) -> str:
    cleaned = sanitize_text(value)
    if not cleaned:
        return ""
    return _truncate_text(cleaned, max_length)


def _normalize_bounded_list(value: Any, *, max_items: int, max_length: int) -> list[str]:
    if value is None:
        return []
    if not isinstance(value, list):
        raise TypeError("expected a list of strings")

    normalized: list[str] = []
    seen: set[str] = set()
    for item in value:
        if not isinstance(item, str):
            raise TypeError("expected a list of strings")
        cleaned = _normalize_bounded_text(item, max_length)
        if not cleaned:
            continue
        key = cleaned.casefold()
        if key in seen:
            continue
        seen.add(key)
        normalized.append(cleaned)
        if len(normalized) >= max_items:
            break
    return normalized


class InterviewPrepRequest(BaseModel):
    model_config = ConfigDict(extra="forbid")

    profile_id: str
    analyzed_profile: LlmContextAnalyzedProfile | None = None
    search_profile: SearchProfileContext | None = None
    ranked_job: RankedJobContext
    deterministic_fit: DeterministicFitContext
    job_fit_explanation: JobFitExplanationResponse | None = None
    application_coach: ApplicationCoachResponse | None = None
    cover_letter_draft: CoverLetterDraftResponse | None = None
    feedback_state: FeedbackStateContext | None = None
    raw_profile_text: str | None = None

    @field_validator("profile_id", "raw_profile_text", mode="before")
    @classmethod
    def normalize_scalar(cls, value: Any) -> str:
        return sanitize_text(value)


class InterviewPrepResponse(BaseModel):
    model_config = ConfigDict(extra="forbid")

    prep_summary: str = ""
    likely_topics: list[str] = Field(default_factory=list)
    technical_focus: list[str] = Field(default_factory=list)
    behavioral_focus: list[str] = Field(default_factory=list)
    stories_to_prepare: list[str] = Field(default_factory=list)
    questions_to_ask: list[str] = Field(default_factory=list)
    risk_areas: list[str] = Field(default_factory=list)
    follow_up_plan: list[str] = Field(default_factory=list)

    @field_validator("prep_summary", mode="before")
    @classmethod
    def normalize_summary(cls, value: Any) -> str:
        return _normalize_bounded_text(value, MAX_SUMMARY_LENGTH)

    @field_validator(
        "likely_topics",
        "technical_focus",
        "behavioral_focus",
        "stories_to_prepare",
        "questions_to_ask",
        "risk_areas",
        "follow_up_plan",
        mode="before",
    )
    @classmethod
    def normalize_list_field(cls, value: Any) -> list[str]:
        return _normalize_bounded_list(
            value,
            max_items=MAX_LIST_ITEMS,
            max_length=MAX_LIST_TEXT_LENGTH,
        )


class InterviewPrepPrompt(BaseModel):
    system_instructions: str
    context_payload: str
    output_schema_expectations: str
    output_schema: dict[str, Any]


class InterviewPrepProviderError(Exception):
    pass


class MalformedInterviewPrepOutputError(InterviewPrepProviderError):
    pass


def interview_prep_schema() -> dict[str, Any]:
    schema = InterviewPrepResponse.model_json_schema()
    schema["additionalProperties"] = False
    return schema


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


def parse_interview_prep_output(raw_output: Any) -> InterviewPrepResponse:
    payload = raw_output
    if isinstance(raw_output, str):
        try:
            payload = json.loads(raw_output)
        except json.JSONDecodeError as exc:
            raise MalformedInterviewPrepOutputError("provider returned non-JSON output") from exc

    if not isinstance(payload, dict):
        raise MalformedInterviewPrepOutputError("provider returned a non-object response")

    try:
        return InterviewPrepResponse.model_validate(payload)
    except (TypeError, ValidationError, ValueError) as exc:
        raise MalformedInterviewPrepOutputError(
            "provider returned invalid interview preparation"
        ) from exc


def http_error_from_interview_prep_error(error: InterviewPrepProviderError) -> HTTPException:
    if isinstance(error, MalformedInterviewPrepOutputError):
        return HTTPException(
            status_code=status.HTTP_502_BAD_GATEWAY,
            detail="Interview prep provider returned malformed output.",
        )

    return HTTPException(
        status_code=status.HTTP_503_SERVICE_UNAVAILABLE,
        detail=str(error) or "Interview prep provider failed.",
    )
