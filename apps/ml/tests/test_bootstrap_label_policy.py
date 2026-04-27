from app.bootstrap_label_policy import (
    has_enough_bootstrap_examples,
    normalize_bootstrap_dataset,
)
from app.dataset import (
    OutcomeDataset,
    OutcomeExample,
    OutcomeRankingFeatures,
    OutcomeSignals,
)


def ranking() -> OutcomeRankingFeatures:
    return OutcomeRankingFeatures(
        deterministic_score=50,
        behavior_score=50,
        learned_reranker_score=50,
    )


def example(signals: OutcomeSignals, label: str = "medium") -> OutcomeExample:
    return OutcomeExample(
        job_id="job-1",
        label=label,
        label_score={"negative": 0, "medium": 1, "positive": 2}[label],
        signals=signals,
        ranking=ranking(),
    )


def dataset(*examples: OutcomeExample) -> OutcomeDataset:
    return OutcomeDataset(
        profile_id="profile-1",
        label_policy_version="test",
        examples=list(examples),
    )


def test_bootstrap_labels_positive_and_negative_signals() -> None:
    normalized = normalize_bootstrap_dataset(
        dataset(
            example(OutcomeSignals(explicit_saved=True, saved=True)),
            example(OutcomeSignals(applied=True)),
            example(OutcomeSignals(explicit_hidden=True, hidden=True)),
            example(OutcomeSignals(explicit_bad_fit=True, bad_fit=True)),
        )
    )

    assert [item.label for item in normalized.examples] == [
        "positive",
        "positive",
        "negative",
        "negative",
    ]

    assert [item.label_score for item in normalized.examples] == [2, 2, 0, 0]


def test_bootstrap_skips_training_under_ten_labeled_examples() -> None:
    normalized = normalize_bootstrap_dataset(
        dataset(*[example(OutcomeSignals(explicit_saved=True, saved=True)) for _ in range(9)])
    )

    assert not has_enough_bootstrap_examples(normalized, min_examples=10)


def test_bootstrap_preserves_existing_labeled_view_without_engagement() -> None:
    normalized = normalize_bootstrap_dataset(
        dataset(example(OutcomeSignals(viewed=True, viewed_event_count=1), label="medium"))
    )

    assert len(normalized.examples) == 1
    assert normalized.examples[0].label == "medium"
    assert normalized.examples[0].label_score == 1


def test_bootstrap_keeps_deeper_view_engagement() -> None:
    normalized = normalize_bootstrap_dataset(
        dataset(example(OutcomeSignals(viewed=True, viewed_event_count=2, returned_count=1)))
    )

    assert len(normalized.examples) == 1
