import asyncio
import json
import sys
from contextlib import asynccontextmanager

import httpx
import pytest

from app import bootstrap_client, bootstrap_training, engine_api_client as engine_api_client_module
from app.api_models import BootstrapRequest
from app.bootstrap_contract import BootstrapWorkflowResult
from app.engine_api_client import EngineApiClient, EngineApiResponseError, EngineApiUnavailableError
from app.reranker_bootstrap_service import (
    RerankerBootstrapService,
    RerankerBootstrapUpstreamHttpError,
    RerankerBootstrapUpstreamUnavailableError,
)
from app.reranker_evaluation import OutcomeDataset
from app.trained_reranker import TrainingSummary
from app.trained_reranker_config import DEFAULT_TRAINED_RERANKER_MODEL_PATH


def dataset_payload(example_count: int = 1) -> dict:
    return {
        "profile_id": "profile-1",
        "label_policy_version": "outcome_label_v3",
        "examples": [
            {
                "job_id": f"job-{index}",
                "source": "djinni",
                "role_family": "engineering",
                "label": "medium",
                "label_score": 1,
                "label_reasons": ["saved"],
                "signals": {
                    "viewed": True,
                    "saved": True,
                    "hidden": False,
                    "bad_fit": False,
                    "applied": False,
                    "dismissed": False,
                    "explicit_feedback": True,
                    "explicit_saved": True,
                    "explicit_hidden": False,
                    "explicit_bad_fit": False,
                    "viewed_event_count": 1,
                    "saved_event_count": 1,
                    "applied_event_count": 0,
                    "dismissed_event_count": 0,
                },
                "ranking": {
                    "deterministic_score": 64,
                    "behavior_score_delta": 1,
                    "behavior_score": 65,
                    "learned_reranker_score_delta": 0,
                    "learned_reranker_score": 65,
                    "matched_role_count": 1,
                    "matched_skill_count": 2,
                    "matched_keyword_count": 1,
                },
            }
            for index in range(example_count)
        ],
    }


def training_summary_payload(loss: float = 0.1) -> dict:
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
        "loss": loss,
    }


def test_engine_api_client_fetch_reranker_dataset_uses_dataset_endpoint_and_validates_payload():
    captured: dict[str, object] = {}

    class FakeResponse:
        status_code = 200
        content = b'{"profile_id":"profile-1"}'

        def json(self) -> dict:
            return dataset_payload()

    class FakeAsyncClient:
        async def get(self, url: str) -> FakeResponse:
            captured["url"] = url
            return FakeResponse()

    dataset = asyncio.run(
        EngineApiClient(
            FakeAsyncClient(),
            base_url="http://engine.test/",
        )
        .fetch_reranker_dataset("profile-1")
    )

    assert captured["url"] == "http://engine.test/api/v1/profiles/profile-1/reranker-dataset"
    assert dataset.profile_id == "profile-1"
    assert len(dataset.examples) == 1


def test_engine_api_client_fetch_reranker_dataset_maps_upstream_error_payload_like_other_reads():
    class FakeResponse:
        status_code = 404
        content = b'{"code":"profile_not_found","message":"profile not found"}'

        def json(self) -> dict:
            return {"code": "profile_not_found", "message": "profile not found"}

    class FakeAsyncClient:
        async def get(self, url: str) -> FakeResponse:
            assert url == "http://engine.test/api/v1/profiles/profile-1/reranker-dataset"
            return FakeResponse()

    with pytest.raises(EngineApiResponseError) as exc_info:
        asyncio.run(
            EngineApiClient(
                FakeAsyncClient(),
                base_url="http://engine.test/",
            ).fetch_reranker_dataset("profile-1")
        )

    assert exc_info.value.status_code == 404
    assert exc_info.value.detail == "profile not found"


def test_fetch_labeled_examples_uses_engine_api_client_and_timeout(monkeypatch):
    captured: dict[str, object] = {}
    expected_dataset = OutcomeDataset.model_validate(dataset_payload())

    class FakeEngineApiClient:
        async def fetch_reranker_dataset(self, profile_id: str) -> OutcomeDataset:
            captured["profile_id"] = profile_id
            return expected_dataset

    @asynccontextmanager
    async def fake_engine_api_client_context(*, base_url=None):
        captured["base_url"] = base_url
        yield FakeEngineApiClient()

    monkeypatch.setattr(
        bootstrap_client,
        "engine_api_client_context",
        fake_engine_api_client_context,
    )

    dataset = asyncio.run(
        bootstrap_client.fetch_labeled_examples(
            "profile-1",
            base_url="http://engine.test/",
        )
    )

    assert captured["base_url"] == "http://engine.test/"
    assert captured["profile_id"] == "profile-1"
    assert dataset == expected_dataset


