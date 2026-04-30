from __future__ import annotations

from typing import Any

from pydantic import BaseModel, ConfigDict, Field, field_validator

MAX_TEXT_LENGTH = 80_000
MAX_KEYWORD_LENGTH = 80
MAX_KEYWORDS = 20
MAX_GAP_SUMMARY_LENGTH = 500


def _normalize_text_field(value: Any) -> str:
    if value is None:
        return ""
    if not isinstance(value, str):
        raise TypeError("expected a string")
    return value.replace("\r\n", "\n").strip()[:MAX_TEXT_LENGTH]


class ResumeMatchRequest(BaseModel):
    model_config = ConfigDict(extra="forbid")

    resume_text: str = Field(min_length=1)
    jd_text: str = Field(min_length=1)

    @field_validator("resume_text", "jd_text", mode="before")
    @classmethod
    def normalize_text(cls, value: Any) -> str:
        return _normalize_text_field(value)


class ResumeMatchResponse(BaseModel):
    model_config = ConfigDict(extra="forbid")

    keyword_coverage_percent: float
    matched_keywords: list[str] = Field(default_factory=list)
    missing_keywords: list[str] = Field(default_factory=list)
    gap_summary: str

