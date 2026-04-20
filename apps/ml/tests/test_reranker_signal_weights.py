from app.reranker_evaluation import OutcomeExample
from app.reranker_signal_weights import (
    DEFAULT_OUTCOME_SIGNAL_WEIGHTS,
    resolve_example_signal_bucket,
    training_target_for_example,
)


def example_payload(
    *,
    label: str,
    label_reasons: list[str],
    signals: dict[str, bool],
) -> OutcomeExample:
    return OutcomeExample.model_validate(
        {
            "job_id": "job-1",
            "label": label,
            "label_score": 2 if label == "positive" else 0 if label == "negative" else 1,
            "label_reasons": label_reasons,
            "signals": signals,
            "ranking": {
                "deterministic_score": 70,
                "behavior_score": 70,
                "learned_reranker_score": 70,
            },
        }
    )


def test_saved_only_example_uses_saved_only_weight():
    example = example_payload(
        label="medium",
        label_reasons=["saved"],
        signals={"saved": True, "viewed": True},
    )

    assert resolve_example_signal_bucket(example) == "saved_only"
    assert training_target_for_example(example) == DEFAULT_OUTCOME_SIGNAL_WEIGHTS.saved_only


def test_viewed_only_example_uses_viewed_only_weight():
    example = example_payload(
        label="medium",
        label_reasons=["viewed"],
        signals={"viewed": True},
    )

    assert resolve_example_signal_bucket(example) == "viewed_only"
    assert training_target_for_example(example) == DEFAULT_OUTCOME_SIGNAL_WEIGHTS.viewed_only


def test_conflicting_signals_still_follow_upstream_applied_and_dismissed_labels():
    applied = example_payload(
        label="positive",
        label_reasons=["applied"],
        signals={"applied": True, "saved": True, "viewed": True},
    )
    dismissed = example_payload(
        label="negative",
        label_reasons=["dismissed", "bad_fit"],
        signals={"dismissed": True, "bad_fit": True, "saved": True, "viewed": True},
    )

    assert resolve_example_signal_bucket(applied) == "applied"
    assert training_target_for_example(applied) == DEFAULT_OUTCOME_SIGNAL_WEIGHTS.applied
    assert resolve_example_signal_bucket(dismissed) == "dismissed"
    assert training_target_for_example(dismissed) == DEFAULT_OUTCOME_SIGNAL_WEIGHTS.dismissed
