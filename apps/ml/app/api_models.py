from pydantic import BaseModel, Field

from app.trained_reranker.artifact import TrainingSummary


class HealthResponse(BaseModel):
    status: str
    service: str
    engine_api_base_url: str


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
    reason: str | None = None
    model_path: str | None = None
    training: TrainingSummary | None = None
    feature_importances: dict[str, float] | None = None
