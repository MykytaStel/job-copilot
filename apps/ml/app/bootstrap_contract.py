from pathlib import Path
from typing import Any

from pydantic import BaseModel

from app.api_models import BootstrapResponse
from app.trained_reranker import TrainingSummary


class BootstrapWorkflowResult(BaseModel):
    retrained: bool
    example_count: int
    min_examples: int | None = None
    reason: str | None = None
    model_path: str | None = None
    training: TrainingSummary | None = None
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
        feature_importances: dict[str, float] | None = None,
    ) -> "BootstrapWorkflowResult":
        return cls(
            retrained=True,
            example_count=example_count,
            model_path=str(model_path),
            training=training,
            feature_importances=feature_importances,
        )

    def to_response(self) -> BootstrapResponse:
        return BootstrapResponse(
            retrained=self.retrained,
            example_count=self.example_count,
            reason=self.reason,
            model_path=self.model_path,
            training=self.training,
            feature_importances=self.feature_importances,
        )

    def to_payload(self) -> dict[str, Any]:
        return self.model_dump(mode="json", exclude_none=True)
