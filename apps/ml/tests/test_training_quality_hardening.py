from app.dataset import OutcomeDataset, OutcomeExample, OutcomeRankingFeatures, OutcomeSignals
from app.trained_reranker.feature_stats import (
    compute_ablation_candidates,
    compute_feature_statistics,
)
from app.trained_reranker.validation import (
    EXPECTED_LABEL_POLICY_VERSION,
    validate_label_distribution,
)


def make_example(
    job_id: str,
    label: str,
    *,
    observed_at: str = "2026-04-15T00:00:00Z",
    matched_skill_count: int = 1,
    matched_keyword_count: int = 1,
    matched_role_count: int = 1,
    returned_count: int = 0,
    interest_rating: int | None = None,
) -> OutcomeExample:
    return OutcomeExample(
        profile_id="profile-1",
        job_id=job_id,
        source="djinni",
        role_family="engineering",
        label=label,
        label_score={"positive": 2, "medium": 1, "negative": 0}[label],
        label_reasons=[label],
        label_observed_at=observed_at,
        signals=OutcomeSignals(
            viewed=True,
            saved=label == "positive",
            hidden=label == "negative",
            bad_fit=label == "negative",
            explicit_feedback=True,
            explicit_saved=label == "positive",
            explicit_hidden=label == "negative",
            returned_count=returned_count,
            interest_rating=interest_rating,
        ),
        ranking=OutcomeRankingFeatures(
            deterministic_score=60,
            behavior_score=60,
            learned_reranker_score=60,
            matched_skill_count=matched_skill_count,
            matched_keyword_count=matched_keyword_count,
            matched_role_count=matched_role_count,
        ),
    )


def test_compute_feature_statistics_falls_back_for_small_dataset():
    stats = compute_feature_statistics(
        [
            make_example("job-1", "positive"),
            make_example("job-2", "medium"),
            make_example("job-3", "negative"),
            make_example("job-4", "negative"),
        ]
    )

    assert stats.matched_skill_count_p95 == 20.0
    assert stats.interest_rating_min == -2.0
    assert stats.interest_rating_max == 2.0


def test_compute_feature_statistics_uses_observed_bounds():
    stats = compute_feature_statistics(
        [
            make_example("job-1", "positive", matched_skill_count=1, returned_count=0, interest_rating=-1),
            make_example("job-2", "positive", matched_skill_count=2, returned_count=1, interest_rating=0),
            make_example("job-3", "medium", matched_skill_count=3, returned_count=2, interest_rating=1),
            make_example("job-4", "negative", matched_skill_count=4, returned_count=3, interest_rating=2),
            make_example("job-5", "negative", matched_skill_count=5, returned_count=4, interest_rating=-2),
        ]
    )

    assert stats.matched_skill_count_p95 == 5.0
    assert stats.returned_count_p95 == 4.0
    assert stats.interest_rating_min == -2.0
    assert stats.interest_rating_max == 2.0


def test_compute_ablation_candidates_reports_fallback_for_small_dataset():
    report = compute_ablation_candidates(
        ["learned_reranker_score", "learned_reranker_score_delta"],
        [
            make_example("job-1", "positive"),
            make_example("job-2", "negative"),
            make_example("job-3", "medium"),
            make_example("job-4", "negative"),
        ],
    )

    assert report.fallback_used is True
    assert report.fallback_reason == "insufficient_examples_for_variance"
    assert "learned_reranker_score" in report.candidates


def test_validate_label_distribution_flags_imbalance_and_policy_mismatch():
    dataset = OutcomeDataset(
        profile_id="profile-1",
        label_policy_version="outcome_label_v2",
        examples=[
            make_example(f"job-{index}", "positive", observed_at=f"2026-04-{index + 1:02d}T00:00:00Z")
            for index in range(19)
        ]
        + [
            make_example("job-negative", "negative", observed_at="2026-04-25T00:00:00Z"),
        ],
    )

    report = validate_label_distribution(dataset)

    assert report.is_imbalanced is True
    assert report.label_policy_version_mismatch is True
    assert report.healthy is False
    assert report.expected_label_policy_version == EXPECTED_LABEL_POLICY_VERSION
    assert "extreme label imbalance" in (report.reason or "")
    assert "label_policy_version mismatch" in (report.reason or "")


def test_validate_label_distribution_keeps_temporal_spread_as_warning():
    dataset = OutcomeDataset(
        profile_id="profile-1",
        label_policy_version=EXPECTED_LABEL_POLICY_VERSION,
        examples=[
            make_example("p-1", "positive"),
            make_example("p-2", "positive"),
            make_example("p-3", "positive"),
            make_example("m-1", "medium"),
            make_example("m-2", "medium"),
            make_example("m-3", "medium"),
            make_example("n-1", "negative"),
            make_example("n-2", "negative"),
            make_example("n-3", "negative"),
            make_example("n-4", "negative"),
            make_example("n-5", "negative"),
        ],
    )

    report = validate_label_distribution(dataset)

    assert report.has_insufficient_temporal_spread is True
    assert report.is_imbalanced is False
    assert report.label_policy_version_mismatch is False
    assert report.healthy is True
    assert "insufficient temporal spread" in (report.reason or "")
