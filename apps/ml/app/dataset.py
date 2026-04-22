from typing import Literal

from pydantic import BaseModel, ConfigDict, Field


class OutcomeRankingFeatures(BaseModel):
    model_config = ConfigDict(extra="forbid")

    deterministic_score: int = Field(ge=0, le=100)
    behavior_score_delta: int = 0
    behavior_score: int = Field(ge=0, le=100)
    learned_reranker_score_delta: int = 0
    learned_reranker_score: int = Field(ge=0, le=100)
    matched_role_count: int = Field(default=0, ge=0)
    matched_skill_count: int = Field(default=0, ge=0)
    matched_keyword_count: int = Field(default=0, ge=0)
    matched_roles: list[str] = Field(default_factory=list)
    matched_skills: list[str] = Field(default_factory=list)
    matched_keywords: list[str] = Field(default_factory=list)
    fit_reasons: list[str] = Field(default_factory=list)
    behavior_reasons: list[str] = Field(default_factory=list)
    learned_reasons: list[str] = Field(default_factory=list)


class OutcomeSignals(BaseModel):
    model_config = ConfigDict(extra="forbid")

    viewed: bool = False
    saved: bool = False
    hidden: bool = False
    bad_fit: bool = False
    applied: bool = False
    dismissed: bool = False
    explicit_feedback: bool = False
    explicit_saved: bool = False
    explicit_hidden: bool = False
    explicit_bad_fit: bool = False
    viewed_event_count: int = Field(default=0, ge=0)
    saved_event_count: int = Field(default=0, ge=0)
    applied_event_count: int = Field(default=0, ge=0)
    dismissed_event_count: int = Field(default=0, ge=0)
    # Slice 1: application outcome
    outcome: str | None = None
    reached_interview: bool = False
    received_offer: bool = False
    was_rejected: bool = False
    was_ghosted: bool = False
    # Slice 2: structured rejection/interest tags
    rejection_tags: list[str] = Field(default_factory=list)
    positive_tags: list[str] = Field(default_factory=list)
    has_salary_rejection: bool = False
    has_remote_rejection: bool = False
    has_tech_rejection: bool = False
    # Slice 3: salary signal
    salary_signal: str | None = None
    salary_below_expectation: bool = False
    # Slice 4: interest rating
    interest_rating: int | None = None
    # Slice 5: work mode signal
    work_mode_deal_breaker: bool = False
    # Slice 6: engagement depth
    scrolled_to_bottom: bool = False
    returned_count: int = Field(default=0, ge=0)
    # Slice 7: legitimacy
    legitimacy_suspicious: bool = False
    legitimacy_spam: bool = False


class OutcomeExample(BaseModel):
    model_config = ConfigDict(extra="forbid")

    profile_id: str | None = None
    job_id: str = Field(min_length=1)
    title: str | None = None
    company_name: str | None = None
    source: str | None = None
    role_family: str | None = None
    label_observed_at: str | None = None
    label: Literal["positive", "medium", "negative"]
    label_score: int = Field(ge=0, le=2)
    label_reasons: list[str] = Field(default_factory=list)
    signals: OutcomeSignals | None = None
    ranking: OutcomeRankingFeatures


class OutcomeDataset(BaseModel):
    model_config = ConfigDict(extra="forbid")

    profile_id: str = Field(min_length=1)
    label_policy_version: str = Field(min_length=1)
    examples: list[OutcomeExample] = Field(default_factory=list)
