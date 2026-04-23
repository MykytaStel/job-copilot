from app.reranker_evaluation import OutcomeDataset, evaluate_dataset


def dataset_payload():
    return {
        "profile_id": "profile-1",
        "label_policy_version": "outcome_label_v3",
        "examples": [
            {
                "job_id": "job-positive-low",
                "label_observed_at": "2026-04-10T00:00:00Z",
                "label": "positive",
                "label_score": 2,
                "label_reasons": ["applied"],
                "signals": {
                    "applied": True,
                    "viewed": True,
                    "saved": True,
                    "hidden": False,
                    "bad_fit": False,
                    "dismissed": False,
                    "explicit_feedback": True,
                    "explicit_saved": True,
                    "explicit_hidden": False,
                    "explicit_bad_fit": False,
                    "viewed_event_count": 1,
                    "saved_event_count": 1,
                    "applied_event_count": 1,
                    "dismissed_event_count": 0,
                },
                "ranking": {
                    "deterministic_score": 60,
                    "behavior_score": 62,
                    "learned_reranker_score": 80,
                },
            },
            {
                "job_id": "job-medium-high",
                "label_observed_at": "2026-04-11T00:00:00Z",
                "label": "medium",
                "label_score": 1,
                "label_reasons": ["viewed"],
                "signals": {
                    "viewed": True,
                    "saved": False,
                    "hidden": False,
                    "bad_fit": False,
                    "applied": False,
                    "dismissed": False,
                    "explicit_feedback": False,
                    "explicit_saved": False,
                    "explicit_hidden": False,
                    "explicit_bad_fit": False,
                    "viewed_event_count": 1,
                    "saved_event_count": 0,
                    "applied_event_count": 0,
                    "dismissed_event_count": 0,
                },
                "ranking": {
                    "deterministic_score": 90,
                    "behavior_score": 88,
                    "learned_reranker_score": 86,
                },
            },
            {
                "job_id": "job-negative-mid",
                "label_observed_at": "2026-04-12T00:00:00Z",
                "label": "negative",
                "label_score": 0,
                "label_reasons": ["dismissed", "hidden"],
                "signals": {
                    "dismissed": True,
                    "hidden": True,
                    "viewed": True,
                    "saved": True,
                    "bad_fit": False,
                    "applied": False,
                    "explicit_feedback": True,
                    "explicit_saved": False,
                    "explicit_hidden": True,
                    "explicit_bad_fit": False,
                    "viewed_event_count": 1,
                    "saved_event_count": 1,
                    "applied_event_count": 0,
                    "dismissed_event_count": 1,
                },
                "ranking": {
                    "deterministic_score": 70,
                    "behavior_score": 50,
                    "learned_reranker_score": 40,
                },
            },
            {
                "job_id": "job-positive-behavior",
                "label_observed_at": "2026-04-13T00:00:00Z",
                "label": "positive",
                "label_score": 2,
                "label_reasons": ["applied"],
                "signals": {
                    "applied": True,
                    "viewed": True,
                    "saved": False,
                    "hidden": False,
                    "bad_fit": False,
                    "dismissed": False,
                    "explicit_feedback": False,
                    "explicit_saved": False,
                    "explicit_hidden": False,
                    "explicit_bad_fit": False,
                    "viewed_event_count": 1,
                    "saved_event_count": 0,
                    "applied_event_count": 1,
                    "dismissed_event_count": 0,
                },
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
    assert summary.train_example_count == 3
    assert summary.test_example_count == 1
    assert summary.split_method == "temporal"
    assert summary.positive_count == 1
    assert summary.signal_weight_policy_version == "outcome_signal_weight_v2"
    assert metrics["deterministic"].ordered_job_ids == ["job-positive-behavior"]
    assert metrics["deterministic"].top_k_positives == 1
    assert metrics["deterministic"].average_label_score_top_n == 2.0
    assert metrics["deterministic"].average_training_weight_top_n == 1.0
    assert metrics["deterministic"].positive_hit_rate == 1.0

    assert metrics["deterministic_behavior"].ordered_job_ids == ["job-positive-behavior"]
    assert metrics["deterministic_behavior"].top_k_positives == 1
    assert metrics["deterministic_behavior"].average_label_score_top_n == 2.0
    assert metrics["deterministic_behavior"].average_training_weight_top_n == 1.0
    assert metrics["deterministic_behavior"].positive_hit_rate == 1.0

    assert metrics["deterministic_behavior_learned"].ordered_job_ids == ["job-positive-behavior"]


def test_evaluator_handles_empty_dataset_without_divide_by_zero():
    summary = evaluate_dataset(
        {
            "profile_id": "profile-1",
            "label_policy_version": "outcome_label_v3",
            "examples": [],
        },
        top_n=0,
    )

    assert summary.example_count == 0
    assert summary.train_example_count == 0
    assert summary.test_example_count == 0
    assert summary.positive_count == 0
    assert summary.top_n == 1
    assert summary.signal_weight_policy_version == "outcome_signal_weight_v2"
    for metrics in summary.variants:
        assert metrics.ordered_job_ids == []
        assert metrics.top_k_positives == 0
        assert metrics.average_label_score_top_n == 0.0
        assert metrics.average_training_weight_top_n == 0.0
        assert metrics.positive_hit_rate == 0.0


def test_evaluator_accepts_validated_dataset_model():
    dataset = OutcomeDataset.model_validate(dataset_payload())
    summary = evaluate_dataset(dataset, top_n=3)

    assert summary.profile_id == "profile-1"
    assert summary.label_policy_version == "outcome_label_v3"
    assert summary.variants[0].top_n == 3
