from __future__ import annotations

import math

from app.dataset import OutcomeExample


def temporal_train_test_split(
    examples: list[OutcomeExample],
) -> tuple[list[OutcomeExample], list[OutcomeExample]]:
    if not examples:
        return [], []

    ordered = sort_examples_temporally(examples)
    if len(ordered) == 1:
        return [], ordered

    test_count = max(1, math.ceil(len(ordered) * 0.2))
    if test_count >= len(ordered):
        test_count = 1

    return ordered[:-test_count], ordered[-test_count:]


def rolling_temporal_windows(
    examples: list[OutcomeExample],
) -> list[tuple[list[OutcomeExample], list[OutcomeExample]]]:
    ordered = sort_examples_temporally(examples)
    if len(ordered) <= 1:
        return [(ordered[:0], ordered)]

    min_train = max(3, math.ceil(len(ordered) * 0.5))
    window_size = max(1, math.ceil(len(ordered) * 0.2))
    windows: list[tuple[list[OutcomeExample], list[OutcomeExample]]] = []
    train_end = min_train
    while train_end < len(ordered):
        test_end = min(len(ordered), train_end + window_size)
        windows.append((ordered[:train_end], ordered[train_end:test_end]))
        if test_end >= len(ordered):
            break
        train_end = test_end
    return windows or [temporal_train_test_split(ordered)]


def sort_examples_temporally(examples: list[OutcomeExample]) -> list[OutcomeExample]:
    return sorted(
        examples,
        key=lambda example: (
            example.label_observed_at or "",
            example.job_id,
        ),
    )
