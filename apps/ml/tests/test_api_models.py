from app.api_models import BootstrapResponse
from app.trained_reranker.artifact import TrainingSummary


def training_summary_payload() -> dict:
    return {
        "example_count": 30,
        "positive_count": 10,
        "medium_count": 10,
        "negative_count": 10,
        "saved_only_count": 4,
        "viewed_only_count": 3,
        "medium_default_count": 0,
        "epochs": 500,
        "learning_rate": 0.08,
        "l2": 0.01,
        "loss": 0.123,
    }


def test_bootstrap_response_keeps_training_json_shape_with_typed_summary():
    response = BootstrapResponse(
        retrained=True,
        example_count=30,
        model_path="/tmp/trained-reranker-v3.json",
        training=TrainingSummary.model_validate(training_summary_payload()),
    )

    assert response.model_dump(mode="json") == {
        "retrained": True,
        "example_count": 30,
        "reason": None,
        "model_path": "/tmp/trained-reranker-v3.json",
        "artifact_version": None,
        "model_type": None,
        "training": training_summary_payload(),
        "evaluation": None,
        "benchmark": None,
        "feature_importances": None,
    }
