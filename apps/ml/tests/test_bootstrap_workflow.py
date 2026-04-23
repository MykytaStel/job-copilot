import asyncio
from pathlib import Path

import pytest

from app.bootstrap_contract import BootstrapWorkflowResult
from app.bootstrap_workflow import bootstrap_and_retrain
from app.dataset import OutcomeDataset, OutcomeExample, OutcomeRankingFeatures, OutcomeSignals

# LightGBM requires a native dylib (libomp) that may not be present in all environments.
# Tests that actually train skip gracefully when the native library is missing.
try:
    import lightgbm  # noqa: F401

    _LGBM_AVAILABLE = True
except OSError:
    _LGBM_AVAILABLE = False

requires_lgbm = pytest.mark.skipif(not _LGBM_AVAILABLE, reason="libomp / LightGBM native library not available")


def make_example(job_id: str, label: str, date: str = "2026-04-15T00:00:00Z") -> OutcomeExample:
    return OutcomeExample(
        profile_id="profile-1",
        job_id=job_id,
        source="djinni",
        role_family="engineering",
        label=label,
        label_score={"positive": 2, "medium": 1, "negative": 0}[label],
        label_reasons=[label],
        label_observed_at=date,
        signals=OutcomeSignals(
            viewed=True,
            saved=label == "positive",
            hidden=label == "negative",
            bad_fit=label == "negative",
            explicit_feedback=label != "medium",
            explicit_saved=label == "positive",
            explicit_hidden=label == "negative",
        ),
        ranking=OutcomeRankingFeatures(
            deterministic_score=60,
            behavior_score=60,
            learned_reranker_score=60,
        ),
    )


def make_dataset(positive: int, medium: int, negative: int) -> OutcomeDataset:
    examples = (
        [make_example(f"p-{i}", "positive", f"2026-0{(i % 3) + 1}-{(i % 28) + 1:02d}T00:00:00Z") for i in range(positive)]
        + [make_example(f"m-{i}", "medium", f"2026-0{(i % 3) + 1}-{(i % 28) + 1:02d}T00:00:00Z") for i in range(medium)]
        + [make_example(f"n-{i}", "negative", f"2026-0{(i % 3) + 1}-{(i % 28) + 1:02d}T00:00:00Z") for i in range(negative)]
    )
    return OutcomeDataset(
        profile_id="profile-1",
        label_policy_version="outcome_label_v3",
        examples=examples,
    )


async def fetch_dataset(dataset: OutcomeDataset):
    async def _fetch(profile_id: str, base_url: str | None) -> OutcomeDataset:
        return dataset

    return _fetch


# --- insufficient_examples path ---


def test_insufficient_total_examples_returns_skipped_result():
    dataset = make_dataset(positive=2, medium=2, negative=2)

    async def fake_fetch(profile_id: str, base_url: str | None) -> OutcomeDataset:
        return dataset

    result = asyncio.run(
        bootstrap_and_retrain(
            profile_id="profile-1",
            min_examples=30,
            fetch_examples=fake_fetch,
        )
    )

    assert result.retrained is False
    assert result.promotion_decision == "skipped_min_examples"
    assert result.example_count == len(dataset.examples)
    assert result.min_examples == 30


def test_zero_examples_returns_skipped_result():
    dataset = make_dataset(positive=0, medium=0, negative=0)

    async def fake_fetch(profile_id: str, base_url: str | None) -> OutcomeDataset:
        return dataset

    result = asyncio.run(
        bootstrap_and_retrain(
            profile_id="profile-1",
            min_examples=30,
            fetch_examples=fake_fetch,
        )
    )

    assert result.retrained is False
    assert result.example_count == 0


# --- skipped_class_mix path ---


def test_bad_class_mix_all_medium_returns_skipped():
    dataset = make_dataset(positive=0, medium=35, negative=0)

    async def fake_fetch(profile_id: str, base_url: str | None) -> OutcomeDataset:
        return dataset

    result = asyncio.run(
        bootstrap_and_retrain(
            profile_id="profile-1",
            min_examples=30,
            fetch_examples=fake_fetch,
        )
    )

    assert result.retrained is False
    assert result.promotion_decision == "skipped_class_mix"
    assert result.example_count == 35


def test_bad_class_mix_missing_negatives_returns_skipped():
    # 3+ positive, 3+ medium, but only 4 negative (need 5)
    dataset = make_dataset(positive=5, medium=5, negative=4)
    total = 14

    async def fake_fetch(profile_id: str, base_url: str | None) -> OutcomeDataset:
        return dataset

    result = asyncio.run(
        bootstrap_and_retrain(
            profile_id="profile-1",
            min_examples=total,
            fetch_examples=fake_fetch,
        )
    )

    assert result.retrained is False
    assert result.promotion_decision == "skipped_class_mix"


# --- successful training path ---


@requires_lgbm
def test_successful_training_returns_retrained_result_with_model_path(tmp_path: Path):
    artifact = tmp_path / "model.json"
    # min class mix: 3 positive + 3 medium + 5 negative = 11
    dataset = make_dataset(positive=4, medium=4, negative=6)

    async def fake_fetch(profile_id: str, base_url: str | None) -> OutcomeDataset:
        return dataset

    result = asyncio.run(
        bootstrap_and_retrain(
            profile_id="profile-1",
            min_examples=len(dataset.examples),
            artifact_path=artifact,
            compatibility_model_path=artifact,
            fetch_examples=fake_fetch,
        )
    )

    assert result.retrained is True
    assert result.example_count == len(dataset.examples)
    assert result.model_path is not None
    assert result.promotion_decision is not None
    assert "promoted" in result.promotion_decision
    assert artifact.exists()


@requires_lgbm
def test_successful_training_populates_benchmark_dict(tmp_path: Path):
    artifact = tmp_path / "model.json"
    dataset = make_dataset(positive=4, medium=4, negative=6)

    async def fake_fetch(profile_id: str, base_url: str | None) -> OutcomeDataset:
        return dataset

    result = asyncio.run(
        bootstrap_and_retrain(
            profile_id="profile-1",
            min_examples=len(dataset.examples),
            artifact_path=artifact,
            compatibility_model_path=artifact,
            fetch_examples=fake_fetch,
        )
    )

    assert result.benchmark is not None
    assert "baseline_model_type" in result.benchmark
    assert "candidate_model_type" in result.benchmark
    assert "feature_set_winner" in result.benchmark
