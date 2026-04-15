import json
from typing import Any

from fastapi import HTTPException, status
from pydantic import BaseModel, ConfigDict, Field, ValidationError, field_validator

from app.profile_insights import (
    MAX_LIST_ITEMS,
    LlmContextAnalyzedProfile,
    LlmContextEvidenceEntry,
    LlmContextFeedbackSummary,
    sanitize_text,
)


def normalize_string_list(value: Any) -> list[str]:
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


class WeeklyGuidanceJobsBySourceEntry(BaseModel):
    model_config = ConfigDict(extra="forbid")

    source: str
    count: int

    @field_validator("source", mode="before")
    @classmethod
    def normalize_source(cls, value: Any) -> str:
        return sanitize_text(value)


class WeeklyGuidanceJobsByLifecycle(BaseModel):
    model_config = ConfigDict(extra="forbid")

    total: int
    active: int
    inactive: int
    reactivated: int


class WeeklyGuidanceAnalyticsSummary(BaseModel):
    model_config = ConfigDict(extra="forbid")

    feedback: LlmContextFeedbackSummary
    jobs_by_source: list[WeeklyGuidanceJobsBySourceEntry] = Field(default_factory=list)
    jobs_by_lifecycle: WeeklyGuidanceJobsByLifecycle
    top_matched_roles: list[str] = Field(default_factory=list)
    top_matched_skills: list[str] = Field(default_factory=list)
    top_matched_keywords: list[str] = Field(default_factory=list)

    @field_validator("top_matched_roles", "top_matched_skills", "top_matched_keywords", mode="before")
    @classmethod
    def normalize_list_field(cls, value: Any) -> list[str]:
        return normalize_string_list(value)


class WeeklyGuidanceBehaviorSignalCount(BaseModel):
    model_config = ConfigDict(extra="forbid")

    key: str
    save_count: int
    hide_count: int
    bad_fit_count: int
    application_created_count: int
    positive_count: int
    negative_count: int
    net_score: int

    @field_validator("key", mode="before")
    @classmethod
    def normalize_key(cls, value: Any) -> str:
        return sanitize_text(value)


class WeeklyGuidanceBehaviorSummary(BaseModel):
    model_config = ConfigDict(extra="forbid")

    search_run_count: int
    top_positive_sources: list[WeeklyGuidanceBehaviorSignalCount] = Field(default_factory=list)
    top_negative_sources: list[WeeklyGuidanceBehaviorSignalCount] = Field(default_factory=list)
    top_positive_role_families: list[WeeklyGuidanceBehaviorSignalCount] = Field(default_factory=list)
    top_negative_role_families: list[WeeklyGuidanceBehaviorSignalCount] = Field(default_factory=list)
    source_signal_counts: list[WeeklyGuidanceBehaviorSignalCount] = Field(default_factory=list)
    role_family_signal_counts: list[WeeklyGuidanceBehaviorSignalCount] = Field(default_factory=list)


class WeeklyGuidanceFunnelSourceCount(BaseModel):
    model_config = ConfigDict(extra="forbid")

    source: str
    count: int

    @field_validator("source", mode="before")
    @classmethod
    def normalize_source(cls, value: Any) -> str:
        return sanitize_text(value)


class WeeklyGuidanceFunnelConversionRates(BaseModel):
    model_config = ConfigDict(extra="forbid")

    open_rate_from_impressions: float
    save_rate_from_opens: float
    application_rate_from_saves: float


class WeeklyGuidanceFunnelSummary(BaseModel):
    model_config = ConfigDict(extra="forbid")

    impression_count: int
    open_count: int
    save_count: int
    hide_count: int
    bad_fit_count: int
    application_created_count: int
    fit_explanation_requested_count: int
    application_coach_requested_count: int
    cover_letter_draft_requested_count: int
    interview_prep_requested_count: int
    conversion_rates: WeeklyGuidanceFunnelConversionRates
    impressions_by_source: list[WeeklyGuidanceFunnelSourceCount] = Field(default_factory=list)
    opens_by_source: list[WeeklyGuidanceFunnelSourceCount] = Field(default_factory=list)
    saves_by_source: list[WeeklyGuidanceFunnelSourceCount] = Field(default_factory=list)
    applications_by_source: list[WeeklyGuidanceFunnelSourceCount] = Field(default_factory=list)


class WeeklyGuidanceLlmContext(BaseModel):
    model_config = ConfigDict(extra="forbid")

    analyzed_profile: LlmContextAnalyzedProfile | None = None
    profile_skills: list[str] = Field(default_factory=list)
    profile_keywords: list[str] = Field(default_factory=list)
    jobs_feed_summary: WeeklyGuidanceJobsByLifecycle
    feedback_summary: LlmContextFeedbackSummary
    top_positive_evidence: list[LlmContextEvidenceEntry] = Field(default_factory=list)
    top_negative_evidence: list[LlmContextEvidenceEntry] = Field(default_factory=list)

    @field_validator("profile_skills", "profile_keywords", mode="before")
    @classmethod
    def normalize_list_field(cls, value: Any) -> list[str]:
        return normalize_string_list(value)


