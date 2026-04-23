from typing import Any

from pydantic import BaseModel, ConfigDict, Field, field_validator

from app.enrichment.shared_profile.contract import (
    LlmContextEvidenceEntry,
    LlmContextFeedbackSummary,
    sanitize_text,
)


class SearchProfileRoleCandidate(BaseModel):
    model_config = ConfigDict(extra="forbid")

    role: str
    confidence: float

    @field_validator("role", mode="before")
    @classmethod
    def normalize_role(cls, value: Any) -> str:
        return sanitize_text(value)


class SearchProfileContext(BaseModel):
    model_config = ConfigDict(extra="forbid")

    primary_role: str
    primary_role_confidence: float | None = None
    target_roles: list[str] = Field(default_factory=list)
    role_candidates: list[SearchProfileRoleCandidate] = Field(default_factory=list)
    seniority: str
    target_regions: list[str] = Field(default_factory=list)
    work_modes: list[str] = Field(default_factory=list)
    allowed_sources: list[str] = Field(default_factory=list)
    profile_skills: list[str] = Field(default_factory=list)
    profile_keywords: list[str] = Field(default_factory=list)
    search_terms: list[str] = Field(default_factory=list)
    exclude_terms: list[str] = Field(default_factory=list)

    @field_validator("primary_role", "seniority", mode="before")
    @classmethod
    def normalize_scalar(cls, value: Any) -> str:
        return sanitize_text(value)


class RankedJobContext(BaseModel):
    model_config = ConfigDict(extra="forbid")

    id: str
    title: str
    company_name: str
    description_text: str
    summary: str | None = None
    source: str | None = None
    source_job_id: str | None = None
    source_url: str | None = None
    remote_type: str | None = None
    seniority: str | None = None
    salary_label: str | None = None
    location_label: str | None = None
    work_mode_label: str | None = None
    freshness_label: str | None = None
    badges: list[str] = Field(default_factory=list)

    @field_validator(
        "id",
        "title",
        "company_name",
        "description_text",
        "summary",
        "source",
        "source_job_id",
        "source_url",
        "remote_type",
        "seniority",
        "salary_label",
        "location_label",
        "work_mode_label",
        "freshness_label",
        mode="before",
    )
    @classmethod
    def normalize_scalar(cls, value: Any) -> str:
        return sanitize_text(value)


class DeterministicFitContext(BaseModel):
    model_config = ConfigDict(extra="forbid")

    job_id: str
    score: int
    matched_roles: list[str] = Field(default_factory=list)
    matched_skills: list[str] = Field(default_factory=list)
    matched_keywords: list[str] = Field(default_factory=list)
    source_match: bool
    work_mode_match: bool | None = None
    region_match: bool | None = None
    reasons: list[str] = Field(default_factory=list)

    @field_validator("job_id", mode="before")
    @classmethod
    def normalize_job_id(cls, value: Any) -> str:
        return sanitize_text(value)


class CurrentJobFeedbackState(BaseModel):
    model_config = ConfigDict(extra="forbid")

    saved: bool = False
    hidden: bool = False
    bad_fit: bool = False
    company_status: str | None = None

    @field_validator("company_status", mode="before")
    @classmethod
    def normalize_company_status(cls, value: Any) -> str:
        return sanitize_text(value)


class FeedbackStateContext(BaseModel):
    model_config = ConfigDict(extra="forbid")

    summary: LlmContextFeedbackSummary
    top_positive_evidence: list[LlmContextEvidenceEntry] = Field(default_factory=list)
    top_negative_evidence: list[LlmContextEvidenceEntry] = Field(default_factory=list)
    current_job_feedback: CurrentJobFeedbackState | None = None
