from __future__ import annotations

from app.bootstrap_metrics import metric_value
from app.reranker_evaluation import OutcomeDataset, RerankerEvaluationSummary, evaluate_dataset
from app.trained_reranker.bpr_model import bpr_candidate_available, train_bpr_model


def build_bpr_benchmark(
    *,
    candidate_dataset: OutcomeDataset,
    dataset: OutcomeDataset,
    selected_feature_names: list[str],
    feature_statistics,
    evaluation: RerankerEvaluationSummary,
) -> dict[str, str | float | bool]:
    logistic_hit_rate = metric_value(
        evaluation,
        "trained_reranker_prediction",
        "positive_hit_rate",
    )

    if not bpr_candidate_available([candidate_dataset]):
        return _unavailable_benchmark(logistic_hit_rate)

    try:
        bpr_model = train_bpr_model(
            [candidate_dataset],
            feature_names=selected_feature_names,
            feature_statistics=feature_statistics,
        )
        bpr_evaluation = evaluate_dataset(dataset, trained_model=bpr_model)
        bpr_hit_rate = metric_value(
            bpr_evaluation,
            "trained_reranker_prediction",
            "positive_hit_rate",
        )
    except Exception:
        return _unavailable_benchmark(logistic_hit_rate)

    return {
        "baseline_model_type": "logistic_regression",
        "candidate_model_type": "bpr",
        "baseline_positive_hit_rate": logistic_hit_rate,
        "candidate_positive_hit_rate": bpr_hit_rate,
        "candidate_available": True,
        "winner": "bpr" if bpr_hit_rate > logistic_hit_rate else "logistic_regression",
    }


def _unavailable_benchmark(logistic_hit_rate: float) -> dict[str, str | float | bool]:
    return {
        "baseline_model_type": "logistic_regression",
        "candidate_model_type": "bpr",
        "baseline_positive_hit_rate": logistic_hit_rate,
        "candidate_positive_hit_rate": 0.0,
        "candidate_available": False,
        "winner": "logistic_regression",
    }
