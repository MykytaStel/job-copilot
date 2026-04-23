from typing import Any

from pydantic import BaseModel, ConfigDict, Field, field_validator

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
