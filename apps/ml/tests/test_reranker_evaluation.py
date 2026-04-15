from app.reranker_evaluation import OutcomeDataset, evaluate_dataset


def dataset_payload():
    return {
        "profile_id": "profile-1",
        "label_policy_version": "outcome_label_v1",
        "examples": [
            {
                "job_id": "job-positive-low",
                "label": "positive",
                "label_score": 2,
                "ranking": {
                    "deterministic_score": 60,
                    "behavior_score": 62,
                    "learned_reranker_score": 80,
                },
            },
            {
                "job_id": "job-medium-high",
                "label": "medium",
                "label_score": 1,
                "ranking": {
                    "deterministic_score": 90,
                    "behavior_score": 88,
                    "learned_reranker_score": 86,
                },
            },
            {
                "job_id": "job-negative-mid",
                "label": "negative",
                "label_score": 0,
                "ranking": {
                    "deterministic_score": 70,
                    "behavior_score": 50,
                    "learned_reranker_score": 40,
                },
            },
            {
                "job_id": "job-positive-behavior",
                "label": "positive",
                "label_score": 2,
                "ranking": {
                    "deterministic_score": 50,
                    "behavior_score": 92,
                    "learned_reranker_score": 94,
                },
            },
        ],
    }


def metrics_by_variant(summary):
    return {metrics.variant: metrics for metrics in summary.variants}


def test_evaluator_compares_ranking_variants_with_deterministic_tiebreaks():
    summary = evaluate_dataset(dataset_payload(), top_n=2)
    metrics = metrics_by_variant(summary)

    assert summary.example_count == 4
    assert summary.positive_count == 2
    assert metrics["deterministic"].ordered_job_ids == [
        "job-medium-high",
        "job-negative-mid",
        "job-positive-low",
        "job-positive-behavior",
    ]
    assert metrics["deterministic"].top_k_positives == 0
    assert metrics["deterministic"].average_label_score_top_n == 0.5
    assert metrics["deterministic"].positive_hit_rate == 0.0

    assert metrics["deterministic_behavior"].ordered_job_ids[:2] == [
        "job-positive-behavior",
        "job-medium-high",
    ]
    assert metrics["deterministic_behavior"].top_k_positives == 1
    assert metrics["deterministic_behavior"].average_label_score_top_n == 1.5
    assert metrics["deterministic_behavior"].positive_hit_rate == 0.5

    assert metrics["deterministic_behavior_learned"].ordered_job_ids[:2] == [
        "job-positive-behavior",
        "job-medium-high",
    ]


def test_evaluator_handles_empty_dataset_without_divide_by_zero():
    summary = evaluate_dataset(
        {
            "profile_id": "profile-1",
            "label_policy_version": "outcome_label_v1",
            "examples": [],
        },
        top_n=0,
    )

    assert summary.example_count == 0
    assert summary.positive_count == 0
    assert summary.top_n == 1
    for metrics in summary.variants:
        assert metrics.ordered_job_ids == []
        assert metrics.top_k_positives == 0
        assert metrics.average_label_score_top_n == 0.0
        assert metrics.positive_hit_rate == 0.0


def test_evaluator_accepts_validated_dataset_model():
    dataset = OutcomeDataset.model_validate(dataset_payload())
    summary = evaluate_dataset(dataset, top_n=3)

    assert summary.profile_id == "profile-1"
    assert summary.label_policy_version == "outcome_label_v1"
    assert summary.variants[0].top_n == 3
