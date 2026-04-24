import importlib.util
from pathlib import Path
import sys


def load_verifier_module():
    script_path = (
        Path(__file__).resolve().parents[3] / "scripts" / "verify_bootstrap_pressure.py"
    )
    spec = importlib.util.spec_from_file_location("verify_bootstrap_pressure", script_path)
    assert spec is not None
    assert spec.loader is not None
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


def test_parse_bootstrap_runtime_detail_extracts_counts():
    verifier = load_verifier_module()

    counts = verifier.parse_bootstrap_runtime_detail(
        "active=2/4, available_slots=2, accepted=3, running=2, completed=5, failed=1"
    )

    assert counts == {
        "active_jobs": 2,
        "max_concurrent_jobs": 4,
        "available_slots": 2,
        "queued": 0,
        "accepted": 3,
        "running": 2,
        "completed": 5,
        "failed": 1,
    }


def test_build_profile_sequence_cycles_input_profiles():
    verifier = load_verifier_module()

    sequence = verifier.build_profile_sequence(["profile-1", "profile-2"], 5)

    assert sequence == [
        "profile-1",
        "profile-2",
        "profile-1",
        "profile-2",
        "profile-1",
    ]


def test_non_negative_duration_ms_clamps_clock_skew():
    verifier = load_verifier_module()

    start = verifier.parse_iso8601("2026-04-24T10:00:00.900000Z")
    finish = verifier.parse_iso8601("2026-04-24T10:00:00Z")

    duration_ms = verifier.non_negative_duration_ms(start, finish)

    assert duration_ms == 0.0


def test_normalize_runtime_sample_subtracts_baseline_counts():
    verifier = load_verifier_module()

    normalized = verifier.normalize_runtime_sample(
        {
            "active_jobs": 2,
            "max_concurrent_jobs": 2,
            "available_slots": 0,
            "queued": 2,
            "accepted": 3,
            "running": 2,
            "completed": 7,
            "failed": 4,
        },
        {
            "active_jobs": 0,
            "max_concurrent_jobs": 2,
            "available_slots": 2,
            "queued": 0,
            "accepted": 1,
            "running": 0,
            "completed": 5,
            "failed": 4,
        },
    )

    assert normalized == {
        "active_jobs": 2,
        "max_concurrent_jobs": 2,
        "available_slots": 0,
        "queued": 2,
        "accepted": 2,
        "running": 2,
        "completed": 2,
        "failed": 0,
    }


def test_summarize_task_results_aggregates_runtime_and_outcomes():
    verifier = load_verifier_module()

    summary = verifier.summarize_task_results(
        [
            {
                "task_id": "task-1",
                "status": "completed",
                "retrained": True,
                "duration_ms": 1200.0,
                "queue_delay_ms": 150.0,
                "promotion_decision": "promoted",
            },
            {
                "task_id": "task-2",
                "status": "completed",
                "retrained": False,
                "duration_ms": 800.0,
                "queue_delay_ms": 50.0,
                "promotion_decision": "skipped_min_examples",
                "reason": "need at least 15 examples, got 3",
            },
            {
                "task_id": "task-3",
                "status": "failed",
                "reason": "engine-api unreachable",
            },
        ],
        [
            {
                "active_jobs": 2,
                "max_concurrent_jobs": 2,
                "available_slots": 0,
                "queued": 1,
                "accepted": 1,
                "running": 2,
                "completed": 0,
                "failed": 0,
            },
            {
                "active_jobs": 0,
                "max_concurrent_jobs": 2,
                "available_slots": 2,
                "queued": 0,
                "accepted": 0,
                "running": 0,
                "completed": 2,
                "failed": 1,
            },
        ],
    )

    assert summary["task_count"] == 3
    assert summary["terminal_status_counts"] == {
        "completed": 2,
        "failed": 1,
    }
    assert summary["retrained_count"] == 1
    assert summary["completed_without_retrain_count"] == 1
    assert summary["promotion_decisions"] == {
        "promoted": 1,
        "skipped_min_examples": 1,
    }
    assert summary["reasons"] == {
        "need at least 15 examples, got 3": 1,
        "engine-api unreachable": 1,
    }
    assert summary["runtime_observations"] == {
        "sample_count": 2,
        "saturation_samples": 1,
        "max_active_jobs": 2,
        "max_queued_jobs": 1,
        "max_accepted_queue": 1,
        "max_running_tasks": 2,
        "max_completed_tasks": 2,
        "max_failed_tasks": 1,
    }
    assert summary["duration_ms"] == {
        "mean": 1000.0,
        "p50": 800.0,
        "p95": 1200.0,
        "max": 1200.0,
    }
    assert summary["queue_delay_ms"] == {
        "mean": 100.0,
        "p50": 50.0,
        "p95": 150.0,
        "max": 150.0,
    }
