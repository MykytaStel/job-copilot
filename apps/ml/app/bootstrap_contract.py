from pathlib import Path
from typing import Any

from pydantic import BaseModel

from app.api_models import BootstrapResponse
from app.metrics import RerankerEvaluationSummary
from app.trained_reranker import ARTIFACT_VERSION, MODEL_TYPE
from app.trained_reranker import TrainingSummary


class BootstrapWorkflowResult(BaseModel):
    retrained: bool
    example_count: int
    profile_id: str | None = None
    min_examples: int | None = None
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

    @classmethod
    def insufficient_examples(
        cls,
        *,
        example_count: int,
        min_examples: int,
        profile_id: str | None = None,
        artifact_path: str | Path | None = None,
        model_path: str | Path | None = None,
        started_at: str | None = None,
        finished_at: str | None = None,
        promotion_decision: str | None = None,
        metrics_version: str | None = None,
        reason: str | None = None,
    ) -> "BootstrapWorkflowResult":
        return cls(
            retrained=False,
            example_count=example_count,
            profile_id=profile_id,
            min_examples=min_examples,
            reason=reason or f"need at least {min_examples} examples, got {example_count}",
            model_path=str(model_path) if model_path is not None else None,
            artifact_path=str(artifact_path) if artifact_path is not None else None,
            started_at=started_at,
            finished_at=finished_at,
            promotion_decision=promotion_decision,
            metrics_version=metrics_version,
        )

    @classmethod
    def trained_model(
        cls,
        *,
        example_count: int,
        profile_id: str,
        artifact_path: str | Path,
        model_path: str | Path,
        training: TrainingSummary,
        reason: str | None = None,
        evaluation: RerankerEvaluationSummary | None = None,
        benchmark: dict[str, str | float | bool] | None = None,
        feature_importances: dict[str, float] | None = None,
        distribution_shift_score: float | None = None,
        started_at: str | None = None,
        finished_at: str | None = None,
        promotion_decision: str | None = None,
        metrics_version: str | None = None,
        lgbm_distilled: bool = False,
    ) -> "BootstrapWorkflowResult":
        return cls(
            retrained=True,
            example_count=example_count,
            profile_id=profile_id,
            reason=reason,
            model_path=str(model_path),
            artifact_path=str(artifact_path),
            artifact_version=ARTIFACT_VERSION,
            model_type=MODEL_TYPE,
            training=training,
            evaluation=evaluation,
            benchmark=benchmark,
            feature_importances=feature_importances,
            distribution_shift_score=distribution_shift_score,
            started_at=started_at,
            finished_at=finished_at,
            promotion_decision=promotion_decision,
            metrics_version=metrics_version,
            lgbm_distilled=lgbm_distilled,
        )

    def to_response(self) -> BootstrapResponse:
        return BootstrapResponse(
            retrained=self.retrained,
            example_count=self.example_count,
            profile_id=self.profile_id,
            reason=self.reason,
            model_path=self.model_path,
            artifact_path=self.artifact_path,
            artifact_version=self.artifact_version,
            model_type=self.model_type,
            training=self.training,
            evaluation=self.evaluation,
            benchmark=self.benchmark,
            feature_importances=self.feature_importances,
            distribution_shift_score=self.distribution_shift_score,
            started_at=self.started_at,
            finished_at=self.finished_at,
            promotion_decision=self.promotion_decision,
            metrics_version=self.metrics_version,
            lgbm_distilled=self.lgbm_distilled,
        )

    def to_payload(self) -> dict[str, Any]:
        return self.model_dump(mode="json", exclude_none=True)
