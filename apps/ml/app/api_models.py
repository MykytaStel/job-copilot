from pydantic import BaseModel, Field

from app.metrics import RerankerEvaluationSummary
from app.trained_reranker.artifact import TrainingSummary


class HealthResponse(BaseModel):
    status: str
    service: str
    engine_api_base_url: str
    llm_provider: str = "template"


class ReadyCheck(BaseModel):
    name: str
    status: str
    detail: str | None = None


class ReadyResponse(BaseModel):
    status: str
    service: str
    checks: list[ReadyCheck]


class FitAnalyzeRequest(BaseModel):
    profile_id: str = Field(min_length=1)
    job_id: str = Field(min_length=1)


class FitAnalyzeResponse(BaseModel):
    profile_id: str
    job_id: str
    score: int
    matched_terms: list[str]
    missing_terms: list[str]
    evidence: list[str]


class RerankRequest(BaseModel):
    profile_id: str = Field(min_length=1)
    job_ids: list[str] = Field(min_length=1, max_length=50)
    cache_bust: bool = False


class RerankInvalidateRequest(BaseModel):
    profile_id: str = Field(min_length=1)


class RerankedJob(BaseModel):
    job_id: str
    title: str
    company_name: str
    score: int
    matched_terms: list[str]
    evidence: list[str]


class RerankResponse(BaseModel):
    profile_id: str
    jobs: list[RerankedJob]


class BootstrapRequest(BaseModel):
    profile_id: str = Field(min_length=1)
    min_examples: int = Field(default=30, ge=1)


class BootstrapResponse(BaseModel):
    retrained: bool
    example_count: int
    profile_id: str | None = None
    reason: str | None = None
    model_path: str | None = None
    artifact_path: str | None = None
    artifact_version: str | None = None
    model_type: str | None = None
    training: TrainingSummary | None = None
    evaluation: RerankerEvaluationSummary | None = None
    benchmark: dict[str, str | float | bool] | None = None
    feature_importances: dict[str, float] | None = None
    distribution_shift_score: float | None = None
    started_at: str | None = None
    finished_at: str | None = None
    promotion_decision: str | None = None
    metrics_version: str | None = None
    lgbm_distilled: bool = False


class BootstrapTaskAccepted(BaseModel):
    task_id: str
    status: str = "accepted"


class BootstrapTaskStatus(BaseModel):
    task_id: str
    profile_id: str | None = None
    status: str  # "accepted" | "running" | "completed" | "failed"
    result: BootstrapResponse | None = None
    error: str | None = None
    artifact_path: str | None = None
    started_at: str | None = None
    finished_at: str | None = None
    promotion_decision: str | None = None
    metrics_version: str | None = None
