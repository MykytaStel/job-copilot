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
    min_examples: int | None = None
    reason: str | None = None
    model_path: str | None = None
    artifact_version: str | None = None
    model_type: str | None = None
    training: TrainingSummary | None = None
    evaluation: RerankerEvaluationSummary | None = None
    benchmark: dict[str, str | float | bool] | None = None
    feature_importances: dict[str, float] | None = None

    @classmethod
    def insufficient_examples(
        cls,
        *,
        example_count: int,
        min_examples: int,
    ) -> "BootstrapWorkflowResult":
        return cls(
            retrained=False,
            example_count=example_count,
            min_examples=min_examples,
            reason=f"need at least {min_examples} examples, got {example_count}",
        )

    @classmethod
    def trained_model(
        cls,
        *,
        example_count: int,
        model_path: str | Path,
        training: TrainingSummary,
        evaluation: RerankerEvaluationSummary | None = None,
        benchmark: dict[str, str | float | bool] | None = None,
        feature_importances: dict[str, float] | None = None,
    ) -> "BootstrapWorkflowResult":
        return cls(
            retrained=True,
            example_count=example_count,
            model_path=str(model_path),
            artifact_version=ARTIFACT_VERSION,
            model_type=MODEL_TYPE,
            training=training,
            evaluation=evaluation,
            benchmark=benchmark,
            feature_importances=feature_importances,
        )

    def to_response(self) -> BootstrapResponse:
        return BootstrapResponse(
            retrained=self.retrained,
            example_count=self.example_count,
            reason=self.reason,
            model_path=self.model_path,
            artifact_version=self.artifact_version,
            model_type=self.model_type,
            training=self.training,
            evaluation=self.evaluation,
            benchmark=self.benchmark,
            feature_importances=self.feature_importances,
        )

    def to_payload(self) -> dict[str, Any]:
        return self.model_dump(mode="json", exclude_none=True)
