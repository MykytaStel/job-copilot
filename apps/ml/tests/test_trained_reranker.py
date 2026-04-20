import json

import pytest
from pydantic import BaseModel

from app.reranker_evaluation import evaluate_dataset
from app.trained_reranker import TrainedRerankerModel, load_dataset, train_model


def dataset_payload():
    return {
        "profile_id": "profile-1",
        "label_policy_version": "outcome_label_v2",
        "examples": [
            {
                "job_id": "positive-strong",
                "source": "djinni",
                "role_family": "engineering",
                "label": "positive",
                "label_score": 2,
                "label_reasons": ["applied"],
                "signals": {
                    "applied": True,
                    "viewed": True,
                    "saved": True,
                },
                "ranking": {
                    "deterministic_score": 78,
                    "behavior_score_delta": 4,
                    "behavior_score": 82,
                    "learned_reranker_score_delta": 3,
                    "learned_reranker_score": 85,
                    "matched_role_count": 1,
                    "matched_skill_count": 5,
                    "matched_keyword_count": 4,
                },
            },
            {
                "job_id": "positive-mid",
                "source": "djinni",
                "role_family": "engineering",
                "label": "positive",
                "label_score": 2,
                "label_reasons": ["applied"],
                "signals": {
                    "applied": True,
                    "viewed": True,
                },
                "ranking": {
                    "deterministic_score": 70,
                    "behavior_score_delta": 3,
                    "behavior_score": 73,
                    "learned_reranker_score_delta": 3,
                    "learned_reranker_score": 76,
                    "matched_role_count": 1,
                    "matched_skill_count": 4,
                    "matched_keyword_count": 3,
                },
            },
            {
                "job_id": "medium-saved",
                "source": "djinni",
                "role_family": "engineering",
                "label": "medium",
                "label_score": 1,
                "label_reasons": ["saved"],
                "signals": {
                    "saved": True,
                    "viewed": True,
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
            },
            {
                "job_id": "negative-weak",
                "source": "work_ua",
                "role_family": None,
                "label": "negative",
                "label_score": 0,
                "label_reasons": ["dismissed", "hidden"],
                "signals": {
                    "dismissed": True,
                    "hidden": True,
                    "viewed": True,
                },
                "ranking": {
                    "deterministic_score": 40,
                    "behavior_score_delta": -5,
                    "behavior_score": 35,
                    "learned_reranker_score_delta": -4,
                    "learned_reranker_score": 31,
                    "matched_role_count": 0,
                    "matched_skill_count": 0,
                    "matched_keyword_count": 1,
                },
            },
            {
                "job_id": "negative-high-baseline",
                "source": None,
                "role_family": None,
                "label": "negative",
                "label_score": 0,
                "label_reasons": ["dismissed", "bad_fit"],
                "signals": {
                    "dismissed": True,
                    "bad_fit": True,
                    "saved": True,
                    "viewed": True,
                },
                "ranking": {
                    "deterministic_score": 88,
                    "behavior_score_delta": -8,
                    "behavior_score": 80,
                    "learned_reranker_score_delta": -6,
                    "learned_reranker_score": 74,
                    "matched_role_count": 0,
                    "matched_skill_count": 0,
                    "matched_keyword_count": 0,
                },
            },
        ],
    }


def metrics_by_variant(summary):
    return {metrics.variant: metrics for metrics in summary.variants}


def test_training_pipeline_runs_and_artifact_round_trips(tmp_path):
    model = train_model([dataset_payload()], epochs=300, learning_rate=0.12)
    output = tmp_path / "models" / "trained-reranker.json"

    model.save(output)
    loaded = TrainedRerankerModel.load(output)

    assert loaded.artifact.artifact_version == "trained_reranker_v2"
    assert loaded.artifact.model_type == "logistic_regression"
    assert loaded.artifact.label_policy_version == "outcome_label_v2"
    assert loaded.artifact.signal_weight_policy_version == "outcome_signal_weight_v1"
    assert loaded.artifact.signal_weights["saved_only"] == 0.6
    assert loaded.artifact.signal_weights["viewed_only"] == 0.4
    assert loaded.artifact.training.example_count == 5
    assert loaded.artifact.training.saved_only_count == 1
    assert loaded.artifact.training.viewed_only_count == 0
    assert loaded.artifact.weights.keys() == set(loaded.artifact.feature_names)
    assert loaded.predict_probability(dataset_payload()["examples"][0]) > loaded.predict_probability(
        dataset_payload()["examples"][3]
    )


def test_training_requires_labeled_examples():
    with pytest.raises(ValueError, match="profile/job outcome signals"):
        train_model(
            [
                {
                    "profile_id": "profile-1",
                    "label_policy_version": "outcome_label_v2",
                    "examples": [],
                }
            ]
        )


def test_trained_model_accepts_equivalent_pydantic_example_instance():
    class EquivalentRanking(BaseModel):
        deterministic_score: int
        behavior_score_delta: int = 0
        behavior_score: int
        learned_reranker_score_delta: int = 0
        learned_reranker_score: int
        matched_role_count: int = 0
        matched_skill_count: int = 0
        matched_keyword_count: int = 0

    class EquivalentExample(BaseModel):
        job_id: str
        source: str | None = None
        role_family: str | None = None
        label: str
        label_score: int
        ranking: EquivalentRanking

    model = train_model([dataset_payload()], epochs=20)
    example = EquivalentExample.model_validate(dataset_payload()["examples"][0])

    assert model.predict_probability(example) > 0


def test_load_dataset_validates_export_shape(tmp_path):
    dataset_path = tmp_path / "dataset.json"
    dataset_path.write_text(
        json.dumps(dataset_payload()),
        encoding="utf-8",
    )

    dataset = load_dataset(dataset_path)

    assert dataset.profile_id == "profile-1"
    assert len(dataset.examples) == 5


def test_evaluation_compares_trained_variant():
    model = train_model([dataset_payload()], epochs=300, learning_rate=0.12)
    summary = evaluate_dataset(dataset_payload(), top_n=2, trained_model=model)
    metrics = metrics_by_variant(summary)

    assert set(metrics) == {
        "deterministic",
        "deterministic_behavior",
        "deterministic_behavior_learned",
        "trained_reranker_prediction",
    }
    assert metrics["trained_reranker_prediction"].top_k_positives == 2
    assert metrics["trained_reranker_prediction"].ordered_job_ids[:2] == [
        "positive-strong",
        "positive-mid",
    ]