def test_engine_api_client_context_uses_timeout_and_base_url(monkeypatch):
    captured: dict[str, object] = {}

    class FakeAsyncClient:
        def __init__(self, *, timeout):
            captured["timeout"] = timeout

        async def __aenter__(self):
            return self

        async def __aexit__(self, exc_type, exc, tb):
            return None

    class FakeEngineApiClient:
        def __init__(self, client, *, base_url=None):
            captured["client"] = client
            captured["base_url"] = base_url

    monkeypatch.setattr(engine_api_client_module, "engine_api_timeout_seconds", lambda: 12.5)
    monkeypatch.setattr(engine_api_client_module.httpx, "AsyncClient", FakeAsyncClient)
    monkeypatch.setattr(engine_api_client_module, "EngineApiClient", FakeEngineApiClient)

    async def open_client() -> None:
        async with engine_api_client_module.engine_api_client_context(
            base_url="http://engine.test/"
        ) as engine_api:
            captured["engine_api"] = engine_api

    asyncio.run(open_client())

    assert isinstance(captured["timeout"], httpx.Timeout)
    assert captured["timeout"].connect == 12.5
    assert captured["base_url"] == "http://engine.test/"
    assert captured["engine_api"] is not None


def test_reranker_bootstrap_service_maps_engine_api_response_error_to_stable_contract(
):
    async def fake_bootstrap_and_retrain(profile_id: str, min_examples: int, model_path):
        raise EngineApiResponseError(status_code=404, detail="profile not found")

    service = RerankerBootstrapService(
        bootstrap_workflow=fake_bootstrap_and_retrain,
        model_path=bootstrap_training.DEFAULT_MODEL_PATH,
    )
    payload = BootstrapRequest(profile_id="profile-1", min_examples=30)

    with pytest.raises(RerankerBootstrapUpstreamHttpError) as exc_info:
        asyncio.run(service.bootstrap(payload=payload))

    assert exc_info.value.status_code == 404
    assert str(exc_info.value) == "engine-api error: 404"


def test_reranker_bootstrap_service_maps_engine_api_unavailable_error_to_stable_contract(
):
    async def fake_bootstrap_and_retrain(profile_id: str, min_examples: int, model_path):
        raise EngineApiUnavailableError("engine-api request failed: timed out")

    service = RerankerBootstrapService(
        bootstrap_workflow=fake_bootstrap_and_retrain,
        model_path=bootstrap_training.DEFAULT_MODEL_PATH,
    )
    payload = BootstrapRequest(profile_id="profile-1", min_examples=30)

    with pytest.raises(RerankerBootstrapUpstreamUnavailableError) as exc_info:
        asyncio.run(service.bootstrap(payload=payload))

    assert exc_info.value.detail == "engine-api unreachable: engine-api request failed: timed out"
    assert str(exc_info.value) == "engine-api unreachable: engine-api request failed: timed out"


def test_reranker_bootstrap_service_uses_injected_workflow_and_model_path():
    captured: dict[str, object] = {}
    training = TrainingSummary.model_validate(training_summary_payload())

    async def fake_bootstrap_and_retrain(profile_id: str, min_examples: int, model_path):
        captured["profile_id"] = profile_id
        captured["min_examples"] = min_examples
        captured["model_path"] = model_path
        return BootstrapWorkflowResult.trained_model(
            example_count=30,
            model_path=model_path,
            training=training,
        )

    service = RerankerBootstrapService(
        bootstrap_workflow=fake_bootstrap_and_retrain,
        model_path=bootstrap_training.DEFAULT_MODEL_PATH,
    )

    response = asyncio.run(
        service.bootstrap(BootstrapRequest(profile_id="profile-1", min_examples=30))
    )

    assert captured == {
        "profile_id": "profile-1",
        "min_examples": 30,
        "model_path": bootstrap_training.DEFAULT_MODEL_PATH,
    }
    assert response.model_dump() == {
        "retrained": True,
        "example_count": 30,
        "reason": None,
        "model_path": str(bootstrap_training.DEFAULT_MODEL_PATH),
        "training": training_summary_payload(),
        "feature_importances": None,
    }


