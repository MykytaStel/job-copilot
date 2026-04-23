import re
from typing import Any

from pydantic import BaseModel, ConfigDict, Field, field_validator


MAX_LIST_ITEMS = 6
WHITESPACE_RE = re.compile(r"\s+")
LIST_PREFIX_RE = re.compile(r"^\s*(?:[-*•]+|\d+\.)\s*")


class LlmContextAnalyzedProfile(BaseModel):
    summary: str
    primary_role: str
    seniority: str
    skills: list[str] = Field(default_factory=list)
    keywords: list[str] = Field(default_factory=list)


class LlmContextEvidenceEntry(BaseModel):
    model_config = ConfigDict(extra="forbid")

    entry_type: str = Field(alias="type")
    label: str

    @field_validator("entry_type", "label", mode="before")
    @classmethod
    def normalize_scalar(cls, value: Any) -> str:
        return sanitize_text(value)


class LlmContextFeedbackSummary(BaseModel):
    model_config = ConfigDict(extra="forbid")

    saved_jobs_count: int
    hidden_jobs_count: int
    bad_fit_jobs_count: int
    whitelisted_companies_count: int
    blacklisted_companies_count: int


def sanitize_text(value: Any) -> str:
    if value is None:
        return ""
    if not isinstance(value, str):
        raise TypeError("expected a string")

    cleaned = value.replace("```json", "").replace("```", "").replace("`", "").strip()
    cleaned = LIST_PREFIX_RE.sub("", cleaned)
    cleaned = cleaned.replace("\n", " ")
    cleaned = WHITESPACE_RE.sub(" ", cleaned).strip()
    return cleaned
