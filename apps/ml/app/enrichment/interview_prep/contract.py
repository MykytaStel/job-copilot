from typing import Any

from pydantic import BaseModel, ConfigDict, Field, field_validator

from app.enrichment.application_coach.contract import ApplicationCoachResponse
from app.enrichment.cover_letter_draft.contract import CoverLetterDraftResponse
from app.enrichment.job_fit_explanation.contract import (
    DeterministicFitContext,
    FeedbackStateContext,
    JobFitExplanationResponse,
    RankedJobContext,
    SearchProfileContext,
)
from app.enrichment.shared_profile.contract import (
    MAX_LIST_ITEMS,
    LlmContextAnalyzedProfile,
    sanitize_text,
)

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