def test_reranker_bootstrap_service_keeps_public_response_shape_when_workflow_skips_retrain():
    async def fake_bootstrap_and_retrain(profile_id: str, min_examples: int, model_path):
        return BootstrapWorkflowResult.insufficient_examples(
            example_count=2,
            min_examples=3,
        )

    service = RerankerBootstrapService(
        bootstrap_workflow=fake_bootstrap_and_retrain,
        model_path=bootstrap_training.DEFAULT_MODEL_PATH,
    )

    response = asyncio.run(
        service.bootstrap(BootstrapRequest(profile_id="profile-1", min_examples=3))
    )

    assert response.model_dump() == {
        "retrained": False,
        "example_count": 2,
        "reason": "need at least 3 examples, got 2",
        "model_path": None,
        "training": None,
        "feature_importances": None,
    }


def test_bootstrap_and_retrain_uses_bootstrap_training_fetch_compat_surface(monkeypatch):
    async def fake_fetch_labeled_examples(profile_id: str, base_url: str | None = None):
        assert profile_id == "profile-1"
        assert base_url == "http://engine.test"
        return OutcomeDataset.model_validate(dataset_payload(example_count=2))

    monkeypatch.setattr(
        bootstrap_training,
        "fetch_labeled_examples",
        fake_fetch_labeled_examples,
    )

    result = asyncio.run(
        bootstrap_training.bootstrap_and_retrain(
            "profile-1",
            min_examples=3,
            base_url="http://engine.test",
        )
    )

    assert result.to_payload() == {
        "retrained": False,
        "example_count": 2,
        "min_examples": 3,
        "reason": "need at least 3 examples, got 2",
    }


def test_main_uses_default_model_path_and_prints_json(monkeypatch, capsys):
    captured: dict[str, object] = {}

    async def fake_bootstrap_and_retrain(
        profile_id: str,
        min_examples: int = 30,
        model_path=bootstrap_training.DEFAULT_MODEL_PATH,
        base_url: str | None = None,
    ) -> BootstrapWorkflowResult:
        captured["profile_id"] = profile_id
        captured["min_examples"] = min_examples
        captured["model_path"] = model_path
        captured["base_url"] = base_url
        return BootstrapWorkflowResult.trained_model(
            example_count=30,
            model_path=model_path,
            training=TrainingSummary.model_validate(training_summary_payload(loss=0.123)),
        )

    monkeypatch.setattr(
        bootstrap_training,
        "bootstrap_and_retrain",
        fake_bootstrap_and_retrain,
    )
    monkeypatch.setattr(
        sys,
        "argv",
        ["bootstrap_training.py", "--profile-id", "profile-1", "--min-examples", "30"],
    )

    bootstrap_training.main()
    output = json.loads(capsys.readouterr().out)

    assert captured == {
        "profile_id": "profile-1",
        "min_examples": 30,
        "model_path": DEFAULT_TRAINED_RERANKER_MODEL_PATH,
        "base_url": None,
    }
    assert output == {
        "retrained": True,
        "example_count": 30,
        "model_path": str(DEFAULT_TRAINED_RERANKER_MODEL_PATH),
        "training": training_summary_payload(loss=0.123),
    }


def test_main_exits_with_status_one_when_bootstrap_does_not_retrain(monkeypatch, capsys):
    async def fake_bootstrap_and_retrain(
        profile_id: str,
        min_examples: int = 30,
        model_path=bootstrap_training.DEFAULT_MODEL_PATH,
        base_url: str | None = None,
    ) -> BootstrapWorkflowResult:
        return BootstrapWorkflowResult.insufficient_examples(
            example_count=12,
            min_examples=min_examples,
        )

    monkeypatch.setattr(
        bootstrap_training,
        "bootstrap_and_retrain",
        fake_bootstrap_and_retrain,
    )
    monkeypatch.setattr(
        sys,
        "argv",
        ["bootstrap_training.py", "--profile-id", "profile-1"],
    )

    with pytest.raises(SystemExit, match="1"):
        bootstrap_training.main()

    output = json.loads(capsys.readouterr().out)
    assert output["retrained"] is False
    assert output["reason"] == "need at least 30 examples, got 12"


def test_bootstrap_training_keeps_default_model_path_compat_export():
    assert bootstrap_training.DEFAULT_MODEL_PATH == DEFAULT_TRAINED_RERANKER_MODEL_PATH
