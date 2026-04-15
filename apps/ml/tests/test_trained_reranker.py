import json

from app.reranker_evaluation import evaluate_dataset
from app.trained_reranker import TrainedRerankerModel, load_dataset, train_model


def dataset_payload():
    return {
        "profile_id": "profile-1",
        "label_policy_version": "outcome_label_v1",
        "examples": [
            {
                "job_id": "positive-strong",
                "source": "djinni",
                "role_family": "engineering",
                "label": "positive",
                "label_score": 2,
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
    output = tmp_path / "trained-reranker.json"

    model.save(output)
    loaded = TrainedRerankerModel.load(output)

    assert loaded.artifact.artifact_version == "trained_reranker_v2"
    assert loaded.artifact.model_type == "logistic_regression"
    assert loaded.artifact.training.example_count == 5
    assert loaded.artifact.weights.keys() == set(loaded.artifact.feature_names)
    assert loaded.predict_probability(dataset_payload()["examples"][0]) > loaded.predict_probability(
        dataset_payload()["examples"][3]
    )


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
