from app.trained_reranker.lgbm_model import lgbm_available, lgbm_candidate_available
from app.dataset import OutcomeDataset, OutcomeExample, OutcomeRankingFeatures, OutcomeSignals


def sample_example(label: str) -> OutcomeExample:
    return OutcomeExample(
        profile_id="profile-1",
        job_id=f"job-{label}",
        title="Backend Engineer",
        company_name="Example",
        source="djinni",
        role_family="engineering",
        label_observed_at="2026-04-24T10:00:00Z",
        label=label,
        label_score={"positive": 2, "medium": 1, "negative": 0}[label],
        label_reasons=[label],
        signals=OutcomeSignals(),
        ranking=OutcomeRankingFeatures(
            deterministic_score=80,
            behavior_score_delta=0,
            behavior_score=80,
            learned_reranker_score_delta=0,
            learned_reranker_score=80,
            matched_roles=["backend_engineer"],
            matched_skills=["Rust"],
            matched_keywords=["backend"],
            matched_role_count=1,
            matched_skill_count=1,
            matched_keyword_count=1,
            fit_reasons=["shared terms: rust"],
            behavior_reasons=[],
            learned_reasons=[],
        ),
    )


def test_lgbm_available_returns_false_when_native_library_missing(monkeypatch):
    original_import = __import__

    def fake_import(name, *args, **kwargs):
        if name == "lightgbm":
            raise OSError("libgomp.so.1 missing")
        return original_import(name, *args, **kwargs)

    monkeypatch.setattr("builtins.__import__", fake_import)

    assert lgbm_available() is False


def test_lgbm_candidate_available_short_circuits_before_import(monkeypatch):
    original_import = __import__

    def fake_import(name, *args, **kwargs):
        if name == "lightgbm":
            raise AssertionError("lightgbm import should not happen for undersized datasets")
        return original_import(name, *args, **kwargs)

    monkeypatch.setattr("builtins.__import__", fake_import)

    dataset = OutcomeDataset(
        profile_id="profile-1",
        label_policy_version="outcome_label_v3",
        examples=[
            sample_example("positive"),
            sample_example("medium"),
            sample_example("negative"),
        ],
    )

    assert lgbm_candidate_available([dataset]) is False
