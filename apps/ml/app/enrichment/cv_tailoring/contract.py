from __future__ import annotations

from typing import Any

from pydantic import BaseModel, ConfigDict, Field, field_validator

from app.enrichment.shared_profile.contract import LIST_PREFIX_RE, WHITESPACE_RE

MAX_SKILL_ITEMS = 10
MAX_GAP_ITEMS = 10
MAX_SKILL_TEXT_LENGTH = 120
MAX_SUGGESTION_TEXT_LENGTH = 300
MAX_SUMMARY_LENGTH = 600


def _sanitize(value: Any) -> str:
    if value is None:
        return ""
    if not isinstance(value, str):
        raise TypeError("expected a string")
    cleaned = value.replace("```json", "").replace("```", "").replace("`", "").strip()
    cleaned = LIST_PREFIX_RE.sub("", cleaned)
    cleaned = cleaned.replace("\n", " ")
    cleaned = WHITESPACE_RE.sub(" ", cleaned).strip()
    return cleaned


def _truncate(value: str, max_length: int) -> str:
    if len(value) <= max_length:
        return value
    candidate = value[: max_length - 1].rstrip()
    boundary = candidate.rfind(" ")
    if boundary >= max_length // 2:
        candidate = candidate[:boundary].rstrip()
    return candidate + "..."


def _normalize_str(value: Any, max_length: int) -> str:
    cleaned = _sanitize(value)
    if not cleaned:
        return ""
    return _truncate(cleaned, max_length)


def _normalize_str_list(value: Any, *, max_items: int, max_length: int) -> list[str]:
    if value is None:
        return []
    if not isinstance(value, list):
        raise TypeError("expected a list of strings")
    result: list[str] = []
    seen: set[str] = set()
    for item in value:
        cleaned = _normalize_str(item, max_length)
        if not cleaned:
            continue
        key = cleaned.casefold()
        if key in seen:
            continue
        seen.add(key)
        result.append(cleaned)
        if len(result) >= max_items:
            break
    return result


class CvTailoringGapItem(BaseModel):
    model_config = ConfigDict(extra="forbid")

    skill: str
    suggestion: str

    @field_validator("skill", mode="before")
    @classmethod
    def normalize_skill(cls, value: Any) -> str:
        return _normalize_str(value, MAX_SKILL_TEXT_LENGTH)

    @field_validator("suggestion", mode="before")
    @classmethod
    def normalize_suggestion(cls, value: Any) -> str:
        return _normalize_str(value, MAX_SUGGESTION_TEXT_LENGTH)


class CvTailoringSuggestions(BaseModel):
    model_config = ConfigDict(extra="forbid")

    skills_to_highlight: list[str] = Field(default_factory=list)
    skills_to_mention: list[str] = Field(default_factory=list)
    gaps_to_address: list[CvTailoringGapItem] = Field(default_factory=list)
    summary_rewrite: str = ""
    key_phrases: list[str] = Field(default_factory=list)

    @field_validator("skills_to_highlight", "skills_to_mention", "key_phrases", mode="before")
    @classmethod
    def normalize_str_list(cls, value: Any) -> list[str]:
        return _normalize_str_list(value, max_items=MAX_SKILL_ITEMS, max_length=MAX_SKILL_TEXT_LENGTH)

    @field_validator("gaps_to_address", mode="before")
    @classmethod
    def normalize_gaps(cls, value: Any) -> list[Any]:
        if value is None:
            return []
        if not isinstance(value, list):
            raise TypeError("expected a list for gaps_to_address")
        return value[:MAX_GAP_ITEMS]

    @field_validator("summary_rewrite", mode="before")
    @classmethod
    def normalize_summary(cls, value: Any) -> str:
        return _normalize_str(value, MAX_SUMMARY_LENGTH)


class CvTailoringRequest(BaseModel):
    model_config = ConfigDict(extra="forbid")

    profile_id: str
    job_id: str
    profile_summary: str = ""
    candidate_skills: list[str] = Field(default_factory=list)
    job_title: str = ""
    job_description: str = ""
    job_required_skills: list[str] = Field(default_factory=list)
    job_nice_to_have_skills: list[str] = Field(default_factory=list)
    candidate_cv_text: str | None = None


class CvTailoringResponse(BaseModel):
    model_config = ConfigDict(extra="forbid")

    suggestions: CvTailoringSuggestions
    provider: str
    generated_at: str


class CvTailoringPrompt(BaseModel):
    system_instructions: str
    context_payload: str
    output_schema_expectations: str
    output_schema: dict[str, Any]


class CvTailoringProviderError(Exception):
    pass


class MalformedCvTailoringOutputError(CvTailoringProviderError):
    pass


def cv_tailoring_suggestions_schema() -> dict[str, Any]:
    schema = CvTailoringSuggestions.model_json_schema()
    schema["additionalProperties"] = False
    return schema
