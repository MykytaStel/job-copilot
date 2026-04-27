from __future__ import annotations

import logging

from app.dataset import OutcomeDataset, OutcomeExample

logger = logging.getLogger(__name__)

MIN_BOOTSTRAP_LABELED_EXAMPLES = 10


def normalize_bootstrap_dataset(dataset: OutcomeDataset) -> OutcomeDataset:
    """
    Normalize only explicit user-event signals.

    Important:
    Existing engine dataset examples are already labeled. Do not drop or rewrite
    neutral/medium examples just because they do not carry explicit save/hide/apply
    flags, otherwise bootstrap workflow validation loses class-mix and policy checks.
    """
    examples = [_normalize_example(example) for example in dataset.examples]

    return dataset.model_copy(update={"examples": examples})


def has_enough_bootstrap_examples(
    dataset: OutcomeDataset,
    min_examples: int = MIN_BOOTSTRAP_LABELED_EXAMPLES,
) -> bool:
    if len(dataset.examples) < min_examples:
        logger.warning(
            "not enough labeled bootstrap examples: got %d, need %d",
            len(dataset.examples),
            min_examples,
        )
        return False

    return True


def _normalize_example(example: OutcomeExample) -> OutcomeExample:
    signals = example.signals

    if signals is None:
        return example

    # Explicit positive outcomes from user_events/applications.
    if signals.applied or signals.explicit_saved:
        return _with_label(example, "positive")

    # Explicit negative outcomes from user feedback.
    if signals.explicit_hidden or signals.explicit_bad_fit or signals.bad_fit or signals.hidden:
        return _with_label(example, "negative")

    # Preserve existing labels for already-materialized outcome dataset rows.
    return example


def _with_label(example: OutcomeExample, label: str) -> OutcomeExample:
    label_score = {"negative": 0, "medium": 1, "positive": 2}[label]
    return example.model_copy(update={"label": label, "label_score": label_score})
