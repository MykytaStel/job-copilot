import json
from typing import Any

from fastapi import HTTPException, status
from pydantic import BaseModel, ConfigDict, Field, ValidationError, field_validator

from app.application_coach import ApplicationCoachResponse
from app.job_fit_explanation import (
    DeterministicFitContext,
    FeedbackStateContext,
    JobFitExplanationResponse,
    RankedJobContext,
    SearchProfileContext,
)
from app.profile_insights import MAX_LIST_ITEMS, LlmContextAnalyzedProfile, sanitize_text

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


def build_cover_letter_draft_prompt(context: CoverLetterDraftRequest) -> CoverLetterDraftPrompt:
    prompt_context = context.model_dump(exclude={"profile_id"}, by_alias=True)
    output_schema = cover_letter_draft_schema()

    return CoverLetterDraftPrompt(
        system_instructions=(
            "You generate additive cover letter drafts for a job search copilot. "
            "Use only the provided deterministic context and optional additive enrichments. "
            "Do not change or reinterpret ranking, score, canonical IDs, source IDs, or entities. "
            "Do not invent employers, achievements, metrics, timelines, responsibilities, technologies, or experience that are not present in the provided context. "
            "Every claim in the draft must be supportable by the profile analysis, search profile, deterministic fit, ranked job payload, optional job-fit explanation, optional application coaching, optional feedback state, or raw profile text. "
            "If evidence is weak or missing, put it in evidence_gaps instead of filling it in. "
            "Keep the draft readable and concise. opening_paragraph and closing_paragraph must each be a single paragraph. body_paragraphs should contain one to three short paragraphs. "
            "key_claims_used must list the grounded claims actually used in the draft. tone_notes must describe how the draft is framed without adding new facts. "
            "Return plain JSON only. Do not use markdown, bullets, headings, salutations, signatures, or code fences."
        ),
        context_payload=json.dumps(prompt_context, ensure_ascii=True, indent=2, sort_keys=True),
        output_schema_expectations=json.dumps(output_schema, ensure_ascii=True, indent=2, sort_keys=True),
        output_schema=output_schema,
    )


def parse_cover_letter_draft_output(raw_output: Any) -> CoverLetterDraftResponse:
    payload = raw_output
    if isinstance(raw_output, str):
        try:
            payload = json.loads(raw_output)
        except json.JSONDecodeError as exc:
            raise MalformedCoverLetterDraftOutputError("provider returned non-JSON output") from exc

    if not isinstance(payload, dict):
        raise MalformedCoverLetterDraftOutputError("provider returned a non-object response")

    try:
        return CoverLetterDraftResponse.model_validate(payload)
    except (TypeError, ValidationError, ValueError) as exc:
        raise MalformedCoverLetterDraftOutputError(
            "provider returned invalid cover letter draft"
        ) from exc


def http_error_from_cover_letter_draft_error(
    error: CoverLetterDraftProviderError,
) -> HTTPException:
    if isinstance(error, MalformedCoverLetterDraftOutputError):
        return HTTPException(
            status_code=status.HTTP_502_BAD_GATEWAY,
            detail="Cover letter draft provider returned malformed output.",
        )

    return HTTPException(
        status_code=status.HTTP_503_SERVICE_UNAVAILABLE,
        detail=str(error) or "Cover letter draft provider failed.",
    )
