import json
import re
from typing import Any

from fastapi import HTTPException, status
from pydantic import BaseModel, ConfigDict, Field, ValidationError, field_validator


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


class LlmContextJobsFeedSummary(BaseModel):
    model_config = ConfigDict(extra="forbid")

    total: int
    active: int
    inactive: int
    reactivated: int


class LlmContextFeedbackSummary(BaseModel):
    model_config = ConfigDict(extra="forbid")

    saved_jobs_count: int
    hidden_jobs_count: int
    bad_fit_jobs_count: int
    whitelisted_companies_count: int
    blacklisted_companies_count: int


class LlmContextRequest(BaseModel):
    model_config = ConfigDict(extra="forbid")

    profile_id: str
    analyzed_profile: LlmContextAnalyzedProfile | None = None
    profile_skills: list[str] = Field(default_factory=list)
    profile_keywords: list[str] = Field(default_factory=list)
    jobs_feed_summary: LlmContextJobsFeedSummary
    feedback_summary: LlmContextFeedbackSummary
    top_positive_evidence: list[LlmContextEvidenceEntry] = Field(default_factory=list)
    top_negative_evidence: list[LlmContextEvidenceEntry] = Field(default_factory=list)


class ProfileInsightsResponse(BaseModel):
    model_config = ConfigDict(extra="forbid")

    profile_summary: str = ""
    search_strategy_summary: str = ""
    strengths: list[str] = Field(default_factory=list)
    risks: list[str] = Field(default_factory=list)
    recommended_actions: list[str] = Field(default_factory=list)
    top_focus_areas: list[str] = Field(default_factory=list)
    search_term_suggestions: list[str] = Field(default_factory=list)
    application_strategy: list[str] = Field(default_factory=list)

    @field_validator("profile_summary", "search_strategy_summary", mode="before")
    @classmethod
    def normalize_summary(cls, value: Any) -> str:
        return sanitize_text(value)

    @field_validator(
        "strengths",
        "risks",
        "recommended_actions",
        "top_focus_areas",
        "search_term_suggestions",
        "application_strategy",
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


class ProfileInsightsPrompt(BaseModel):
    system_instructions: str
    context_payload: str
    output_schema_expectations: str
    output_schema: dict[str, Any]


class ProfileInsightsProviderError(Exception):
    pass


class MalformedProviderOutputError(ProfileInsightsProviderError):
    pass


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


def profile_insights_schema() -> dict[str, Any]:
    schema = ProfileInsightsResponse.model_json_schema()
    schema["additionalProperties"] = False
    return schema


def build_profile_insights_prompt(context: LlmContextRequest) -> ProfileInsightsPrompt:
    prompt_context = context.model_dump(exclude={"profile_id"}, by_alias=True)
    output_schema = profile_insights_schema()

    return ProfileInsightsPrompt(
        system_instructions=(
            "You generate additive profile enrichment for a job search copilot. "
            "Use only the deterministic context provided. "
            "Do not change ranking, do not invent facts, and do not create canonical role IDs, source IDs, or new entities. "
            "Keep all output grounded in the provided profile analysis, feedback summary, and evidence. "
            "Return plain JSON only. Do not use markdown, bullets, headings, or code fences."
        ),
        context_payload=json.dumps(prompt_context, ensure_ascii=True, indent=2, sort_keys=True),
        output_schema_expectations=json.dumps(output_schema, ensure_ascii=True, indent=2, sort_keys=True),
        output_schema=output_schema,
    )


def parse_profile_insights_output(raw_output: Any) -> ProfileInsightsResponse:
    payload = raw_output
    if isinstance(raw_output, str):
        try:
            payload = json.loads(raw_output)
        except json.JSONDecodeError as exc:
            raise MalformedProviderOutputError("provider returned non-JSON output") from exc

    if not isinstance(payload, dict):
        raise MalformedProviderOutputError("provider returned a non-object response")

    try:
        return ProfileInsightsResponse.model_validate(payload)
    except (TypeError, ValidationError, ValueError) as exc:
        raise MalformedProviderOutputError("provider returned invalid profile insights") from exc


def http_error_from_provider_error(error: ProfileInsightsProviderError) -> HTTPException:
    if isinstance(error, MalformedProviderOutputError):
        detail = "Profile insights provider returned malformed output."
        status_code = status.HTTP_502_BAD_GATEWAY
    else:
        detail = str(error) or "Profile insights provider failed."
        status_code = status.HTTP_503_SERVICE_UNAVAILABLE

    return HTTPException(status_code=status_code, detail=detail)
