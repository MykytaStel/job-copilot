from typing import Any

from pydantic import BaseModel, ConfigDict, Field, field_validator

from app.enrichment.application_coach.contract import ApplicationCoachResponse
from app.enrichment.job_fit_explanation.contract import (
    JobFitExplanationResponse,
)
from app.enrichment.shared_job_fit.contract import (
    DeterministicFitContext,
    FeedbackStateContext,
    RankedJobContext,
    SearchProfileContext,
)
from app.enrichment.shared_profile.contract import (
    MAX_LIST_ITEMS,
    LlmContextAnalyzedProfile,
    sanitize_text,
)

MAX_BODY_PARAGRAPHS = 3
MAX_SUMMARY_LENGTH = 220
MAX_PARAGRAPH_LENGTH = 420
MAX_LIST_TEXT_LENGTH = 180


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


def _normalize_bounded_list(
    value: Any,
    *,
    max_items: int,
    max_length: int,
) -> list[str]:
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


class CoverLetterDraftRequest(BaseModel):
    model_config = ConfigDict(extra="forbid")

    profile_id: str
    analyzed_profile: LlmContextAnalyzedProfile | None = None
    search_profile: SearchProfileContext | None = None
    ranked_job: RankedJobContext
    deterministic_fit: DeterministicFitContext
    job_fit_explanation: JobFitExplanationResponse | None = None
    application_coach: ApplicationCoachResponse | None = None
    feedback_state: FeedbackStateContext | None = None
    raw_profile_text: str | None = None

    @field_validator("profile_id", "raw_profile_text", mode="before")
    @classmethod
    def normalize_scalar(cls, value: Any) -> str:
        return sanitize_text(value)


class CoverLetterDraftResponse(BaseModel):
    model_config = ConfigDict(extra="forbid")

    draft_summary: str = ""
    opening_paragraph: str = ""
    body_paragraphs: list[str] = Field(default_factory=list)
    closing_paragraph: str = ""
    key_claims_used: list[str] = Field(default_factory=list)
    evidence_gaps: list[str] = Field(default_factory=list)
    tone_notes: list[str] = Field(default_factory=list)

    @field_validator("draft_summary", mode="before")
    @classmethod
    def normalize_summary(cls, value: Any) -> str:
        return _normalize_bounded_text(value, MAX_SUMMARY_LENGTH)

    @field_validator("opening_paragraph", "closing_paragraph", mode="before")
    @classmethod
    def normalize_paragraph(cls, value: Any) -> str:
        return _normalize_bounded_text(value, MAX_PARAGRAPH_LENGTH)

    @field_validator("body_paragraphs", mode="before")
    @classmethod
    def normalize_body_paragraphs(cls, value: Any) -> list[str]:
        return _normalize_bounded_list(
            value,
            max_items=MAX_BODY_PARAGRAPHS,
            max_length=MAX_PARAGRAPH_LENGTH,
        )

    @field_validator("key_claims_used", "evidence_gaps", "tone_notes", mode="before")
    @classmethod
    def normalize_list_field(cls, value: Any) -> list[str]:
        return _normalize_bounded_list(
            value,
            max_items=MAX_LIST_ITEMS,
            max_length=MAX_LIST_TEXT_LENGTH,
        )


class CoverLetterDraftPrompt(BaseModel):
    system_instructions: str
    context_payload: str
    output_schema_expectations: str
    output_schema: dict[str, Any]


class CoverLetterDraftProviderError(Exception):
    pass


class MalformedCoverLetterDraftOutputError(CoverLetterDraftProviderError):
    pass


def cover_letter_draft_schema() -> dict[str, Any]:
    schema = CoverLetterDraftResponse.model_json_schema()
    schema["additionalProperties"] = False
    return schema