class WeeklyGuidanceRecentSearchSummary(BaseModel):
    model_config = ConfigDict(extra="forbid")

    target_roles: list[str] = Field(default_factory=list)
    search_terms: list[str] = Field(default_factory=list)
    exclude_terms: list[str] = Field(default_factory=list)
    allowed_sources: list[str] = Field(default_factory=list)
    target_regions: list[str] = Field(default_factory=list)
    work_modes: list[str] = Field(default_factory=list)

    @field_validator(
        "target_roles",
        "search_terms",
        "exclude_terms",
        "allowed_sources",
        "target_regions",
        "work_modes",
        mode="before",
    )
    @classmethod
    def normalize_list_field(cls, value: Any) -> list[str]:
        return normalize_string_list(value)


class WeeklyGuidanceRecentFeedbackSummary(BaseModel):
    model_config = ConfigDict(extra="forbid")

    summary: LlmContextFeedbackSummary
    top_positive_evidence: list[LlmContextEvidenceEntry] = Field(default_factory=list)
    top_negative_evidence: list[LlmContextEvidenceEntry] = Field(default_factory=list)


class WeeklyGuidanceRequest(BaseModel):
    model_config = ConfigDict(extra="forbid")

    profile_id: str
    analytics_summary: WeeklyGuidanceAnalyticsSummary
    behavior_summary: WeeklyGuidanceBehaviorSummary
    funnel_summary: WeeklyGuidanceFunnelSummary
    llm_context: WeeklyGuidanceLlmContext
    recent_search_summary: WeeklyGuidanceRecentSearchSummary | None = None
    recent_feedback_summary: WeeklyGuidanceRecentFeedbackSummary | None = None

    @field_validator("profile_id", mode="before")
    @classmethod
    def normalize_profile_id(cls, value: Any) -> str:
        return sanitize_text(value)


class WeeklyGuidanceResponse(BaseModel):
    model_config = ConfigDict(extra="forbid")

    weekly_summary: str = ""
    what_is_working: list[str] = Field(default_factory=list)
    what_is_not_working: list[str] = Field(default_factory=list)
    recommended_search_adjustments: list[str] = Field(default_factory=list)
    recommended_source_moves: list[str] = Field(default_factory=list)
    recommended_role_focus: list[str] = Field(default_factory=list)
    funnel_bottlenecks: list[str] = Field(default_factory=list)
    next_week_plan: list[str] = Field(default_factory=list)

    @field_validator("weekly_summary", mode="before")
    @classmethod
    def normalize_summary(cls, value: Any) -> str:
        return sanitize_text(value)

    @field_validator(
        "what_is_working",
        "what_is_not_working",
        "recommended_search_adjustments",
        "recommended_source_moves",
        "recommended_role_focus",
        "funnel_bottlenecks",
        "next_week_plan",
        mode="before",
    )
    @classmethod
    def normalize_list_field(cls, value: Any) -> list[str]:
        return normalize_string_list(value)


class WeeklyGuidancePrompt(BaseModel):
    system_instructions: str
    context_payload: str
    output_schema_expectations: str
    output_schema: dict[str, Any]


class WeeklyGuidanceProviderError(Exception):
    pass


class MalformedWeeklyGuidanceOutputError(WeeklyGuidanceProviderError):
    pass


def weekly_guidance_schema() -> dict[str, Any]:
    schema = WeeklyGuidanceResponse.model_json_schema()
    schema["additionalProperties"] = False
    return schema


def build_weekly_guidance_prompt(context: WeeklyGuidanceRequest) -> WeeklyGuidancePrompt:
    prompt_context = context.model_dump(exclude={"profile_id"}, by_alias=True, exclude_none=True)
    output_schema = weekly_guidance_schema()

    return WeeklyGuidancePrompt(
        system_instructions=(
            "You generate additive weekly job-search guidance for a job search copilot. "
            "Use only the structured deterministic analytics, behavior, funnel, and LLM context provided. "
            "Do not change or reinterpret ranking, feedback state, canonical IDs, source IDs, event history, or entities. "
            "Do not invent trends, causal explanations, or facts that are not directly supported by the provided summaries. "
            "If evidence is weak or mixed, say so conservatively and avoid strong recommendations. "
            "Keep each list item short, concrete, and grounded in the supplied metrics or evidence. "
            "Return strict JSON only. Do not use markdown, bullets, headings, or code fences."
        ),
        context_payload=json.dumps(prompt_context, ensure_ascii=True, indent=2, sort_keys=True),
        output_schema_expectations=json.dumps(output_schema, ensure_ascii=True, indent=2, sort_keys=True),
        output_schema=output_schema,
    )


def parse_weekly_guidance_output(raw_output: Any) -> WeeklyGuidanceResponse:
    payload = raw_output
    if isinstance(raw_output, str):
        try:
            payload = json.loads(raw_output)
        except json.JSONDecodeError as exc:
            raise MalformedWeeklyGuidanceOutputError("provider returned non-JSON output") from exc

    if not isinstance(payload, dict):
        raise MalformedWeeklyGuidanceOutputError("provider returned a non-object response")

    try:
        return WeeklyGuidanceResponse.model_validate(payload)
    except (TypeError, ValidationError, ValueError) as exc:
        raise MalformedWeeklyGuidanceOutputError("provider returned invalid weekly guidance") from exc


def http_error_from_weekly_guidance_error(error: WeeklyGuidanceProviderError) -> HTTPException:
    if isinstance(error, MalformedWeeklyGuidanceOutputError):
        return HTTPException(
            status_code=status.HTTP_502_BAD_GATEWAY,
            detail="Weekly guidance provider returned malformed output.",
        )

    return HTTPException(
        status_code=status.HTTP_503_SERVICE_UNAVAILABLE,
        detail=str(error) or "Weekly guidance provider failed.",
    )
